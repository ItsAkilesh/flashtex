// name: BufferMaterializingSession.cs
// purpose: An IEngineHostSession decorator for projects that exist only in memory
//   (the seeded document, before anything is saved). flashtex-host checks that the
//   main file exists on disk before it applies a COMPILE's `buffers`, so this writes
//   every buffer under the request's root first, then delegates. Each path is
//   resolved and must stay inside the root, so a crafted document name cannot write
//   elsewhere.
// author: Claude Sonnet 5.5 (paired with Akilesh S)
// date: 2026-10-28
// modified: 2026-10-28 -- scoped to the scratch root so a real project is never written to.

namespace FlashTeX.EngineHost;

/// <summary>
/// Writes a request's buffers into its root, then runs the compile on the inner session. Only for
/// requests whose root is <paramref name="scratchRoot"/>: a real project's files are never written
/// here, because that would silently save the editor's unsaved text.
/// </summary>
public sealed class BufferMaterializingSession(IEngineHostSession inner, string scratchRoot) : IEngineHostSession
{
    private readonly string _scratchRoot = Path.GetFullPath(scratchRoot);

    /// <inheritdoc />
    public async Task<CompileResult> CompileAsync(CompileRequest request, Action<Dl3Frame>? onFrame, CancellationToken ct)
    {
        if (IsScratch(request))
        {
            await MaterializeAsync(request, ct).ConfigureAwait(false);
        }

        return await inner.CompileAsync(request, onFrame, ct).ConfigureAwait(false);
    }

    private bool IsScratch(CompileRequest request) =>
        string.Equals(
            Path.TrimEndingDirectorySeparator(Path.GetFullPath(request.Root)),
            Path.TrimEndingDirectorySeparator(_scratchRoot),
            StringComparison.OrdinalIgnoreCase);

    private static async Task MaterializeAsync(CompileRequest request, CancellationToken ct)
    {
        var root = Path.GetFullPath(request.Root);
        Directory.CreateDirectory(root);
        foreach (var buffer in request.Buffers ?? [])
        {
            var target = ResolveInside(root, buffer.Path);
            Directory.CreateDirectory(Path.GetDirectoryName(target)!); // non-null: target always has a parent under root
            await File.WriteAllTextAsync(target, buffer.Text, ct).ConfigureAwait(false);
        }
    }

    private static string ResolveInside(string root, string relativePath)
    {
        var target = Path.GetFullPath(Path.Combine(root, relativePath));
        var rootWithSeparator = root.EndsWith(Path.DirectorySeparatorChar) ? root : root + Path.DirectorySeparatorChar;
        if (!target.StartsWith(rootWithSeparator, StringComparison.OrdinalIgnoreCase))
        {
            throw new EngineHostException("path", $"'{relativePath}' is outside the project folder");
        }

        return target;
    }
}
