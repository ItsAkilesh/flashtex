import AppKit
import SwiftUI
import FlashTeXProtocol

/// NSTextView wrapper. Uses a monospaced font, reports edits, applies UTF-16
/// selections requested by preview navigation, and underlines diagnostic marks
/// with layout-manager temporary attributes (never touching the text storage,
/// so undo and the `text` binding are unaffected).
///
/// Responsiveness and accessibility rules (owner: mac-editor-accessibility):
/// - Marks are painted only over a window around the visible text
///   (`MarkPainter`), so 200 marks on a 60 KB buffer cost a fraction of a
///   millisecond per change; scrolling paints the newly exposed window.
/// - A navigation selection never fights typing: it waits while the text view
///   has marked (IME) text, and while the user is actively typing it is
///   deferred instead of moving the caret backwards. It is applied as soon as
///   typing pauses (`typingGuardNs`), and only if it is still the newest token.
/// - VoiceOver: the view is labelled, and every selection change that is not
///   a plain typing step announces "Line L, column C" (or the selection
///   extent), coalesced to one announcement per run-loop turn.
/// - A capture insertion is exactly one undo step: typing coalescing is
///   broken on both sides, the change goes through `shouldChangeText` /
///   `didChangeText`, and the model learns about it once through
///   `onEditApplied` (the binding is not written during the view update).
/// - Input methods: while the view has marked text (IME composition, dead
///   keys) nothing reaches the binding — AppKit does not post a text change
///   for `setMarkedText`, and a commit that still had marked text is held
///   back — so no revision/compile per composition step; the caret is
///   reported at the composition start (a position of the model's text);
///   composition steps are not announced; the completion list is closed and
///   its pending scan cancelled; the view is not re-synced from the model
///   until the composition ends.
struct SourceEditorView: NSViewRepresentable {
    @Binding var text: String
    var selection: ShellModel.Selection?
    var pendingEdit: ShellModel.PendingEdit?
    var marks: [EditorDiagnostics.Mark] = []
    var result: RuntimeV1.CompileResult? // for completion (Completion.swift)
    /// Editor revision the buffer is at; completion metadata binds to it.
    var editorRevision: Int?
    var projectIndexMetadata: Completion.Metadata?
    var onCaretChange: (Int) -> Void = { _ in }
    var onSelectionChange: (NSRange) -> Void = { _ in }
    var onEditApplied: (ShellModel.PendingEdit, String) -> Void = { _, _ in }

    /// A navigation selection that would move the caret backwards is deferred
    /// while the last user edit is younger than this.
    static let typingGuardNs: UInt64 = 350_000_000

    func makeCoordinator() -> Coordinator { Coordinator(self) }

    func makeNSView(context: Context) -> NSScrollView {
        let scroll = CompletingTextView.scrollable() // Completion.swift
        let tv = scroll.documentView as! NSTextView
        // Marks are TextKit 1 temporary attributes. Touching `layoutManager` on
        // an empty view selects TextKit 1 now, instead of a full re-layout of a
        // large document the first time a diagnostic arrives.
        _ = tv.layoutManager
        tv.delegate = context.coordinator
        // Font, tab interval, wrapping and appearance follow EditorPreferences (applied now and on every change).
        context.coordinator.preferencesToken = EditorPreferences.shared.observeApplying(to: tv)
        tv.isRichText = false
        tv.isAutomaticQuoteSubstitutionEnabled = false
        tv.isAutomaticDashSubstitutionEnabled = false
        tv.isAutomaticTextReplacementEnabled = false
        tv.allowsUndo = true
        tv.textContainerInset = NSSize(width: 8, height: 8)
        tv.setAccessibilityLabel("LaTeX source") // FlashTeXAccessibility: VoiceOver names the editor
        tv.setAccessibilityHelp("LaTeX source editor. Moving the selection announces the line and column.")
        tv.string = text
        context.coordinator.attach(scroll)
        return scroll
    }

    func updateNSView(_ scroll: NSScrollView, context: Context) {
        let tv = scroll.documentView as! NSTextView
        let co = context.coordinator
        co.parent = self
        (tv as? CompletingTextView)?.compileResult = result
        (tv as? CompletingTextView)?.editorRevision = editorRevision
        if let m = projectIndexMetadata { _ = (tv as? CompletingTextView)?.accept(projectIndex: m) }
        if let edit = pendingEdit, edit.token != co.appliedEditToken {
            co.appliedEditToken = edit.token
            co.applyPendingEdit(edit, to: tv)
            return
        }
        // Until `onEditApplied` has delivered an applied edit, the binding still
        // holds the pre-edit text; resetting the view from it would undo the edit.
        // While marked text exists the storage is ahead of the model by the
        // composition; every sync waits for the commit (which updates the model
        // and brings the next update here).
        guard !co.awaitingEditDelivery, !tv.hasMarkedText() else { return }
        // `tv.string` bridges a fresh copy and compares it character by character
        // (Unicode-normalized) on every update. The coordinator keeps the exact
        // String instance last exchanged with the text view; when the binding
        // still holds that instance the comparison is a pointer check.
        var textReset = false
        if text != co.lastKnownText {
            co.programmaticChanges += 1
            tv.string = text // drops temporary attributes; repaint marks below
            co.programmaticChanges -= 1
            co.lastKnownText = text
            textReset = true
        }
        co.marks.update(marks, in: tv, reset: textReset)
        if let selection, selection.token != co.appliedToken {
            co.appliedToken = selection.token
            co.applySelection(selection, to: tv)
        }
    }

    /// Replaces underline/tooltip temporary attributes over the whole text.
    /// Ranges outside the current string are skipped; errors are applied after
    /// warnings so an error wins where they overlap.
    @MainActor
    static func applyMarks(_ marks: [EditorDiagnostics.Mark], to tv: NSTextView) {
        guard let lm = tv.layoutManager else { return }
        let whole = NSRange(location: 0, length: (tv.string as NSString).length)
        MarkPainter.paint(marks, window: whole, length: whole.length, layoutManager: lm)
    }

    // MARK: accessibility

    /// Announcement for a selection: "Line 3, column 5" for a caret, or
    /// "Selected N characters, line 3 column 5 to line 4 column 2" for a range.
    /// Lines and columns are 1-based; the column counts user-perceived
    /// characters (grapheme clusters) from the start of the line.
    /// Nil when `range` is not a valid UTF-16 range of `text`.
    static func selectionAnnouncement(text: String, range: NSRange) -> String? {
        guard range.location >= 0, range.length >= 0,
              let start = lineColumn(text: text, utf16: range.location) else { return nil }
        if range.length == 0 { return "Line \(start.line), column \(start.column)" }
        guard let end = lineColumn(text: text, utf16: NSMaxRange(range)),
              let r = Range(range, in: text) else { return nil }
        let count = text[r].count
        let chars = count == 1 ? "1 character" : "\(count) characters"
        if start.line == end.line {
            return "Selected \(chars), line \(start.line) column \(start.column) to \(end.column)"
        }
        return "Selected \(chars), line \(start.line) column \(start.column) to line \(end.line) column \(end.column)"
    }

    /// 1-based line and column for a UTF-16 offset, nil when out of range or
    /// inside a surrogate pair. `\n`, `\r` and `\r\n` each end a line (NSString
    /// line semantics for the breaks LaTeX sources contain). Newlines in the
    /// prefix are counted with `memchr` over the UTF-8 storage.
    static func lineColumn(text: String, utf16: Int) -> (line: Int, column: Int)? {
        guard let index = scalarIndex(text: text, utf16: utf16) else { return nil }
        let byte = text.utf8.distance(from: text.utf8.startIndex, to: index)
        var line = 1
        var lineStartByte = 0
        var scanned = false
        var copy = text
        copy.withUTF8 { buffer in
            guard let base = buffer.baseAddress else { return }
            scanned = true
            var p = 0
            while p < byte, let hit = memchr(base + p, 0x0A, byte - p) {
                p = UnsafePointer<UInt8>(hit.assumingMemoryBound(to: UInt8.self)) - base + 1
                line += 1; lineStartByte = p
            }
            p = 0
            while p < byte, let hit = memchr(base + p, 0x0D, byte - p) {
                p = UnsafePointer<UInt8>(hit.assumingMemoryBound(to: UInt8.self)) - base + 1
                if p < buffer.count, buffer[p] == 0x0A { continue } // CRLF: counted by the LF pass
                line += 1; lineStartByte = max(lineStartByte, p)
            }
        }
        guard scanned else { return nil }
        let lineStart = text.utf8.index(text.utf8.startIndex, offsetBy: lineStartByte)
        return (line, text[lineStart..<index].count + 1)
    }

    /// `String.Index` for a UTF-16 offset on a scalar boundary; nil when out of
    /// range or inside a surrogate pair (`Range(NSRange, in:)` would snap that).
    static func scalarIndex(text: String, utf16: Int) -> String.Index? {
        guard utf16 >= 0, utf16 <= text.utf16.count,
              let index = text.utf16.index(text.utf16.startIndex, offsetBy: utf16, limitedBy: text.utf16.endIndex),
              index.samePosition(in: text.unicodeScalars) != nil else { return nil }
        return index
    }

    /// UTF-8 byte offset of a UTF-16 caret position in `text` (nil when it is
    /// out of range or inside a surrogate pair): the contract coordinate for
    /// the caret.
    static func caretByte(text: String, utf16: Int) -> Int? {
        scalarIndex(text: text, utf16: utf16).map { text.utf8.distance(from: text.utf8.startIndex, to: $0) }
    }

    /// The text view's contents as a native (contiguous UTF-8) `String`.
    /// `tv.string` bridges the storage lazily: every later byte-wise use of
    /// it — the model's `sameBytes`, UTF-8 caret offsets, `==` — transcodes
    /// the whole NSString again (measured 2–4 ms per keystroke on a 60 KB
    /// buffer with non-ASCII text). One `getBytes` pass costs ~0.2 ms and
    /// makes everything downstream a pointer check or O(1).
    static func nativeText(of tv: NSTextView) -> String {
        let ns = (tv.textStorage?.string ?? tv.string) as NSString
        let length = ns.length
        guard length > 0 else { return "" }
        let capacity = length * 3 // a UTF-16 unit never needs more than 3 UTF-8 bytes
        var complete = false
        let native = String(unsafeUninitializedCapacity: capacity) { buffer in
            var used = 0
            var remaining = NSRange(location: 0, length: 0)
            let ok = ns.getBytes(buffer.baseAddress, maxLength: capacity, usedLength: &used,
                                 encoding: String.Encoding.utf8.rawValue, options: [],
                                 range: NSRange(location: 0, length: length), remaining: &remaining)
            complete = ok && remaining.length == 0
            return complete ? used : 0
        }
        if complete { return native }
        // Unpaired surrogates cannot be encoded; the bridge replaces them.
        var bridged = tv.string
        bridged.makeContiguousUTF8()
        return bridged
    }

    // MARK: marks

    /// Paints diagnostic marks as temporary attributes over a window around the
    /// visible text only, and extends the painted range as the view scrolls.
    /// Marks outside the window cost nothing until they scroll into view.
    @MainActor
    final class MarkPainter {
        static let keys: [NSAttributedString.Key] = [.underlineStyle, .underlineColor, .toolTip]
        /// Extra characters painted on each side of the visible range so short
        /// scrolls need no repaint.
        static let padding = 4_000

        private(set) var marks: [EditorDiagnostics.Mark] = []
        /// Disjoint, sorted ranges whose temporary attributes match `marks`
        /// (over-approximated across edits, see `noteEdit`).
        private(set) var painted: [NSRange] = []
        /// Count of passes that painted something (tests and evidence).
        private(set) var paints = 0

        /// New marks (or a text reset, which drops all temporary attributes).
        func update(_ new: [EditorDiagnostics.Mark], in tv: NSTextView, reset: Bool) {
            guard reset || new != marks else { return }
            guard let lm = tv.layoutManager else { return }
            if !reset {
                // Clear only what was painted; the layout manager kept these
                // ranges aligned with edits made since.
                let whole = NSRange(location: 0, length: tv.textStorage?.length ?? 0)
                for range in painted {
                    let stale = NSIntersectionRange(range, whole)
                    guard stale.length > 0 else { continue }
                    for key in Self.keys { lm.removeTemporaryAttribute(key, forCharacterRange: stale) }
                }
            }
            painted = []
            marks = new
            extend(to: Self.window(for: tv), in: tv, layoutManager: lm)
        }

        /// The view scrolled: paint marks over any part of the new window not yet painted.
        func scrolled(_ tv: NSTextView) {
            guard !marks.isEmpty, let lm = tv.layoutManager else { return }
            let window = Self.window(for: tv)
            guard window.length > 0, !gaps(in: window).isEmpty else { return }
            extend(to: window, in: tv, layoutManager: lm)
        }

        /// The text view is about to replace `range` with `replacementLength`
        /// UTF-16 units: grow the painted ranges so they still cover every
        /// attribute the layout manager shifts.
        func noteEdit(range: NSRange, replacementLength: Int) {
            let growth = max(0, replacementLength - range.length)
            for i in painted.indices where NSMaxRange(painted[i]) >= range.location {
                painted[i] = NSUnionRange(painted[i], NSRange(location: range.location, length: 0))
                painted[i].length += growth
            }
            painted = Self.merged(painted)
        }

        /// Parts of `window` not covered by `painted`.
        func gaps(in window: NSRange) -> [NSRange] {
            var result: [NSRange] = []
            var cursor = window.location
            for range in painted where NSMaxRange(range) > cursor {
                if range.location >= NSMaxRange(window) { break }
                if range.location > cursor { result.append(NSRange(location: cursor, length: range.location - cursor)) }
                cursor = max(cursor, NSMaxRange(range))
            }
            if cursor < NSMaxRange(window) { result.append(NSRange(location: cursor, length: NSMaxRange(window) - cursor)) }
            return result
        }

        private func extend(to window: NSRange, in tv: NSTextView, layoutManager lm: NSLayoutManager) {
            let length = tv.textStorage?.length ?? 0
            let gaps = gaps(in: window)
            if !marks.isEmpty, !gaps.isEmpty {
                // Only the uncovered parts are painted; a mark straddling a
                // boundary is clipped to each part with identical attributes.
                for gap in gaps {
                    Self.paint(marks, window: NSIntersectionRange(gap, NSRange(location: 0, length: length)),
                               length: length, layoutManager: lm)
                }
                paints += 1
            }
            painted = Self.merged(painted + [window])
        }

        /// Sorted union of `ranges` (overlapping or adjacent ranges merged).
        static func merged(_ ranges: [NSRange]) -> [NSRange] {
            var result: [NSRange] = []
            for range in ranges.sorted(by: { $0.location < $1.location }) where range.length > 0 {
                if let last = result.last, range.location <= NSMaxRange(last) {
                    result[result.count - 1] = NSUnionRange(last, range)
                } else {
                    result.append(range)
                }
            }
            return result
        }

        /// Characters in the visible rect plus padding; the whole text when
        /// the view is not laid out in a window (unit tests, offscreen views).
        static func window(for tv: NSTextView) -> NSRange {
            let length = (tv.string as NSString).length
            guard let lm = tv.layoutManager, let container = tv.textContainer, tv.window != nil,
                  !tv.visibleRect.isEmpty else { return NSRange(location: 0, length: length) }
            let glyphs = lm.glyphRange(forBoundingRect: tv.visibleRect, in: container)
            let visible = lm.characterRange(forGlyphRange: glyphs, actualGlyphRange: nil)
            let start = max(0, visible.location - padding)
            let end = min(length, NSMaxRange(visible) + padding)
            return NSRange(location: start, length: max(0, end - start))
        }

        /// Clears this painter's keys over `window` and paints every mark that
        /// intersects it (clipped to the text). Errors are applied after
        /// warnings so an error wins where they overlap.
        static func paint(_ marks: [EditorDiagnostics.Mark], window: NSRange, length: Int, layoutManager lm: NSLayoutManager) {
            guard window.length > 0 else { return }
            for key in keys { lm.removeTemporaryAttribute(key, forCharacterRange: window) }
            let warningAttrs = attributes(for: .warning), errorAttrs = attributes(for: .error)
            for severity in [RuntimeV1.Severity.warning, .error] {
                let base = severity == .error ? errorAttrs : warningAttrs
                for mark in marks where mark.severity == severity {
                    let r = mark.nsRange
                    guard r.location >= 0, r.length > 0, NSMaxRange(r) <= length else { continue }
                    let clipped = NSIntersectionRange(r, window)
                    guard clipped.length > 0 else { continue }
                    var attrs = base
                    attrs[.toolTip] = mark.toolTip
                    lm.addTemporaryAttributes(attrs, forCharacterRange: clipped)
                }
            }
        }

        private static func attributes(for severity: RuntimeV1.Severity) -> [NSAttributedString.Key: Any] {
            [.underlineStyle: NSUnderlineStyle.thick.rawValue | NSUnderlineStyle.patternDot.rawValue,
             .underlineColor: severity == .error ? NSColor.systemRed : NSColor.systemOrange]
        }
    }

    // MARK: coordinator

    @MainActor
    final class Coordinator: NSObject, NSTextViewDelegate {
        var parent: SourceEditorView
        var appliedToken: Int
        var appliedEditToken = 0
        /// Keeps EditorPreferences applied to the text view (EditorPreferences.swift).
        var preferencesToken: EditorPreferences.ObservationToken?
        let marks = MarkPainter()
        /// The String instance last set on, or read from, the text view.
        var lastKnownText: String
        /// > 0 while this coordinator itself edits the text view (string reset,
        /// pending edit, navigation selection); the delegate then neither writes
        /// the binding nor announces.
        var programmaticChanges = 0
        /// A pending edit was applied to the view but `onEditApplied` has not run yet.
        private(set) var awaitingEditDelivery = false
        /// Monotonic time of the last text change the user made (0 = never).
        private(set) var lastUserEditNs: UInt64 = 0
        /// Thread CPU time at the last user text change (benchmark evidence:
        /// the delegate -> binding round trip net of preemption).
        private(set) var lastUserEditCpuNs: UInt64 = 0
        /// Navigation selection waiting for a typing pause or the end of an IME composition.
        private(set) var deferredSelection: ShellModel.Selection?
        private var deferredTimer: Timer?
        /// True from a text change until the end of the run-loop turn: selection
        /// changes in that turn are typing steps, not caret moves.
        private var textChangedThisTurn = false
        private var announcementPending = false
        /// True while the view has marked text (an IME composition or dead key).
        var composing: Bool { textView?.hasMarkedText() ?? false }
        /// Composition selection changes observed (tests and evidence).
        private(set) var compositionSteps = 0
        /// VoiceOver sink; tests replace it to observe announcements.
        var announce: (String) -> Void = { _ in }
        /// Announcements posted (tests and evidence).
        private(set) var announcements: [String] = []
        private weak var scrollView: NSScrollView?
        private var boundsObserver: NSObjectProtocol?

        init(_ parent: SourceEditorView) {
            self.parent = parent
            lastKnownText = parent.text
            // A recreated view must not replay the last navigation (that would
            // move the caret back to an old target); a pending edit is applied
            // because the model is still waiting for `onEditApplied`.
            appliedToken = parent.selection?.token ?? 0
            super.init()
            announce = { [weak self] message in self?.post(message) }
        }

        deinit {
            if let boundsObserver { NotificationCenter.default.removeObserver(boundsObserver) }
            deferredTimer?.invalidate()
        }

        func attach(_ scroll: NSScrollView) {
            scrollView = scroll
            scroll.contentView.postsBoundsChangedNotifications = true
            boundsObserver = NotificationCenter.default.addObserver(
                forName: NSView.boundsDidChangeNotification, object: scroll.contentView, queue: nil
            ) { [weak self, weak scroll] _ in
                MainActor.assumeIsolated {
                    guard let self, let tv = scroll?.documentView as? NSTextView else { return }
                    self.marks.scrolled(tv)
                }
            }
        }

        var textView: NSTextView? { scrollView?.documentView as? NSTextView }

        // MARK: pending edit (one undo step)

        func applyPendingEdit(_ edit: ShellModel.PendingEdit, to tv: NSTextView) {
            let ns = edit.nsRange
            var applied = false
            if ns.location >= 0, NSMaxRange(ns) <= (tv.textStorage?.length ?? 0) {
                tv.breakUndoCoalescing() // preceding typing stays its own undo step
                if tv.shouldChangeText(in: ns, replacementString: edit.text) {
                    programmaticChanges += 1
                    tv.textStorage?.replaceCharacters(in: ns, with: edit.text)
                    tv.didChangeText() // registers undo, fires textDidChange
                    tv.undoManager?.setActionName("Insert Capture")
                    tv.breakUndoCoalescing() // following typing starts a new step
                    let inserted = NSRange(location: ns.location, length: (edit.text as NSString).length)
                    tv.setSelectedRange(inserted)
                    tv.scrollRangeToVisible(inserted)
                    tv.showFindIndicator(for: inserted)
                    tv.window?.makeFirstResponder(tv)
                    programmaticChanges -= 1
                    applied = true
                }
            }
            let s = SourceEditorView.nativeText(of: tv)
            lastKnownText = s
            if applied { announceNow(text: s, range: tv.selectedRange(), prefix: "Inserted capture. ") }
            // The model is updated outside the SwiftUI view update; `editApplied`
            // bumps the revision and reaches the bridge/ledger through
            // `updateActiveText` exactly once.
            awaitingEditDelivery = true
            let onEditApplied = parent.onEditApplied
            DispatchQueue.main.async { [weak self] in
                self?.awaitingEditDelivery = false
                onEditApplied(edit, s)
            }
        }

        // MARK: navigation selection

        func applySelection(_ selection: ShellModel.Selection, to tv: NSTextView) {
            let range = selection.nsRange
            guard range.location >= 0, range.length >= 0, NSMaxRange(range) <= (tv.textStorage?.length ?? 0) else {
                deferredSelection = nil; return // no longer a range of the buffer
            }
            if tv.hasMarkedText() {
                deferSelection(selection, for: 0.1); return
            }
            let sinceEdit = MonotonicClock.nowNs() &- lastUserEditNs
            if lastUserEditNs != 0, sinceEdit < SourceEditorView.typingGuardNs,
               range.location < tv.selectedRange().location {
                deferSelection(selection, for: Double(SourceEditorView.typingGuardNs &- sinceEdit) / 1e9); return
            }
            deferredSelection = nil
            deferredTimer?.invalidate()
            programmaticChanges += 1
            tv.setSelectedRange(range)
            tv.scrollRangeToVisible(range) // scrolls only when the range is off screen
            tv.showFindIndicator(for: range)
            tv.window?.makeFirstResponder(tv)
            programmaticChanges -= 1
            announceNow(text: currentText(of: tv), range: range, prefix: "")
        }

        private func deferSelection(_ selection: ShellModel.Selection, for seconds: TimeInterval) {
            deferredSelection = selection
            deferredTimer?.invalidate()
            let timer = Timer(timeInterval: max(0.01, seconds), repeats: false) { [weak self] _ in
                MainActor.assumeIsolated { self?.retryDeferredSelection() }
            }
            RunLoop.main.add(timer, forMode: .common)
            deferredTimer = timer
        }

        private func retryDeferredSelection() {
            guard let selection = deferredSelection, parent.selection?.token == selection.token,
                  let tv = textView else { deferredSelection = nil; return }
            applySelection(selection, to: tv)
        }

        // MARK: delegate

        func textView(_ textView: NSTextView, shouldChangeTextIn range: NSRange, replacementString: String?) -> Bool {
            marks.noteEdit(range: range, replacementLength: (replacementString as NSString?)?.length ?? 0)
            return true
        }

        func textDidChange(_ notification: Notification) {
            guard let tv = notification.object as? NSTextView else { return }
            TypingBench.shared.textViewDidChange() // stamps the delegate time for keystroke -> paint
            if !textChangedThisTurn {
                textChangedThisTurn = true
                DispatchQueue.main.async { [weak self] in self?.textChangedThisTurn = false }
            }
            guard programmaticChanges == 0 else { return }
            lastUserEditNs = MonotonicClock.nowNs()
            lastUserEditCpuNs = clock_gettime_nsec_np(CLOCK_THREAD_CPUTIME_ID)
            // A change that leaves marked text behind is a composition step:
            // the model sees the buffer once the composition is committed.
            guard !tv.hasMarkedText() else { return }
            let s = SourceEditorView.nativeText(of: tv)
            lastKnownText = s
            parent.text = s
        }

        func textViewDidChangeSelection(_ notification: Notification) {
            guard let tv = notification.object as? NSTextView else { return }
            if tv.hasMarkedText() { compositionStep(tv); return }
            let range = tv.selectedRange()
            parent.onCaretChange(range.location)
            parent.onSelectionChange(range)
            // A typing step already reads as typed text in VoiceOver; only
            // caret/selection moves are announced, once per run-loop turn.
            // (`textChangedThisTurn` is checked again when the turn ends because
            // AppKit may post the selection change before the text change.)
            guard programmaticChanges == 0, !textChangedThisTurn, !announcementPending else { return }
            announcementPending = true
            DispatchQueue.main.async { [weak self] in
                guard let self else { return }
                announcementPending = false
                guard !textChangedThisTurn, let tv = textView, tv.window?.firstResponder === tv else { return }
                announceNow(text: currentText(of: tv), range: tv.selectedRange(), prefix: "")
            }
        }

        // MARK: input method composition

        /// `setMarkedText` posts no text change, only selection changes. The
        /// caret the model hears is the composition start — a position of the
        /// text it holds — nothing is announced, and the completion list (whose
        /// key path re-scans after every keystroke) is closed with its pending
        /// scan cancelled at the head of the next run-loop turn, i.e. after the
        /// keystroke that started the step has enqueued that scan.
        private func compositionStep(_ tv: NSTextView) {
            compositionSteps += 1
            lastUserEditNs = MonotonicClock.nowNs() // composing is typing for the navigation guard
            let start = tv.markedRange().location
            if start != NSNotFound {
                parent.onCaretChange(start)
                parent.onSelectionChange(NSRange(location: start, length: 0))
            }
            guard let completing = tv as? CompletingTextView else { return }
            TypingBench.nextRunLoopTurn { [weak completing] in
                guard let completing, completing.hasMarkedText() else { return }
                completing.scheduler.cancel()
                completing.close(.textChanged)
            }
        }

        // MARK: announcements

        /// The native text last exchanged with the view when it still matches
        /// the storage length, else a fresh native copy (never the lazy bridge).
        func currentText(of tv: NSTextView) -> String {
            if let length = tv.textStorage?.length, lastKnownText.utf16.count == length { return lastKnownText }
            return SourceEditorView.nativeText(of: tv)
        }

        func announceNow(text: String, range: NSRange, prefix: String) {
            guard let message = SourceEditorView.selectionAnnouncement(text: text, range: range) else { return }
            announcements.append(prefix + message)
            if announcements.count > 64 { announcements.removeFirst(announcements.count - 64) }
            announce(prefix + message)
        }

        private func post(_ message: String) {
            guard let tv = textView else { return }
            NSAccessibility.post(element: tv, notification: .announcementRequested,
                                 userInfo: [.announcement: message, .priority: NSAccessibilityPriorityLevel.low.rawValue])
        }
    }
}
