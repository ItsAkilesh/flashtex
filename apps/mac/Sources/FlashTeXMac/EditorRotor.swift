import AppKit
import FlashTeXAccessibility

/// VoiceOver rotor over the source editor (lane mac-editor-a11y-3): the
/// "Headings" rotor (`\chapter` … `\paragraph`) and an "Environments" rotor,
/// both from `AccessibleEditorModel.rotorItems`. Read-only: a chosen item
/// becomes the text view's selection (AppKit moves the selection to the
/// result's `targetRange`); nothing edits the buffer.
///
/// Search semantics follow `NSAccessibilityCustomRotor.SearchParameters`:
/// no current item → the first (next) or last (previous) item; a current
/// item whose range location is `NSNotFound` → from the first/last character;
/// otherwise strictly after / strictly before the current item's location, so
/// the caret position VoiceOver passes as the current item finds the next
/// heading even when it is off-screen. `filterString` is a case-insensitive
/// substring match on the item label (type-ahead). No wrap-around: nil at
/// either end, which is what VoiceOver expects ("no more headings").
@MainActor
final class EditorRotorSearch: NSObject, NSAccessibilityCustomRotorItemSearchDelegate {
    typealias Category = AccessibleEditorModel.RotorCategory
    typealias Item = AccessibleEditorModel.RotorItem

    private(set) weak var textView: NSTextView?
    /// Categories exposed, in rotor order. Diagnostics and captures live in
    /// `ShellModel` (marks/anchor), not in the text view, so they are not here.
    static let categories: [Category] = [.headings, .environments]

    /// Items per category, valid for one `(edit generation, length)` of the storage.
    private var cache: (generation: Int, length: Int, items: [Category: [Item]])?
    private var generation = 0
    private var storageObserver: NSObjectProtocol?
    /// Evidence for tests: how many times the model was rebuilt.
    private(set) var rebuilds = 0

    init(textView: NSTextView) {
        self.textView = textView
        super.init()
        rotors = Self.categories.map { category in
            let rotor = category == .headings
                ? NSAccessibilityCustomRotor(rotorType: .heading, itemSearchDelegate: self)
                : NSAccessibilityCustomRotor(label: category.title, itemSearchDelegate: self)
            rotorCategories[ObjectIdentifier(rotor)] = category
            return rotor
        }
        if let storage = textView.textStorage {
            storageObserver = NotificationCenter.default.addObserver(
                forName: NSTextStorage.didProcessEditingNotification, object: storage, queue: nil
            ) { [weak self] note in
                guard let storage = note.object as? NSTextStorage, storage.editedMask.contains(.editedCharacters) else { return }
                MainActor.assumeIsolated { self?.invalidate() }
            }
        }
    }

    deinit {
        if let storageObserver { NotificationCenter.default.removeObserver(storageObserver) }
    }

    /// Drops the cached items (an edit happened, or the owner replaced the text).
    func invalidate() { generation += 1 }

    /// The rotors AppKit returns from `accessibilityCustomRotors`. The
    /// headings rotor uses the built-in `.heading` type so VoiceOver lists it
    /// under its own Headings rotor; environments are a custom-label rotor.
    private(set) var rotors: [NSAccessibilityCustomRotor] = []
    private var rotorCategories: [ObjectIdentifier: Category] = [:]

    func category(of rotor: NSAccessibilityCustomRotor) -> Category? { rotorCategories[ObjectIdentifier(rotor)] }

    /// Items of `category` for the current text (cached until the next edit).
    func items(_ category: Category) -> [Item] {
        guard let textView else { return [] }
        let length = textView.textStorage?.length ?? (textView.string as NSString).length
        if let cache, cache.generation == generation, cache.length == length { return cache.items[category] ?? [] }
        // One model per generation: every category comes from the same text.
        let model = AccessibleEditorModel(text: SourceEditorView.nativeText(of: textView))
        var items: [Category: [Item]] = [:]
        for c in Self.categories { items[c] = model.rotorItems(c) }
        rebuilds += 1
        cache = (generation, length, items)
        return items[category] ?? []
    }

    /// Where a search starts, as AppKit describes it: nil for "from the
    /// first/last item, inclusive"; `.character(offset)` for a current item
    /// with a real range (strictly after/before it).
    enum Start: Equatable { case fromEnds, character(Int) }

    static func start(of parameters: NSAccessibilityCustomRotor.SearchParameters) -> Start {
        guard let current = parameters.currentItem else { return .fromEnds }
        let range = current.targetRange
        if range.location == NSNotFound { return .fromEnds }
        return .character(range.location)
    }

    /// Pure search over sorted `items` (document order). `filter` empty matches all.
    static func resolve(items: [Item], start: Start, forward: Bool, filter: String) -> Item? {
        let matching = filter.isEmpty ? items : items.filter { $0.label.localizedCaseInsensitiveContains(filter) }
        switch (start, forward) {
        case (.fromEnds, true): return matching.first
        case (.fromEnds, false): return matching.last
        case (.character(let at), true): return matching.first { $0.utf16.location > at }
        case (.character(let at), false): return matching.last { $0.utf16.location < at }
        }
    }

    func result(for item: Item) -> NSAccessibilityCustomRotor.ItemResult? {
        guard let textView else { return nil }
        let result = NSAccessibilityCustomRotor.ItemResult(targetElement: textView)
        result.targetRange = item.utf16
        result.customLabel = item.label
        return result
    }

    nonisolated func rotor(_ rotor: NSAccessibilityCustomRotor,
                           resultFor searchParameters: NSAccessibilityCustomRotor.SearchParameters) -> NSAccessibilityCustomRotor.ItemResult? {
        MainActor.assumeIsolated {
            guard let category = category(of: rotor) else { return nil }
            let found = Self.resolve(items: items(category), start: Self.start(of: searchParameters),
                                     forward: searchParameters.searchDirection == .next,
                                     filter: searchParameters.filterString)
            return found.flatMap(result(for:))
        }
    }
}
