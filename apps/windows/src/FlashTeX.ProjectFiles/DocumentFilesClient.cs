// name: DocumentFilesClient.cs
// purpose: Client for the flashtex-project-files JSON Lines helper
//   (protocol project-files-v1, crates/project-files/src/bin/flashtex-project-files.rs):
//   rooted, symlink-refusing read/status/save/remove/rename on one project
//   directory. Ported
//   from the responsibility of apps/mac/Sources/FlashTeXMac/DocumentFilesClient.swift,
//   built on the shared FlashTeX.Ipc.HelperProcess transport (the same pattern
//   as FlashTeX.Ipc.WorkerClient/BridgeClient), following the wire shape's own
//   framing: requests are `{id, operation, ...fields}`; replies are
//   `{id, payload}` or `{id, error: {code, message}}` (no envelope `type`).
// author: Claude Sonnet 5
// date: 2026-09-14

using System.Text.Json;
using System.Threading;
using FlashTeX.Ipc;
using FlashTeX.Protocol;
using FlashTeX.Protocol.ProjectFilesV1;

namespace FlashTeX.ProjectFiles;

/// <summary>Thrown when the helper replies with an <c>error {code, message}</c> envelope.</summary>
public sealed class ProjectFilesErrorException(string requestId, string code, string message)
    : Exception($"flashtex-project-files returned error '{code}' for request '{requestId}': {message}")
{
    public string RequestId { get; } = requestId;
    public string Code { get; } = code;
}

/// <summary>
/// One running <c>flashtex-project-files --root &lt;dir&gt;</c> child process, bound
/// to one project directory. Every request is a single round trip correlated
/// by an internally generated <c>id</c>; the helper answers in request order,
/// so one caller that gives up on a reply still blocks every later one on this
/// same process (mirroring the Swift client's own documented caveat) — a
/// caller that cannot tolerate that should start a fresh client.
/// </summary>
public sealed class DocumentFilesClient : IAsyncDisposable
{
    private readonly HelperProcess _process;
    private long _nextRequestId;

    private DocumentFilesClient(HelperProcess process)
    {
        _process = process;
    }

    /// <summary>Launches <paramref name="executablePath"/> (the <c>flashtex-project-files</c> binary) against <paramref name="rootDirectory"/>.</summary>
    public static DocumentFilesClient Start(
        string executablePath,
        string rootDirectory,
        Action<string> onStderrLine,
        string? workingDirectory = null)
    {
        var process = HelperProcess.Start(
            new HelperProcessOptions
            {
                ExecutablePath = executablePath,
                Arguments = ["--root", rootDirectory],
                WorkingDirectory = workingDirectory,
                MaxLineBytes = ProjectFilesV1Protocol.MaxLineBytes,
            },
            onStderrLine);
        return new DocumentFilesClient(process);
    }

    /// <summary>Confirms the helper is alive and reports its bound root and process id.</summary>
    public Task<PingPayload> PingAsync(CancellationToken cancellationToken = default) =>
        SendAsync<PingRequest, PingPayload>(id => new PingRequest(id), cancellationToken);

    /// <summary>Reads a rooted, project-relative file. A missing file is <see cref="ReadPayload.Exists"/> <c>false</c>, not an error.</summary>
    public Task<ReadPayload> ReadAsync(string path, CancellationToken cancellationToken = default) =>
        SendAsync<ReadRequest, ReadPayload>(id => new ReadRequest(id, path), cancellationToken);

    /// <summary>
    /// Reports a rooted file's state relative to <paramref name="expectedSha256"/>
    /// (<c>null</c> means the caller expects no file yet).
    /// </summary>
    public Task<StatusPayload> StatusAsync(string path, string? expectedSha256, CancellationToken cancellationToken = default) =>
        SendAsync<StatusRequest, StatusPayload>(id => new StatusRequest(id, path, expectedSha256), cancellationToken);

    /// <summary>
    /// Saves <paramref name="text"/> to a rooted, project-relative file with
    /// optimistic-concurrency semantics governed by <paramref name="expected"/>.
    /// A conflict is returned, never thrown: nothing was written and the
    /// caller's buffer is unaffected.
    /// </summary>
    public async Task<SaveOutcome> SaveAsync(
        string path,
        string text,
        Expected expected,
        bool force = false,
        CancellationToken cancellationToken = default)
    {
        SaveOutcomeWire wire = await SendAsync<SaveRequest, SaveOutcomeWire>(
            id => new SaveRequest(id, path, text, expected.Wire, force), cancellationToken).ConfigureAwait(false);
        return wire.Outcome switch
        {
            "saved" => new SaveOutcome.Saved(
                wire.Receipt ?? throw new HelperProtocolException("save outcome 'saved' carries no receipt")),
            "conflict" => new SaveOutcome.Conflict(
                wire.Conflict ?? throw new HelperProtocolException("save outcome 'conflict' carries no conflict details")),
            var other => throw new HelperProtocolException($"unknown save outcome '{other}'"),
        };
    }

    /// <summary>
    /// Unlinks a rooted, project-relative file. The returned
    /// <see cref="RemovePayload.Removed"/> is <c>false</c> when nothing was
    /// there — an absent path is not an error, so deleting a file another
    /// process already deleted converges instead of throwing. A symlink or
    /// non-regular entry is refused and left in place
    /// (<see cref="ProjectFilesErrorException"/> with code <c>refused</c>).
    /// </summary>
    public Task<RemovePayload> RemoveAsync(string path, CancellationToken cancellationToken = default) =>
        SendAsync<RemoveRequest, RemovePayload>(id => new RemoveRequest(id, path), cancellationToken);

    /// <summary>
    /// Renames a rooted, project-relative file within its own directory.
    ///
    /// The helper performs exactly one no-replace rename (crates/project-files
    /// <c>ProjectLock::rename</c>), so either <paramref name="to"/> names the
    /// file and <paramref name="from"/> is gone, or nothing changed: the two
    /// names are never both present and never both absent, even if the helper
    /// is killed mid-call. A conflict is returned, never thrown — the source
    /// is gone, or the new name is already taken and was *not* clobbered — and
    /// in both cases nothing on disk moved. <paramref name="from"/> and
    /// <paramref name="to"/> must share a parent directory; a cross-directory
    /// pair is an <c>invalid_request</c>
    /// <see cref="ProjectFilesErrorException"/>.
    /// </summary>
    public async Task<RenameOutcome> RenameAsync(
        string from,
        string to,
        CancellationToken cancellationToken = default)
    {
        RenameOutcomeWire wire = await SendAsync<RenameRequest, RenameOutcomeWire>(
            id => new RenameRequest(id, from, to), cancellationToken).ConfigureAwait(false);
        return wire.Outcome switch
        {
            "renamed" => new RenameOutcome.Renamed(
                wire.From ?? throw new HelperProtocolException("rename outcome 'renamed' carries no 'from'"),
                wire.To ?? throw new HelperProtocolException("rename outcome 'renamed' carries no 'to'")),
            "conflict" => new RenameOutcome.Conflict(
                wire.Conflict ?? throw new HelperProtocolException("rename outcome 'conflict' carries no conflict details")),
            var other => throw new HelperProtocolException($"unknown rename outcome '{other}'"),
        };
    }

    /// <summary>
    /// Recursively lists project files (<c>.tex</c>/<c>.bib</c>/<c>.sty</c>/
    /// <c>.cls</c>/<c>.bst</c>/<c>.clo</c>) under <paramref name="subdirectory"/>
    /// (the whole project root when <c>null</c>), as root-relative paths,
    /// sorted. The helper's rooted, symlink-refusing walk excludes symlinks
    /// and hidden directories (<c>.flashtex</c> included) rather than erroring
    /// on them — see <see cref="ListPayload"/>. <see cref="ListPayload.Truncated"/>
    /// is <c>true</c> if the helper's file-count cap was hit before the whole
    /// tree was walked.
    /// </summary>
    public Task<ListPayload> ListAsync(string? subdirectory = null, CancellationToken cancellationToken = default) =>
        SendAsync<ListRequest, ListPayload>(id => new ListRequest(id, subdirectory), cancellationToken);

    /// <summary>
    /// Sends one request and decodes its reply, throwing <see cref="ProjectFilesErrorException"/>
    /// for an <c>error</c> envelope or <see cref="HelperProtocolException"/> if the
    /// reply carries neither <c>payload</c> nor <c>error</c>.
    /// </summary>
    private async Task<TReply> SendAsync<TRequest, TReply>(
        Func<string, TRequest> makeRequest, CancellationToken cancellationToken)
    {
        string id = NextRequestId();
        TRequest request = makeRequest(id);
        JsonDocument reply = await _process.SendAsync(id, request, FlashTeXJson.Options, cancellationToken)
            .ConfigureAwait(false);

        if (reply.RootElement.TryGetProperty("error", out JsonElement errorElement))
        {
            ErrorPayload error = errorElement.Deserialize<ErrorPayload>(FlashTeXJson.Options)
                ?? throw new HelperProtocolException("project-files error payload decoded to null");
            throw new ProjectFilesErrorException(id, error.Code, error.Message);
        }

        if (!reply.RootElement.TryGetProperty("payload", out JsonElement payloadElement))
        {
            throw new HelperProtocolException($"project-files reply for '{id}' carries neither 'payload' nor 'error'");
        }

        return payloadElement.Deserialize<TReply>(FlashTeXJson.Options)
            ?? throw new HelperProtocolException($"project-files payload for '{id}' decoded to null");
    }

    private string NextRequestId() => $"pf-{Interlocked.Increment(ref _nextRequestId)}";

    public ValueTask DisposeAsync() => _process.DisposeAsync();
}
