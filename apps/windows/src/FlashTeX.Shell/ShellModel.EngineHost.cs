// name: ShellModel.EngineHost.cs
// purpose: The new-engine compile path of ShellModel. With an IEngineHostSession
//   attached (flashtex-host, display-list-v3), a compile sends the host a COMPILE
//   for the project folder on disk, with every open document's text as `buffers`
//   so unsaved edits compile without being saved, then applies the DONE: status,
//   diagnostics and the path of the PDF the preview draws. This replaces the
//   WorkerClient/runtime-v1 path, which upstream is retiring.
// author: Claude Sonnet 5.5 (paired with Akilesh S)
// date: 2026-10-28

using FlashTeX.EngineHost;
using FlashTeX.Protocol.RuntimeV1;

namespace FlashTeX.Shell;

public partial class ShellModel
{
    private const string EngineExternalTools = "auto";
    private const string EngineBackendLabel = "engine v3 (PDF preview)";

    private IEngineHostSession? _engine;
    private string? _engineScratchRoot;
    private string? _engineOutputDir;

    /// <summary>The PDF the last engine compile wrote, which the preview draws; null before the first or after a failure.</summary>
    public string? LastPdfPath { get; private set; }

    /// <summary>The last engine compile's DONE and diagnostics (with file and line), null for the old worker path.</summary>
    public FlashTeX.EngineHost.CompileResult? LastEngineResult { get; private set; }

    /// <summary>Whether an engine host session is attached.</summary>
    public bool HasEngineAttached => _engine is not null;

    /// <summary>
    /// Attaches the engine host. <paramref name="scratchRoot"/> is the project folder used when
    /// no <c>ProjectRoot</c> is set (the seeded in-memory document); <paramref name="outputDir"/>
    /// receives the PDF, log and auxiliary files.
    /// </summary>
    public void AttachEngineHost(IEngineHostSession session, string scratchRoot, string outputDir)
    {
        _engine = session;
        _engineScratchRoot = scratchRoot;
        _engineOutputDir = outputDir;
        WorkerStatus = "attached: flashtex-host";
        OnPropertyChanged(nameof(HasEngineAttached));
        OnPropertyChanged(nameof(HasWorkerAttached));
    }

    /// <summary>Detaches the engine host session. The caller owns and disposes the session itself.</summary>
    public void DetachEngineHost()
    {
        _engine = null;
        WorkerStatus = "no worker attached";
        OnPropertyChanged(nameof(HasEngineAttached));
        OnPropertyChanged(nameof(HasWorkerAttached));
    }

    /// <summary>
    /// A document path as the host wants it: relative to <paramref name="root"/>, `/`-separated,
    /// no `..` (spec section 1). Null when the document lies outside the root.
    /// </summary>
    internal static string? ToHostRelativePath(string path, string root)
    {
        var relative = Path.IsPathRooted(path) ? Path.GetRelativePath(root, path) : path;
        var normalized = relative.Replace('\\', '/');
        var isEscaping = normalized == ".." || normalized.StartsWith("../", StringComparison.Ordinal)
            || Path.IsPathRooted(normalized);
        return isEscaping ? null : normalized;
    }

    internal FlashTeX.EngineHost.CompileRequest? BuildEngineRequest(long id)
    {
        var root = ProjectRoot ?? _engineScratchRoot;
        var entry = ActiveDocumentPath ?? Documents[0].Path;
        if (root is null || ToHostRelativePath(entry, root) is not { } main)
        {
            return null;
        }

        var buffers = new List<BufferFile>();
        foreach (var document in Documents)
        {
            if (ToHostRelativePath(document.Path, root) is { } relative)
            {
                buffers.Add(new BufferFile(relative, document.Text));
            }
        }

        return new FlashTeX.EngineHost.CompileRequest
        {
            Id = id,
            Root = root,
            Main = main,
            OutputDir = _engineOutputDir,
            ExternalTools = EngineExternalTools,
            Incremental = true,
            Buffers = buffers,
        };
    }

    private async Task RunEngineCompileAsync(IEngineHostSession engine, CancellationToken cancellationToken)
    {
        var request = BuildEngineRequest(++_compileRevision);
        if (request is null)
        {
            WorkerStatus = "compile skipped: the active document is outside the project folder";
            return;
        }

        try
        {
            var result = await engine.CompileAsync(request, onFrame: null, cancellationToken).ConfigureAwait(true);
            ApplyEngineResult(result);
        }
        catch (Exception ex) when (ex is EngineHostException or Dl3ProtocolException or IOException)
        {
            LastPdfPath = null;
            WorkerStatus = "compile failed: " + ex.Message;
        }
    }

    private void ApplyEngineResult(FlashTeX.EngineHost.CompileResult result)
    {
        var done = result.Done;
        Diagnostics = result.Diagnostics.Select(ToProtocolDiagnostic).ToList();
        WordCount = CountWords(ActiveDocument?.Text ?? string.Empty);
        HasCompileResult = true;
        LastEngineResult = result;
        LastCompileResult = null;
        LastDisplayList = null;
        DisplayListRefusal = EngineBackendLabel;
        LastPdfPath = done.Pdf;
        WorkerStatus = $"revision {done.Id}: {done.Status}, {result.Diagnostics.Count} diagnostics · {done.Pages} page(s)"
            + (done.Mode is { } mode ? $" · {mode}" : string.Empty);
        RenderGeneration++;
    }

    private static Diagnostic ToProtocolDiagnostic(HostDiagnostic diagnostic)
    {
        var severity = diagnostic.Severity == "error" ? Severity.error : Severity.warning;
        var location = diagnostic.File is null
            ? string.Empty
            : diagnostic.Line is { } line ? $"{diagnostic.File}:{line}: " : $"{diagnostic.File}: ";
        return new Diagnostic(severity, location + diagnostic.Message, Source: null, Recovery: null);
    }
}
