// name: FakeHost.cs
// purpose: A scripted in-process stand-in for flashtex-host: listens on a real
//   AF_UNIX socket in a short temp path, accepts one connection and runs the
//   test's script against it. The script gets the connection as a stream.
// author: Claude Sonnet 5.5 (paired with Akilesh S)
// date: 2026-10-28

using System.Net.Sockets;
using System.Text;

namespace FlashTeX.EngineHost.Tests;

internal sealed class FakeHost : IAsyncDisposable
{
    private const int SocketNameChars = 10;

    private readonly Socket _listener;
    private readonly Task _script;

    private FakeHost(Socket listener, string socketPath, Task script)
    {
        _listener = listener;
        SocketPath = socketPath;
        _script = script;
    }

    public string SocketPath { get; }

    public static FakeHost Start(Func<NetworkStream, Task> script)
    {
        var path = Path.Combine(Path.GetTempPath(), $"fx{Guid.NewGuid():N}"[..SocketNameChars] + ".sock");
        var listener = new Socket(AddressFamily.Unix, SocketType.Stream, ProtocolType.Unspecified);
        listener.Bind(new UnixDomainSocketEndPoint(path));
        listener.Listen(1);
        return new FakeHost(listener, path, RunAsync(listener, script));
    }

    public static async Task AnswerHelloAsync(NetworkStream stream, string helloJson)
    {
        await ReadFrameAsync(stream);
        await SendAsync(stream, Dl3Kind.HostHello, Encoding.UTF8.GetBytes(helloJson));
    }

    public static async Task<Dl3Frame> ReadFrameAsync(NetworkStream stream) =>
        await Dl3Framing.ReadAsync(stream, CancellationToken.None)
        ?? throw new InvalidOperationException("client closed the connection");

    public static Task SendAsync(NetworkStream stream, byte kind, byte[] body) =>
        Dl3Framing.WriteAsync(stream, kind, body, CancellationToken.None);

    public async ValueTask DisposeAsync()
    {
        _listener.Dispose();
        try
        {
            await _script.WaitAsync(TimeSpan.FromSeconds(5));
        }
        catch (Exception ex) when (ex is IOException or SocketException or TimeoutException or InvalidOperationException)
        {
            // The script's own connection ended when the client disposed; that is expected.
        }

        File.Delete(SocketPath);
    }

    private static async Task RunAsync(Socket listener, Func<NetworkStream, Task> script)
    {
        using var connection = await listener.AcceptAsync();
        await using var stream = new NetworkStream(connection, ownsSocket: false);
        await script(stream);
    }
}
