import AppKit
import UniformTypeIdentifiers
import FlashTeXProtocol

/// Capture-bridge lifecycle for the shell (contract: transfer-v1, bridge commit
/// b5ca96b). The Mac owns UI review, the document transaction and the bridge
/// process: it opens the active document, streams edits, pins destinations,
/// submits captures, converts them, verifies prepared edits and applies each
/// one exactly once through the editor's undo manager.
extension ShellModel {
    var bridgeAttached: Bool { bridge?.running == true }

    func isBridgeCapture(_ captureId: String) -> Bool { bridge?.capture(captureId) != nil }

    /// The most recent capture that can still be converted or reviewed.
    var latestConvertibleCapture: BridgeSession.Capture? {
        bridgeCaptures.last { $0.state == .received || $0.state == .proposed }
    }

    // MARK: attach / detach

    /// Attaches `$FLASHTEX_BRIDGE`, a bundled `flashtex-bridge`, or
    /// `crates/bridge/target/{release,debug}/flashtex-bridge`.
    @discardableResult
    func attachDiscoveredBridge() -> Bool {
        guard let url = BridgeClient.locateBridge() else {
            bridgeStatus = "no built flashtex-bridge found (build crates/bridge or set FLASHTEX_BRIDGE)"
            return false
        }
        attachBridge(executable: url, storeDirectory: BridgeClient.defaultStoreDirectory())
        return true
    }

    /// Launches `executable arguments... --store <storeDirectory>` and runs the
    /// contract's restart reconciliation before opening the active document.
    /// Returns once the document is open (or the attach failed).
    func attachBridge(executable: URL, arguments: [String] = [], storeDirectory: URL, enableGrok: Bool = false) {
        Task { await attachBridgeAndWait(executable: executable, arguments: arguments, storeDirectory: storeDirectory, enableGrok: enableGrok) }
    }

    @discardableResult
    func attachBridgeAndWait(executable: URL, arguments: [String] = [], storeDirectory: URL, enableGrok: Bool = false) async -> Bool {
        let session: BridgeSession
        do {
            session = try BridgeSession(executable: executable, arguments: arguments, storeDirectory: storeDirectory,
                                        enableGrok: enableGrok, projectId: projectId)
        } catch {
            bridgeStatus = "bridge launch failed: \(error.localizedDescription)"
            return false
        }
        setBridge(session)
        session.onChange = { [weak self, weak session] in
            guard let self, let session else { return }
            self.bridgeStatus = session.status
            self.bridgeCaptures = session.captures
            self.bridgeDestination = session.destination
        }
        // Contract step 5: consult capture_status and the ledger before anything else.
        let (actions, minimumRevision) = await session.reconcile(path: activePath, currentText: activeText, currentRevision: editorRevision)
        advanceEditorRevision(atLeast: minimumRevision)
        for action in actions {
            switch action {
            case .confirmed(let editId): captureNote = "Ledger: edit \(editId) confirmed by the bridge."
            case .replayedReceipt(let editId, let rev): captureNote = "Ledger: replayed missing receipt for \(editId) (revision \(rev))."
            case .reoffered(let captureId, let proposal):
                captureNote = "Ledger: prepared edit for \(captureId) re-offered for review (buffer unchanged)."
                enqueue(proposal)
            case .reselectionRequired(let editId, let why), .abandoned(let editId, let why):
                captureNote = "Ledger: \(editId) — \(why)"
            }
        }
        do {
            try await session.open(path: activePath, revision: editorRevision, text: activeText)
        } catch {
            captureNote = "Bridge could not open \(activePath): \((error as? BridgeClient.Failure)?.text ?? "\(error)")"
            return false
        }
        return true
    }

    func detachBridge() { setBridge(nil) }

    // MARK: document synchronization

    func bridgeTextChanged(path: String, old: String, new: String, base: Int, revision: Int) {
        guard let bridge, bridge.running else { return }
        if let expected = bridge.expectedApplication, expected.edit.path == path {
            if new == expected.afterText {
                // Contract step 4: the reviewed edit landed; confirm it, do not send document_edit.
                appliedCaptureIDs.insert(expected.edit.captureId)
                bridge.applicationApplied(newRevision: revision, afterText: new) // also drops the pinned destination
                captureNote = "Inserted \(expected.edit.captureId) via bridge edit \(expected.edit.editId) (undo with ⌘Z); pin a new insertion point for the next capture."
                return
            }
            bridge.abandonExpectedApplication("buffer changed differently than the prepared edit")
            captureNote = "Prepared edit \(expected.edit.editId) was not applied as prepared; pin a new destination and submit a new capture."
        }
        bridge.edited(path: path, oldText: old, newText: new, base: base, revision: revision)
    }

    func bridgeDocumentReplaced() {
        guard let bridge, bridge.running else { return }
        bridge.abandonExpectedApplication("document replaced")
        bridge.invalidateDestination()
        Task { try? await bridge.open(path: activePath, revision: editorRevision, text: activeText) }
    }

    // MARK: destinations

    /// Mirrors a local pin to the bridge: the caret (or selection) as a byte
    /// range at the current revision. The local anchor stays for offline use.
    func bridgePin(_ anchor: InsertionAnchor) {
        guard let bridge, bridge.running else { return }
        let text = activeText
        let end = text.utf8ByteRange(of: NSRange(location: caretUTF16, length: caretLengthUTF16))?.end ?? anchor.byteOffset
        Task { await bridgePinAndWait(destinationId: anchor.id, path: anchor.path, revision: anchor.revision,
                                      startByte: anchor.byteOffset, endByte: max(end, anchor.byteOffset)) }
    }

    @discardableResult
    func bridgePinAndWait(destinationId: String, path: String, revision: Int, startByte: Int, endByte: Int) async -> TransferV1.Anchor? {
        guard let bridge, bridge.running else { return nil }
        do {
            let a = try await bridge.pin(destinationId: destinationId, path: path, revision: revision, startByte: startByte, endByte: endByte)
            captureNote = "Pinned \(a.destinationId) on the bridge at \(a.path) bytes \(a.startByte)..<\(a.endByte) (revision \(a.pinnedRevision))."
            return a
        } catch {
            captureNote = "Bridge pin failed: \((error as? BridgeClient.Failure)?.text ?? "\(error)")"
            return nil
        }
    }

    // MARK: captures

    /// `Edit > Submit Sample Capture…`: a PNG/JPEG file becomes one
    /// `capture_submit` bound to the pinned destination.
    func submitSampleCapturePanel() {
        guard bridgeAttached else { captureNote = "Attach the capture bridge first (Edit > Attach Capture Bridge)."; return }
        guard bridgeDestination != nil else { captureNote = "Pin an insertion point first (⌘⇧P) so the capture has a destination."; return }
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [.png, .jpeg]
        panel.message = "Choose a PNG or JPEG capture to submit through the bridge"
        guard panel.runModal() == .OK, let url = panel.url else { return }
        Task { await submitCapture(imageAt: url) }
    }

    @discardableResult
    func submitCapture(imageAt url: URL, captureId: String? = nil, instructions: String = "Transcribe the selected handwriting to LaTeX; preserve notation.") async -> TransferV1.CaptureReceived? {
        guard let data = try? Data(contentsOf: url) else { captureNote = "Could not read \(url.lastPathComponent)."; return nil }
        let mime = url.pathExtension.lowercased() == "png" ? "image/png" : "image/jpeg"
        return await submitCapture(image: .init(mimeType: mime, dataBase64: data.base64EncodedString()),
                                   captureId: captureId, instructions: instructions)
    }

    /// Builds `capture_submit` from the pinned destination (`base_revision` is
    /// the anchor's pinned revision, never a guess) and shows the receipt.
    @discardableResult
    func submitCapture(image: RuntimeV1.CaptureImage, captureId: String? = nil, instructions: String) async -> TransferV1.CaptureReceived? {
        guard let bridge, bridge.running else { captureNote = "No bridge attached."; return nil }
        guard let destination = bridgeDestination else { captureNote = "Pin an insertion point first (⌘⇧P)."; return nil }
        guard RuntimeV1.acceptedCaptureMimeTypes.contains(image.mimeType) else { captureNote = "Only PNG and JPEG captures are accepted."; return nil }
        let id = captureId ?? "mac-capture-\(UUID().uuidString.lowercased())"
        let submit = RuntimeV1.CaptureSubmit(captureId: id, destinationId: destination.destinationId,
                                             baseRevision: destination.pinnedRevision, image: image, instructions: instructions)
        do {
            let received = try await bridge.submit(submit)
            captureNote = "Capture \(received.captureId) received (durable: \(received.durable)); use Edit > Convert Capture."
            return received
        } catch {
            captureNote = "Capture submit failed: \((error as? BridgeClient.Failure)?.text ?? "\(error)")"
            return nil
        }
    }

    /// `Edit > Convert Capture`: `capture_convert` for the latest received
    /// capture; a proposal is queued in the review sheet. Provider errors
    /// (`provider_disabled`, `provider_auth_missing`) are shown as plain text.
    func convertLatestCapture() {
        guard let c = latestConvertibleCapture else { captureNote = "No received capture to convert."; return }
        Task { await convertCapture(captureId: c.captureId) }
    }

    @discardableResult
    func convertCapture(captureId: String, supportedFeatures: [String] = []) async -> RuntimeV1.CaptureProposal? {
        guard let bridge, bridge.running else { captureNote = "No bridge attached."; return nil }
        do {
            let proposal = try await bridge.convert(captureId: captureId, supportedFeatures: supportedFeatures)
            enqueue(proposal)
            return proposal
        } catch {
            captureNote = "Conversion unavailable: \((error as? BridgeClient.Failure)?.text ?? "\(error)")"
            return nil
        }
    }

    /// Approval for a bridge capture: `capture_prepare_insert` at the current
    /// editor revision, full verification of the returned edit, one undoable
    /// edit through `pendingEdit`, ledger entry, then `capture_applied` once the
    /// editor reports the change. The reviewer's LaTeX must equal the journaled
    /// proposal: the contract has no field to send edited text, so an edited
    /// proposal is refused rather than silently replaced.
    @discardableResult
    func approveBridgeProposal(_ proposal: RuntimeV1.CaptureProposal, latex: String) async -> ApproveOutcome {
        guard let bridge, bridge.running else { captureNote = "No bridge attached."; return .refused("no bridge") }
        guard !appliedCaptureIDs.contains(proposal.captureId) else {
            proposals.removeAll { $0.captureId == proposal.captureId }
            if reviewing?.captureId == proposal.captureId { reviewing = proposals.first }
            captureNote = "Capture \(proposal.captureId) already inserted."
            return .duplicate
        }
        guard latex.trimmingCharacters(in: .whitespacesAndNewlines) == proposal.latex.trimmingCharacters(in: .whitespacesAndNewlines) else {
            captureNote = "Edited LaTeX cannot be inserted through the bridge (transfer-v1 prepares only the journaled proposal); reject and resubmit instead."
            return .refused("edited LaTeX")
        }
        guard bridge.expectedApplication == nil else {
            captureNote = "Another prepared edit is still being applied."
            return .refused("edit in progress")
        }
        let edit: TransferV1.CaptureEdit
        do {
            edit = try await bridge.prepare(captureId: proposal.captureId, expectedRevision: editorRevision)
        } catch {
            let f = (error as? BridgeClient.Failure)
            captureNote = "Cannot insert \(proposal.captureId): \(f?.text ?? "\(error)")"
            if f?.code == "already_applied" {
                appliedCaptureIDs.insert(proposal.captureId)
                proposals.removeAll { $0.captureId == proposal.captureId }
                reviewing = proposals.first
                return .duplicate
            }
            if f?.code == "destination_reselection_required" || f?.code == "revision_conflict" {
                bridge.invalidateDestination()
                return .needsReselection(f?.text ?? "reselection required")
            }
            return .refused(f?.text ?? "\(error)")
        }
        let text = activeText
        switch BridgeSession.verify(edit, projectId: projectId, path: activePath, revision: editorRevision, text: text) {
        case .refused(let why):
            captureNote = "Refused bridge edit \(edit.editId): \(why). Pin a new destination and submit a new capture."
            bridge.invalidateDestination()
            return .needsReselection(why)
        case .ok(let afterText):
            guard let ns = text.nsRange(utf8Bytes: .init(path: edit.path, startByte: edit.startByte, endByte: edit.endByte)) else {
                return .needsReselection("byte range is not representable in UTF-16")
            }
            do {
                guard try bridge.recordPrepared(edit, beforeText: text, afterText: afterText) else {
                    captureNote = "Edit \(edit.editId) is already in the ledger as applied; not applying again."
                    proposals.removeAll { $0.captureId == proposal.captureId }
                    reviewing = proposals.first
                    return .duplicate
                }
            } catch {
                captureNote = "Ledger write failed; not applying: \(error.localizedDescription)"
                return .refused("ledger")
            }
            activePath = edit.path
            pendingEdit = .init(path: edit.path, nsRange: ns, text: edit.replacement, token: (pendingEdit?.token ?? 0) + 1)
            proposals.removeAll { $0.captureId == proposal.captureId }
            reviewing = proposals.first
            captureNote = "Applying bridge edit \(edit.editId) at bytes \(edit.startByte)..<\(edit.endByte)…"
            return .inserted(byteOffset: edit.startByte)
        }
    }

    func bridgeReject(captureId: String) {
        guard let bridge, bridge.running, bridge.capture(captureId) != nil else { return }
        Task { await bridgeRejectAndWait(captureId: captureId) }
    }

    @discardableResult
    func bridgeRejectAndWait(captureId: String) async -> Bool {
        guard let bridge, bridge.running else { return false }
        do { try await bridge.reject(captureId: captureId); return true }
        catch { captureNote = "Reject failed: \((error as? BridgeClient.Failure)?.text ?? "\(error)")"; return false }
    }
}
