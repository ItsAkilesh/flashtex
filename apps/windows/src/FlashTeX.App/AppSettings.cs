// name: AppSettings.cs
// purpose: Persists the user-facing app settings SettingsWindow exposes
//   (editor font size/family, color theme, auto-compile debounce interval)
//   across launches. Follows PaneSettings.cs's established pattern exactly
//   (a small JSON file under %LOCALAPPDATA%\FlashTeX, not
//   Windows.Storage.ApplicationData.Current.LocalSettings) rather than
//   inventing a second persistence mechanism: this app runs unpackaged
//   (WindowsPackageType=None, FlashTeX.App.csproj), and ApplicationData.Current
//   throws "the process has no package identity" without a real package/MSIX
//   identity -- see PaneSettings.cs's own header comment for the same finding.
//   A separate file (not window-panes.json) keeps the two concerns -- window
//   layout vs. user preferences -- independently readable/deletable.
// author: Claude Sonnet 5
// date: 2026-09-18

using System.Text.Json;
using FlashTeX.Shell;

namespace FlashTeX.App;

/// <summary>Which <see cref="Microsoft.UI.Xaml.ElementTheme"/> the app should use.</summary>
public enum AppThemeChoice
{
    System,
    Light,
    Dark,
}

/// <summary>The subset of user preferences this app remembers between launches.</summary>
public sealed record AppSettingsData(
    double EditorFontSize,
    string? EditorFontFamily,
    AppThemeChoice Theme,
    double CompileDebounceMs)
{
    /// <summary>Every field at its documented default -- what a first launch (or a corrupt/missing file) uses.</summary>
    public static readonly AppSettingsData Default = new(
        ShellModel.DefaultEditorFontSizePt,
        EditorFontFamily: null,
        AppThemeChoice.System,
        ShellModel.DefaultCompileDebounceInterval.TotalMilliseconds);

    /// <summary>
    /// Returns a copy with every numeric field clamped to the range SettingsWindow's controls
    /// allow, so a hand-edited or stale JSON file can never hand ShellModel/EditorHost an
    /// out-of-range value. <see cref="EditorFontFamily"/> is left as-is: SettingsWindow only
    /// ever offers a font already in its own fixed list or null, and re-validating an unknown
    /// family here would need the same "is it actually installed" logic the picker already
    /// encapsulates, which is out of scope for a plain data record.
    /// </summary>
    public AppSettingsData Clamped() => this with
    {
        EditorFontSize = Math.Clamp(EditorFontSize, ShellModel.MinEditorFontSizePt, ShellModel.MaxEditorFontSizePt),
        CompileDebounceMs = Math.Clamp(
            CompileDebounceMs,
            ShellModel.MinCompileDebounceInterval.TotalMilliseconds,
            ShellModel.MaxCompileDebounceInterval.TotalMilliseconds),
    };
}

/// <summary>Reads and writes <see cref="AppSettingsData"/> to a small JSON file, tolerating any I/O failure as "use defaults".</summary>
public static class AppSettings
{
    private static readonly string FilePath = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "FlashTeX", "app-settings.json");

    /// <summary>The saved settings (clamped to valid ranges), or null if none was saved yet or the file could not be read.</summary>
    public static AppSettingsData? TryLoad()
    {
        try
        {
            if (!File.Exists(FilePath))
            {
                return null;
            }
            AppSettingsData? loaded = JsonSerializer.Deserialize<AppSettingsData>(File.ReadAllText(FilePath));
            return loaded?.Clamped();
        }
        catch (Exception ex) when (ex is IOException or JsonException or UnauthorizedAccessException)
        {
            return null;
        }
    }

    /// <summary>Best-effort save; a failure here must never crash a settings change.</summary>
    public static void Save(AppSettingsData settings)
    {
        try
        {
            Directory.CreateDirectory(Path.GetDirectoryName(FilePath)!);
            File.WriteAllText(FilePath, JsonSerializer.Serialize(settings));
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        {
            // Best-effort only; the in-memory setting is still correct for this session.
        }
    }
}
