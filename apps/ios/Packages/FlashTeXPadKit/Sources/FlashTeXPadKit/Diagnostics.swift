import Foundation
import FlashTeXProtocol

/// Diagnostics as the iPad lists them: runtime-v1 `compile_result.diagnostics`
/// (docs/contracts/runtime-v1.md) bound to the revision they were produced
/// for. Nearby-v1 does not carry compile results to a companion, so the only
/// producers on the iPad are a bundled runtime-v1 fixture or a
/// `compile_result` JSON line the user opens; the source is always shown.
public struct DiagnosticItem: Identifiable, Equatable {
    public var id: Int
    public var severity: RuntimeV1.Severity
    public var message: String
    public var recovery: String?
    public var range: RuntimeV1.SourceRange?
    /// Excerpt of the bound source at `range`, when the offsets are valid for it.
    public var excerpt: String?
}

public enum DiagnosticsSource: Equatable {
    case none
    case fixture(name: String, revision: Int)
    case file(name: String, revision: Int)
    public var label: String {
        switch self {
        case .none: return "no compile_result loaded"
        case .fixture(let n, let r): return "runtime-v1 fixture \(n) (revision \(r))"
        case .file(let n, let r): return "compile_result \(n) (revision \(r))"
        }
    }
}

public enum DiagnosticsModel {
    /// Decodes a runtime-v1 `compile_result` envelope (JSON object or one JSON line).
    public static func decode(_ data: Data) throws -> RuntimeV1.CompileResult {
        try RuntimeV1.decodeCompileResult(data).payload
    }

    public static func items(from result: RuntimeV1.CompileResult, boundTo document: PadDocument?) -> [DiagnosticItem] {
        result.diagnostics.enumerated().map { i, d in
            var excerpt: String?
            if let r = d.source, let doc = document, r.path == doc.path {
                excerpt = doc.slice(startByte: r.startByte, endByte: r.endByte)
            }
            return DiagnosticItem(id: i, severity: d.severity, message: d.message, recovery: d.recovery, range: d.source, excerpt: excerpt)
        }
    }
}
