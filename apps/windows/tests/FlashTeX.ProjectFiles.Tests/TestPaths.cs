// name: TestPaths.cs
// purpose: Locates repo-relative paths (the compiled flashtex-project-files
//   binary) from the test output directory, independent of the test runner's
//   working directory. Mirrors tests/FlashTeX.Ipc.Tests/TestPaths.cs.
// author: Claude Sonnet 5
// date: 2026-09-14

namespace FlashTeX.ProjectFiles.Tests;

internal static class TestPaths
{
    /// <summary>
    /// Walks up from the test binary's directory until a `.git` marker locates
    /// the repo root. A `.git` <b>file</b> counts as well as a directory, so a
    /// `git worktree` checkout stops at its own root: without that the walk
    /// climbs past it into the main checkout and these tests would exercise
    /// <em>another tree's</em> flashtex-project-files binary against this
    /// tree's expectations. Same reasoning as
    /// <c>FlashTeX.App.CompilerLocator.FindRepoRoot</c>.
    /// </summary>
    public static string RepoRoot { get; } = FindRepoRoot();

    /// <summary>
    /// The release build of `crates/project-files` (`cargo build --release`
    /// from that directory produces this). Tests that depend on it fail
    /// loudly, with the exact expected path, if it hasn't been built yet.
    /// </summary>
    public static string ProjectFilesExePath { get; } =
        Path.Combine(RepoRoot, "crates", "project-files", "target", "release", "flashtex-project-files.exe");

    private static string FindRepoRoot()
    {
        var dir = new DirectoryInfo(AppContext.BaseDirectory);
        while (dir is not null)
        {
            string marker = Path.Combine(dir.FullName, ".git");
            if (Directory.Exists(marker) || File.Exists(marker))
            {
                return dir.FullName;
            }
            dir = dir.Parent;
        }
        throw new DirectoryNotFoundException(
            $"could not locate the repo root (a '.git' directory or worktree pointer file) above '{AppContext.BaseDirectory}'");
    }
}
