// name: EngineHostLocator.cs
// purpose: Finds what the app needs to run flashtex-host: the host binary, the
//   no-TeX-Live bundle lock file (its pinned URL and digest), and the per-user
//   data directories for the bundle cache, the format cache, compile output and
//   the scratch project. Also says whether a TeX Live is already on PATH, in
//   which case the host needs no bundle and nothing is downloaded. Reuses
//   CompilerLocator's walk to the repo root, like the other locators.
// author: Claude Sonnet 5.5 (paired with Akilesh S)
// date: 2026-10-28

using System.Text.RegularExpressions;

namespace FlashTeX.App;

/// <summary>The bundle a lock file pins: where to fetch it and the digest it must have.</summary>
internal sealed record BundleLock(string LockPath, string Url, string Digest);

internal static partial class EngineHostLocator
{
    private const string HostOverrideVariable = "FLASHTEX_HOST";
    private const string LockOverrideVariable = "FLASHTEX_BUNDLE_LOCK";
    private const string HostExeName = "flashtex-host.exe";
    private const string LockFileName = "flashtex-bundle.lock";
    private const string KpsewhichExeName = "kpsewhich.exe";
    private const string DataDirectoryName = "FlashTeX";

    private static readonly string[] HostRepoRelativePaths =
    [
        "target/x86_64-pc-windows-gnu/release/" + HostExeName,
        "target/release/" + HostExeName,
    ];

    private static readonly string LockRepoRelativePath = "tools/bundle/tl2026/" + LockFileName;

    /// <summary>The per-user root: bundle cache, format cache, output and scratch folders live under it.</summary>
    public static string DataRoot { get; } =
        Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), DataDirectoryName);

    public static string BundleCacheDirectory => Path.Combine(DataRoot, "bundle-cache");

    public static string FormatCacheDirectory => Path.Combine(DataRoot, "format-cache");

    /// <summary>The host binary: the FLASHTEX_HOST override, a repo build, or one beside the app; null if none exists.</summary>
    public static string? FindHost()
    {
        if (Environment.GetEnvironmentVariable(HostOverrideVariable) is { Length: > 0 } configured && File.Exists(configured))
        {
            return configured;
        }

        var candidates = new List<string> { Path.Combine(AppContext.BaseDirectory, HostExeName) };
        if (CompilerLocator.FindRepoRoot(new DirectoryInfo(AppContext.BaseDirectory)) is { } root)
        {
            candidates.AddRange(HostRepoRelativePaths.Select(p => Path.Combine(root.FullName, p.Replace('/', Path.DirectorySeparatorChar))));
        }

        return candidates.FirstOrDefault(File.Exists);
    }

    /// <summary>The bundle lock: the FLASHTEX_BUNDLE_LOCK override, the repo's, or one beside the app; null if unreadable.</summary>
    public static BundleLock? FindBundleLock()
    {
        var candidates = new List<string>();
        if (Environment.GetEnvironmentVariable(LockOverrideVariable) is { Length: > 0 } configured)
        {
            candidates.Add(configured);
        }

        candidates.Add(Path.Combine(AppContext.BaseDirectory, LockFileName));
        if (CompilerLocator.FindRepoRoot(new DirectoryInfo(AppContext.BaseDirectory)) is { } root)
        {
            candidates.Add(Path.Combine(root.FullName, LockRepoRelativePath.Replace('/', Path.DirectorySeparatorChar)));
        }

        return candidates.Where(File.Exists).Select(ParseLock).FirstOrDefault(l => l is not null);
    }

    /// <summary>Whether this bundle's files are already cached, so using it fetches nothing new at start.</summary>
    public static bool IsBundleCached(BundleLock bundle) =>
        Directory.Exists(Path.Combine(BundleCacheDirectory, bundle.Digest));

    /// <summary>Whether a TeX Live is on PATH (the host then reads it and needs no bundle).</summary>
    public static bool IsTexLiveOnPath() =>
        (Environment.GetEnvironmentVariable("PATH") ?? string.Empty)
            .Split(Path.PathSeparator, StringSplitOptions.RemoveEmptyEntries)
            .Any(directory => File.Exists(Path.Combine(directory, KpsewhichExeName)));

    internal static BundleLock? ParseLock(string lockPath)
    {
        string text;
        try
        {
            text = File.ReadAllText(lockPath);
        }
        catch (IOException)
        {
            return null;
        }

        var url = LockValue("url").Match(text);
        var digest = LockValue("digest").Match(text);
        return url.Success && digest.Success
            ? new BundleLock(lockPath, url.Groups[1].Value, digest.Groups[1].Value)
            : null;
    }

    private static Regex LockValue(string key) =>
        new($"""^\s*{key}\s*=\s*"([^"]+)"\s*(#.*)?$""", RegexOptions.Multiline | RegexOptions.CultureInvariant);
}
