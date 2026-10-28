// name: EngineHostClient.cs
// purpose: Client for one flashtex-host over its AF_UNIX socket (display-list-v3,
//   docs/protocol/display-list-v3.md section 6): HELLO handshake, one COMPILE
//   at a time, collecting DIAGNOSTICs until DONE. Binary frames (PAGE, FONT,
//   IMAGE, FORM, SOURCES) are handed to an optional callback untouched, so a
//   renderer can decode them later; milestone 1 reads the DONE PDF instead.
// author: Claude Sonnet 5.5 (paired with Akilesh S)
// date: 2026-10-28
// modified: 2026-10-28 -- IEngineHostSession, so ShellModel can be tested without a socket.

using System.Net.Sockets;
using System.Text.Json;

namespace FlashTeX.EngineHost;

/// <summary>What one compile produced: the final status, its diagnostics and the frames seen.</summary>
public sealed record CompileResult(CompileDone Done, IReadOnlyList<HostDiagnostic> Diagnostics, int FrameCount);

/// <summary>What a shell needs from an engine host: run one compile. Lets tests stand in for the socket.</summary>
public interface IEngineHostSession
{
    Task<CompileResult> CompileAsync(CompileRequest request, Action<Dl3Frame>? onFrame, CancellationToken ct);
}

/// <summary>A connection to one flashtex-host; compiles run one at a time.</summary>
public sealed class EngineHostClient : IEngineHostSession, IAsyncDisposable
{
    private const string ProtocolName = "display-list-v3";
    private const string ClientName = "FlashTeX Windows";
    private const string UnknownErrorCode = "unknown";
    private static readonly int[] ClientVersion = [3, 2];
    private static readonly TimeSpan ByeTimeout = TimeSpan.FromSeconds(1);

    private readonly Socket _socket;
    private readonly NetworkStream _stream;
    private readonly SemaphoreSlim _compileGate = new(1, 1);

    private EngineHostClient(Socket socket, NetworkStream stream, HostHello hello)
    {
        _socket = socket;
        _stream = stream;
        Hello = hello;
    }

    /// <summary>What the host said about itself in its HELLO.</summary>
    public HostHello Hello { get; }

    /// <summary>Connects to the socket path and completes the HELLO exchange.</summary>
    public static async Task<EngineHostClient> ConnectAsync(string socketPath, CancellationToken ct)
    {
        var socket = new Socket(AddressFamily.Unix, SocketType.Stream, ProtocolType.Unspecified);
        NetworkStream? stream = null;
        try
        {
            await socket.ConnectAsync(new UnixDomainSocketEndPoint(socketPath), ct).ConfigureAwait(false);
            stream = new NetworkStream(socket, ownsSocket: false);
            var hello = await ExchangeHelloAsync(stream, ct).ConfigureAwait(false);
            return new EngineHostClient(socket, stream, hello);
        }
        catch
        {
            if (stream is not null)
            {
                await stream.DisposeAsync().ConfigureAwait(false);
            }

            socket.Dispose();
            throw;
        }
    }

    /// <summary>
    /// Runs one compile. Every frame other than DIAGNOSTIC, ERROR and this compile's DONE
    /// goes to onFrame; a DONE for another id (a superseded compile) is skipped.
    /// </summary>
    public async Task<CompileResult> CompileAsync(
        CompileRequest request, Action<Dl3Frame>? onFrame, CancellationToken ct)
    {
        await _compileGate.WaitAsync(ct).ConfigureAwait(false);
        try
        {
            await Dl3Framing.WriteAsync(_stream, Dl3Kind.Compile, Dl3Json.Serialize(request), ct).ConfigureAwait(false);
            return await ReadUntilDoneAsync(request.Id, onFrame, ct).ConfigureAwait(false);
        }
        finally
        {
            _compileGate.Release();
        }
    }

    /// <summary>Asks the host to stop compile id; its DONE then says cancelled.</summary>
    public Task CancelAsync(long id, CancellationToken ct) =>
        Dl3Framing.WriteAsync(_stream, Dl3Kind.Cancel, Dl3Json.Serialize(new { id }), ct);

    /// <inheritdoc />
    public async ValueTask DisposeAsync()
    {
        try
        {
            using var timeout = new CancellationTokenSource(ByeTimeout);
            await Dl3Framing.WriteAsync(_stream, Dl3Kind.Bye, "{}"u8.ToArray(), timeout.Token).ConfigureAwait(false);
        }
        catch (Exception ex) when (ex is IOException or SocketException or OperationCanceledException or ObjectDisposedException)
        {
            // The host is already gone or the socket broke: nothing left to tell it.
        }

        await _stream.DisposeAsync().ConfigureAwait(false);
        _socket.Dispose();
        _compileGate.Dispose();
    }

    private static async Task<HostHello> ExchangeHelloAsync(NetworkStream stream, CancellationToken ct)
    {
        var hello = new { protocol = ProtocolName, version = ClientVersion, client = ClientName };
        await Dl3Framing.WriteAsync(stream, Dl3Kind.ClientHello, Dl3Json.Serialize(hello), ct).ConfigureAwait(false);
        var frame = await Dl3Framing.ReadAsync(stream, ct).ConfigureAwait(false)
            ?? throw new Dl3ProtocolException("host closed the connection during HELLO");
        return frame.Kind switch
        {
            Dl3Kind.HostHello => Dl3Json.Deserialize<HostHello>(frame.Body),
            Dl3Kind.Error => throw ReadError(frame),
            _ => throw new Dl3ProtocolException($"unexpected frame 0x{frame.Kind:X2} during HELLO"),
        };
    }

    private async Task<CompileResult> ReadUntilDoneAsync(long id, Action<Dl3Frame>? onFrame, CancellationToken ct)
    {
        var diagnostics = new List<HostDiagnostic>();
        var frameCount = 0;
        while (true)
        {
            var frame = await Dl3Framing.ReadAsync(_stream, ct).ConfigureAwait(false)
                ?? throw new Dl3ProtocolException("host closed the connection mid-compile");
            frameCount++;
            if (frame.Kind == Dl3Kind.Error)
            {
                throw ReadError(frame);
            }

            if (frame.Kind == Dl3Kind.Diagnostic)
            {
                CollectDiagnostic(frame, id, diagnostics);
            }
            else if (frame.Kind == Dl3Kind.Done)
            {
                var done = Dl3Json.Deserialize<CompileDone>(frame.Body);
                if (done.Id == id)
                {
                    return new CompileResult(done, diagnostics, frameCount);
                }
            }
            else
            {
                onFrame?.Invoke(frame);
            }
        }
    }

    private static void CollectDiagnostic(Dl3Frame frame, long id, List<HostDiagnostic> into)
    {
        var diagnostic = Dl3Json.Deserialize<HostDiagnostic>(frame.Body);
        if (diagnostic.Id == id)
        {
            into.Add(diagnostic);
        }
    }

    private static EngineHostException ReadError(Dl3Frame frame)
    {
        using var document = JsonDocument.Parse(frame.Body);
        var root = document.RootElement;
        var code = root.TryGetProperty("code", out var c) ? c.GetString() ?? UnknownErrorCode : UnknownErrorCode;
        var message = root.TryGetProperty("message", out var m) ? m.GetString() ?? string.Empty : string.Empty;
        return new EngineHostException(code, message);
    }
}
