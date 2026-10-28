// name: MainWindow.Engine.cs
// purpose: Starts the new engine for the window: finds flashtex-host, asks the user
//   before any bundle download (the host only fetches what the app names in
//   FLASHTEX_BUNDLE_ALLOW_FETCH), launches the host, connects an EngineHostClient and
//   attaches it to the ShellModel. Returns false, with the reason in the status bar,
//   when anything is missing, so startup can fall back to the old worker path.
//   The host process and client are disposed when the window closes.
// author: Claude Sonnet 5.5 (paired with Akilesh S)
// date: 2026-10-28

using FlashTeX.EngineHost;
using Microsoft.UI.Xaml.Controls;

namespace FlashTeX.App;

public sealed partial class MainWindow
{
    private const string EngineSocketPrefix = "fx";
    private const int EngineSocketNameChars = 10;

    private EngineHostProcess? _engineProcess;
    private EngineHostClient? _engineClient;

    /// <summary>Starts and attaches flashtex-host. False when it could not (see WorkerStatus).</summary>
    private async Task<bool> TryStartEngineAsync()
    {
        var hostExe = EngineHostLocator.FindHost();
        if (hostExe is null)
        {
            _shell.WorkerStatus = "engine host not found (build flashtex-host, or set FLASHTEX_HOST)";
            return false;
        }

        var environment = await BuildEngineEnvironmentAsync().ConfigureAwait(true);
        if (environment is null)
        {
            return false;
        }

        try
        {
            Directory.CreateDirectory(EngineHostLocator.FormatCacheDirectory);
            var options = new EngineHostOptions
            {
                HostExePath = hostExe,
                SocketPath = NewSocketPath(),
                Environment = environment,
            };
            _shell.WorkerStatus = "starting engine host (the first start builds the format)...";
            _engineProcess = await EngineHostProcess.StartAsync(options, OnEngineStatusLine, CancellationToken.None).ConfigureAwait(true);
            _engineClient = await EngineHostClient.ConnectAsync(_engineProcess.SocketPath, CancellationToken.None).ConfigureAwait(true);
        }
        catch (Exception ex) when (ex is EngineHostException or Dl3ProtocolException or IOException or System.Net.Sockets.SocketException or TimeoutException)
        {
            _shell.WorkerStatus = "engine host failed: " + ex.Message;
            await DisposeEngineAsync().ConfigureAwait(true);
            return false;
        }

        var session = Guid.NewGuid().ToString("N");
        var scratch = Path.Combine(EngineHostLocator.DataRoot, "scratch", session);
        var output = Path.Combine(EngineHostLocator.DataRoot, "out", session);
        Directory.CreateDirectory(scratch);
        Directory.CreateDirectory(output);
        _shell.AttachEngineHost(new BufferMaterializingSession(_engineClient, scratch), scratch, output);
        return true;
    }

    /// <summary>
    /// The environment the host reads. With a TeX Live on PATH nothing extra is needed. Otherwise the
    /// bundle lock is passed, and the download is allowed only after the user agrees (or the bundle is
    /// already cached). Null when no bundle is available or the user declined.
    /// </summary>
    private async Task<Dictionary<string, string>?> BuildEngineEnvironmentAsync()
    {
        var environment = new Dictionary<string, string>
        {
            ["FLASHTEX_FORMAT_CACHE_DIR"] = EngineHostLocator.FormatCacheDirectory,
        };
        if (EngineHostLocator.IsTexLiveOnPath())
        {
            return environment;
        }

        if (EngineHostLocator.FindBundleLock() is not { } bundle)
        {
            _shell.WorkerStatus = "no TeX Live and no bundle lock found";
            return null;
        }

        environment["FLASHTEX_BUNDLE_LOCK"] = bundle.LockPath;
        environment["FLASHTEX_BUNDLE_CACHE_DIR"] = EngineHostLocator.BundleCacheDirectory;
        if (!EngineHostLocator.IsBundleCached(bundle) && !await AskBundleConsentAsync(bundle).ConfigureAwait(true))
        {
            _shell.WorkerStatus = "TeX bundle download declined: install TeX Live or accept the download to compile";
            return null;
        }

        environment["FLASHTEX_BUNDLE_ALLOW_FETCH"] = $"{bundle.Digest}@{bundle.Url}";
        return environment;
    }

    private async Task<bool> AskBundleConsentAsync(BundleLock bundle)
    {
        var dialog = new ContentDialog
        {
            Title = "Download the TeX bundle?",
            Content = "No TeX Live was found. FlashTeX can fetch the TeX packages and fonts it needs, " +
                "on demand, from:\n\n" + bundle.Url + "\n\nThey are verified against a pinned checksum " +
                "and cached for later.",
            PrimaryButtonText = "Download",
            CloseButtonText = "Not now",
            DefaultButton = ContentDialogButton.Primary,
            XamlRoot = Content.XamlRoot,
        };
        return await dialog.ShowAsync() == ContentDialogResult.Primary;
    }

    private void OnEngineStatusLine(string line)
    {
        DispatcherQueue.TryEnqueue(() =>
        {
            if (line.Contains("bundle_progress", StringComparison.Ordinal))
            {
                _shell.WorkerStatus = "downloading TeX bundle...";
            }
        });
    }

    private static string NewSocketPath() =>
        Path.Combine(Path.GetTempPath(), EngineSocketPrefix + Guid.NewGuid().ToString("N")[..EngineSocketNameChars] + ".sock");

    private async Task DisposeEngineAsync()
    {
        _shell.DetachEngineHost();
        if (_engineClient is not null)
        {
            await _engineClient.DisposeAsync().ConfigureAwait(true);
            _engineClient = null;
        }

        if (_engineProcess is not null)
        {
            await _engineProcess.DisposeAsync().ConfigureAwait(true);
            _engineProcess = null;
        }
    }
}
