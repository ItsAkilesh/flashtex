import XCTest
@testable import FlashTeXProtocol
@testable import FlashTeXMac

final class CompletionTests: XCTestCase {
    private func caret(after needle: String, in text: String) -> Int {
        let r = (text as NSString).range(of: needle)
        XCTAssertNotEqual(r.location, NSNotFound, "needle \(needle) missing")
        return NSMaxRange(r)
    }

    private func labels(_ s: [Completion.Suggestion]) -> [String] { s.map(\.label) }

    // MARK: triggers and prefix filtering

    func testBackslashTriggersCommandsAndPrefixFilters() {
        let text = "Hello \\se"
        let s = Completion.suggestions(in: text, caretUTF16: (text as NSString).length, result: nil)
        XCTAssertEqual(labels(s), ["\\section"])
        XCTAssertEqual(s.first?.kind, .command)
        XCTAssertEqual(s.first?.insertText, "\\section")
        XCTAssertEqual(s.first?.detail, "supported by this compiler")

        // A lone backslash lists every supported command (capped at 12).
        let all = Completion.suggestions(in: "x \\", caretUTF16: 3, result: nil)
        XCTAssertEqual(all.count, Completion.maxSuggestions)
        XCTAssertEqual(all.first?.label, "\\section")
        XCTAssertTrue(all.allSatisfy { $0.kind == .command })

        // `\\` itself is a supported command.
        let dbl = Completion.suggestions(in: "a\\\\", caretUTF16: 3, result: nil)
        XCTAssertEqual(labels(dbl), ["\\\\"])

        // Math commands carry the verification detail.
        let math = Completion.suggestions(in: "$\\al", caretUTF16: 4, result: nil)
        XCTAssertEqual(labels(math), ["\\alpha"])
        XCTAssertEqual(math.first?.detail, "math · verified in compiler at \(Completion.mathVerifiedAt)")
        XCTAssertEqual(Completion.mathCommands.count, 24)

        // No suggestions for a non-matching prefix or a caret with nothing before it.
        XCTAssertTrue(Completion.suggestions(in: "\\zzz", caretUTF16: 4, result: nil).isEmpty)
        XCTAssertTrue(Completion.suggestions(in: "abc ", caretUTF16: 4, result: nil).isEmpty)
    }

    func testWordsNeedTwoLettersAndAreFrequencyRanked() {
        let text = "theorem theory theorem thesis theorem theory th"
        let caret = (text as NSString).length
        XCTAssertTrue(Completion.suggestions(in: text, caretUTF16: caret - 1, result: nil).isEmpty, "one letter never triggers")
        let s = Completion.suggestions(in: text, caretUTF16: caret, result: nil)
        XCTAssertEqual(labels(s), ["theorem", "theory", "thesis"])
        XCTAssertEqual(s.first?.detail, "3× in this document")
        XCTAssertTrue(s.allSatisfy { $0.kind == .word })

        // Prefix filtering is ASCII case-insensitive; short words (<= 3 chars) never appear;
        // the word being typed is not counted as its own completion.
        let t2 = "The the them Then thesis thesis Th"
        let s2 = Completion.suggestions(in: t2, caretUTF16: (t2 as NSString).length, result: nil)
        XCTAssertEqual(labels(s2), ["thesis", "Then", "them"]) // "The"/"the" are too short
        let t3 = "thesis thesis"
        XCTAssertEqual(labels(Completion.suggestions(in: t3, caretUTF16: 13, result: nil)), ["thesis"]) // token itself not counted
        XCTAssertEqual(Completion.suggestions(in: t3, caretUTF16: 13, result: nil).first?.detail, "1× in this document")
    }

    func testUnclosedEnvironmentSuggestsEndFirst() {
        let text = "\\begin{document}\n\\begin{itemize}\n\\item a\n\\e"
        let s = Completion.suggestions(in: text, caretUTF16: (text as NSString).length, result: nil)
        XCTAssertEqual(Array(labels(s).prefix(3)), ["\\end{itemize}", "\\end{document}", "\\emph"])
        XCTAssertEqual(s[0].kind, .environment)
        XCTAssertEqual(s[0].detail, "closes \\begin{itemize} at byte 17")
        XCTAssertEqual(s[0].insertText, "\\end{itemize}")

        // Once itemize is closed only document remains open.
        let closed = text + "nd{itemize}\n\\en"
        let s2 = Completion.suggestions(in: closed, caretUTF16: (closed as NSString).length, result: nil)
        XCTAssertEqual(labels(s2), ["\\end{document}", "\\end"])

        // Inside `\end{` the open environments come first, then known/seen names.
        let inBrace = "\\begin{document}\\begin{itemize}\\end{"
        let env = Completion.suggestions(in: inBrace + "i", caretUTF16: (inBrace as NSString).length + 1, result: nil)
        XCTAssertEqual(labels(env), ["itemize"])
        XCTAssertEqual(env.first?.insertText, "itemize}")
        XCTAssertEqual(env.first?.kind, .environment)
        let beginCtx = "\\begin{itemize}\\end{itemize}\\begin{d"
        let b = Completion.suggestions(in: beginCtx, caretUTF16: (beginCtx as NSString).length, result: nil)
        XCTAssertEqual(labels(b), ["document"])
        XCTAssertEqual(b.first?.detail, "supported by this compiler")
    }

    func testUnsupportedDocumentCommandsAreMarked() {
        let text = "\\documentclass{article}\n\\usepackage{amsmath}\n\\newpage\n\\ne"
        let diag = RuntimeV1.Diagnostic(severity: .error,
                                        message: "\\newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented",
                                        source: nil, recovery: nil)
        let result = RuntimeV1.CompileResult(projectId: "p", revision: 1, status: .recovered, pages: [], diagnostics: [diag], pdfPath: nil)
        let s = Completion.suggestions(in: text, caretUTF16: (text as NSString).length, result: result)
        XCTAssertEqual(labels(s), ["\\neq", "\\newpage"])
        XCTAssertEqual(s[0].detail, "math · verified in compiler at \(Completion.mathVerifiedAt)")
        XCTAssertTrue(s[1].detail.hasPrefix("not supported by this compiler version"), s[1].detail)
        XCTAssertTrue(s[1].detail.contains(diag.message), s[1].detail)

        // Without a diagnostic naming it the mark is still there, without a message.
        let s2 = Completion.suggestions(in: text, caretUTF16: (text as NSString).length, result: nil)
        XCTAssertEqual(s2[1].detail, "not supported by this compiler version")

        // The command being typed is not offered as its own completion.
        let s3 = Completion.suggestions(in: "\\usepack", caretUTF16: 8, result: nil)
        XCTAssertTrue(s3.isEmpty, "\(labels(s3))")

        // A custom supported list replaces the default.
        let s4 = Completion.suggestions(in: text, caretUTF16: (text as NSString).length, result: nil, supported: ["newpage"])
        XCTAssertEqual(labels(s4), ["\\newpage"])
        XCTAssertEqual(s4[0].detail, "supported by this compiler")
    }

    func testReferencesSuggestLabels() {
        let text = "\\label{eq:main}\\label{fig:naïve}\nsee \\ref{fi"
        let s = Completion.suggestions(in: text, caretUTF16: (text as NSString).length, result: nil)
        XCTAssertEqual(labels(s), ["fig:naïve"])
        XCTAssertEqual(s.first?.kind, .reference)
        XCTAssertEqual(s.first?.insertText, "fig:naïve}")
        let e = "\\label{eq:main} \\eqref{eq"
        XCTAssertEqual(labels(Completion.suggestions(in: e, caretUTF16: (e as NSString).length, result: nil)), ["eq:main"])
    }

    // MARK: non-ASCII and invalid carets

    func testNonASCIIWordsAndCaretsAreSafe() {
        let text = "Résumé naïve naïveté 😀 naïve na"
        let ns = text as NSString
        XCTAssertNotEqual(text.utf8.count, ns.length)
        let s = Completion.suggestions(in: text, caretUTF16: ns.length, result: nil)
        XCTAssertEqual(labels(s), ["naïve", "naïveté"])
        XCTAssertEqual(s.first?.detail, "2× in this document")

        // Typing a non-ASCII prefix.
        let t2 = "Résumé Réunion Ré"
        XCTAssertEqual(labels(Completion.suggestions(in: t2, caretUTF16: (t2 as NSString).length, result: nil)), ["Résumé", "Réunion"])

        // A caret inside the emoji's surrogate pair, past the end, or negative yields nothing.
        let emoji = ns.range(of: "😀")
        XCTAssertEqual(emoji.length, 2)
        XCTAssertTrue(Completion.suggestions(in: text, caretUTF16: emoji.location + 1, result: nil).isEmpty)
        XCTAssertTrue(Completion.suggestions(in: text, caretUTF16: ns.length + 5, result: nil).isEmpty)
        XCTAssertTrue(Completion.suggestions(in: text, caretUTF16: -1, result: nil).isEmpty)
        XCTAssertTrue(Completion.suggestions(in: "", caretUTF16: 0, result: nil).isEmpty)
        // Non-letter scalars adjacent to a word never form a token.
        XCTAssertTrue(Completion.suggestions(in: "ab—", caretUTF16: 3, result: nil).isEmpty)
        XCTAssertNil(Completion.token(in: "ab—", caretUTF16: 3))

        // Completion range: includes the backslash; empty at a caret with no token;
        // clamped when out of range; UTF-16 exact after a multi-byte prefix.
        let cmd = "naïve \\sec"
        XCTAssertEqual(Completion.completionRange(in: cmd, caretUTF16: (cmd as NSString).length), NSRange(location: 6, length: 4))
        XCTAssertEqual(Completion.completionRange(in: "abc ", caretUTF16: 4), NSRange(location: 4, length: 0))
        XCTAssertEqual(Completion.completionRange(in: "abc", caretUTF16: 99), NSRange(location: 0, length: 3))
        XCTAssertEqual(Completion.completionRange(in: text, caretUTF16: emoji.location + 1), NSRange(location: emoji.location + 1, length: 0))
    }

    // MARK: NSTextView integration

    @MainActor
    func testCompletingTextViewUsesSubclassAndBackslashRange() throws {
        let scroll = CompletingTextView.scrollable()
        let tv = try XCTUnwrap(scroll.documentView as? CompletingTextView)
        tv.string = "\\begin{document}\nnaïve \\se"
        let end = (tv.string as NSString).length
        tv.setSelectedRange(NSRange(location: end, length: 0))
        XCTAssertEqual(tv.rangeForUserCompletion, NSRange(location: end - 3, length: 3))
        var index = -1
        let items = tv.completions(forPartialWordRange: tv.rangeForUserCompletion, indexOfSelectedItem: &index)
        XCTAssertEqual(items, ["\\section"])
        XCTAssertEqual(index, 0)
        // `\e` offers the unclosed environment first.
        tv.string = "\\begin{document}\n\\e"
        tv.setSelectedRange(NSRange(location: 19, length: 0))
        XCTAssertEqual(tv.completions(forPartialWordRange: tv.rangeForUserCompletion, indexOfSelectedItem: &index)?.first, "\\end{document}")
        // No token: nil (AppKit shows nothing rather than an empty popup).
        tv.string = "abc "
        tv.setSelectedRange(NSRange(location: 4, length: 0))
        XCTAssertNil(tv.completions(forPartialWordRange: tv.rangeForUserCompletion, indexOfSelectedItem: &index))
        // Stale or absurd ranges (text shrank after the range was computed) are refused, not trapped.
        for bad in [NSRange(location: 2, length: 10), NSRange(location: NSNotFound, length: 0),
                    NSRange(location: 99, length: 0), NSRange(location: -1, length: 1)] {
            XCTAssertNil(tv.completions(forPartialWordRange: bad, indexOfSelectedItem: &index), "\(bad)")
            tv.insertCompletion("\\section", forPartialWordRange: bad, movement: NSReturnTextMovement, isFinal: true)
            XCTAssertEqual(tv.string, "abc ", "stale range \(bad) must not splice text")
        }
        // A valid final insertion replaces exactly the partial token.
        tv.string = "x \\se"
        tv.setSelectedRange(NSRange(location: 5, length: 0))
        tv.insertCompletion("\\section", forPartialWordRange: tv.rangeForUserCompletion, movement: NSReturnTextMovement, isFinal: true)
        XCTAssertEqual(tv.string, "x \\section")
    }

    // MARK: fault tolerance

    func testNeverThrowsOnMalformedInput() {
        let junk = ["\\", "\\\\\\", "\\begin{", "\\begin{}\\end{", "{{{}}}}", "\\ref{", "\\end{x}\\end{x}", "a\u{FFFD}b", String(repeating: "\\", count: 50)]
        for text in junk {
            for caret in -1...((text as NSString).length + 1) {
                _ = Completion.suggestions(in: text, caretUTF16: caret, result: nil)
                _ = Completion.completionRange(in: text, caretUTF16: caret)
            }
        }
        let empty = RuntimeV1.CompileResult(projectId: "p", revision: 1, status: .failed, pages: [], diagnostics: [
            .init(severity: .error, message: "\\", source: nil, recovery: nil),
            .init(severity: .error, message: "no command here", source: nil, recovery: nil),
        ], pdfPath: nil)
        XCTAssertFalse(Completion.suggestions(in: "\\x \\", caretUTF16: 4, result: empty).isEmpty)
    }

    // MARK: performance

    /// Completion on a ~1 MB buffer. The measured average is printed so it can
    /// be reported; the assertion is generous so CI noise does not fail the suite.
    func testCompletionOnOneMegabyteBufferIsFast() {
        var text = ""
        text.reserveCapacity(1_100_000)
        let para = "\\section{Introduction} A naïve approach fails because theorem \\textbf{proofs} need \\emph{careful} statements. Résumé of the steps follows here with theory and thesis words.\n\n"
        while text.utf8.count < 1_000_000 { text += para }
        text += "\\begin{itemize} the"
        let caret = (text as NSString).length
        XCTAssertGreaterThan(text.utf8.count, 1_000_000)

        let iterations = 20
        var wordMs = 0.0, cmdMs = 0.0
        for _ in 0..<iterations {
            let t0 = DispatchTime.now().uptimeNanoseconds
            let words = Completion.suggestions(in: text, caretUTF16: caret, result: nil)
            let t1 = DispatchTime.now().uptimeNanoseconds
            let cmds = Completion.suggestions(in: text + " \\e", caretUTF16: caret + 3, result: nil)
            let t2 = DispatchTime.now().uptimeNanoseconds
            wordMs += Double(t1 - t0) / 1e6
            cmdMs += Double(t2 - t1) / 1e6
            XCTAssertEqual(words.first?.label, "theorem")
            XCTAssertEqual(cmds.first?.label, "\\end{itemize}")
        }
        wordMs /= Double(iterations); cmdMs /= Double(iterations)
        print("completion latency on \(text.utf8.count)-byte buffer: words \(String(format: "%.2f", wordMs)) ms, commands \(String(format: "%.2f", cmdMs)) ms (avg of \(iterations))")
        XCTAssertLessThan(wordMs, 20)
        XCTAssertLessThan(cmdMs, 20)
        measure {
            _ = Completion.suggestions(in: text, caretUTF16: caret, result: nil)
        }
    }
}
