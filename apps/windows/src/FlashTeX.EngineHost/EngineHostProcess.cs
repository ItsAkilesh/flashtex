// name: EngineHostProcess.cs
// purpose: Starts and owns one flashtex-host child process (spec section 6.1):
//   launches it with --socket, passes the no-TeX-Live bundle configuration and
//   cache directories through the environment, surfaces its status lines
//   (bundle download progress, the warm-up line) and completes StartAsync when
//   it prints "listening on". The host is a separate process by design: it is
//   GPL, the app is MIT, and the process boundary is the licence boundary.
// author: Claude Sonnet 5.5 (paired with Akilesh S)
// date: 2026-10-28

using System.Diagnostics;

namespace FlashTeX.EngineHost;

/// <summary>How to launch the host.</summary>
public sealed record EngineHostOptions
{
    public required string HostExePath { get; init; }

    /// <summary>AF_UNIX socket path; Windows limits it to 107 bytes, so keep it short.</summary>
    public required string SocketPath { get; init; }

    /// <summary>Extra environment variables (FLASHTEX_BUNDLE_*, FLASHTEX_FORMAT_CACHE_DIR, ...).</summary>
    public IReadOnlyDictionary<string, string> Environment { get; init; } = new Dictionary<string, string>();

    public TimeSpan StartupTimeout { get; init; } = TimeSpan.FromMinutes(5);
}

/// <summary>A running flashtex-host; disposing it kills the process tree and removes the socket file.</summary>
public sealed class EngineHostProcess : IAsyncDisposable
{
    private const string ListeningMarker = "listening on";
    private const int MaxSocketPathBytes = 107;

    private readonly Process _process;
    private readonly EngineHostOptions _options;

    private EngineHostProcess(Process process, EngineHostOptions options)
    {
        _process = process;
        _options = options;
    }

    /// <summary>The socket path to hand to <see cref="EngineHostClient.ConnectAsync"/>.</summary>
    public string SocketPath => _options.SocketPath;

    /// <summary>Starts the host and waits until it listens. onStatus receives each line it prints.</summary>
    public static async Task<EngineHostProcess> StartAsync(
        EngineHostOptions options, Action<string>? onStatus, CancellationToken ct)
    {
        if (System.Text.Encoding.UTF8.GetByteCount(options.SocketPath) > MaxSocketPathBytes)
        {
            throw new ArgumentException($"socket path is longer than {MaxSocketPathBytes} bytes", nameof(options));
        }

        var process = Launch(options);
        var host = new EngineHostProcess(process, options);
        var listening = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        void OnLine(string? line)
        {
            if (line is null)
            {
                return;
            }

            onStatus?.Invoke(line);
            if (line.Contains(ListeningMarker, StringComparison.Ordinal))
            {
                listening.TrySetResult();
            }
        }

        process.OutputDataReceived += (_, e) => OnLine(e.Data);
        process.ErrorDataReceived += (_, e) => OnLine(e.Data);
        process.EnableRaisingEvents = true;
        process.Exited += (_, _) => listening.TrySetException(
            new EngineHostException("exited", $"flashtex-host exited with code {SafeExitCode(process)} before listening"));
        process.BeginOutputReadLine();
        process.BeginErrorReadLine();
        try
        {
            await listening.Task.WaitAsync(options.StartupTimeout, ct).ConfigureAwait(false);
            return host;
        }
        catch
        {
            await host.DisposeAsync().ConfigureAwait(false);
            throw;
        }
    }

    /// <inheritdoc />
    public ValueTask DisposeAsync()
    {
        try
        {
            if (!_process.HasExited)
            {
                _process.Kill(entireProcessTree: true);
            }
        }
        catch (InvalidOperationException)
        {
            // Already exited between the check and the kill.
        }

        _process.Dispose();
        TryDeleteSocketFile();
        return ValueTask.CompletedTask;
    }

    private static Process Launch(EngineHostOptions options)
    {
        var info = new ProcessStartInfo(options.HostExePath)
        {
            UseShellExecute = false,
            CreateNoWindow = true,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
        };
        info.ArgumentList.Add("--socket");
        info.ArgumentList.Add(options.SocketPath);
        foreach (var (key, value) in options.Environment)
        {
            info.Environment[key] = value;
        }

        TryDelete(options.SocketPath);
        return Process.Start(info)
            ?? throw new EngineHostException("launch", $"could not start {options.HostExePath}");
    }

    private static int SafeExitCode(Process process)
    {
        try
        {
            return process.ExitCode;
        }
        catch (InvalidOperationException)
        {
            return -1;
        }
    }

    private void TryDeleteSocketFile() => TryDelete(_options.SocketPath);

    private static void TryDelete(string path)
    {
        try
        {
            File.Delete(path);
        }
        catch (IOException)
        {
            // Best effort: a stale socket file is replaced on the next start.
        }
        catch (UnauthorizedAccessException)
        {
            // Same: not ours to remove, or still in use.
        }
    }
}
