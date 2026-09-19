// name: SettingsWindow.xaml.cs
// purpose: Singleton auxiliary window for the settings this port exposes:
//   editor font size/family, color theme (light/dark/system) and the
//   auto-compile debounce interval. Every control applies its change
//   immediately to the live ShellModel/AppTheme (no OK/Cancel/Apply, matching
//   the Mac port's SettingsRootView -- apps/mac/Sources/FlashTeXMac/
//   EditorPreferences.swift) and persists it via AppSettings.Save.
//   MainWindow.Settings.cs reactivates one existing instance rather than
//   creating a new one each time CommandIds.Settings runs, per this port's
//   established EditHistoryWindow/MainWindow.EditHistory.cs precedent.
// author: Claude Sonnet 5
// date: 2026-09-18

using FlashTeX.Shell;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;

namespace FlashTeX.App;

public sealed partial class SettingsWindow : Window
{
    /// <summary>
    /// The fixed monospace font choices offered (task scope: "a small fixed list ... doesn't need
    /// arbitrary font picking"). The first entry (null) is the bridge's own default stack (see
    /// FlashTeX.Editor/web/src/main.ts's <c>DEFAULT_FONT_FAMILY_STACK</c>); the rest ship with
    /// Windows 10/11 or Visual Studio/WebView2's Cascadia fonts, so no availability probing is
    /// attempted (unlike the Mac port's <c>installedMonospacedFamilies()</c>, which has an AppKit
    /// API for that; WinUI3 has no equivalent enumeration this port depends on elsewhere).
    /// </summary>
    private static readonly (string Label, string? Family)[] FontFamilyChoices =
    {
        ("Default (Cascadia Code)", null),
        ("Cascadia Code", "Cascadia Code"),
        ("Cascadia Mono", "Cascadia Mono"),
        ("Consolas", "Consolas"),
        ("Courier New", "Courier New"),
        ("Lucida Console", "Lucida Console"),
    };

    private readonly ShellModel _shell;

    /// <summary>True while populating controls from the current settings, so the change handlers those population calls trigger don't re-save/re-apply what was just loaded.</summary>
    private bool _initializing;

    public SettingsWindow(ShellModel shell)
    {
        _shell = shell;
        InitializeComponent();
        Title = "Settings";

        PopulateFontFamilyChoices();
        LoadCurrentValues();

        FontFamilyCombo.SelectionChanged += (_, _) => OnFontFamilyChanged();
        FontSizeSlider.ValueChanged += (_, _) => OnFontSizeChanged();
        ThemeCombo.SelectionChanged += (_, _) => OnThemeChanged();
        DebounceSlider.ValueChanged += (_, _) => OnDebounceChanged();
        RestoreDefaultsButton.Click += (_, _) => RestoreDefaults();

        SettingsRootGrid.RequestedTheme = AppTheme.Current;
        AppTheme.Changed += OnAppThemeChanged;
        Closed += OnClosed;
    }

    private void PopulateFontFamilyChoices()
    {
        foreach ((string label, string? family) in FontFamilyChoices)
        {
            FontFamilyCombo.Items.Add(new ComboBoxItem { Content = label, Tag = family });
        }
    }

    /// <summary>Seeds every control from ShellModel's live values (font size/family) plus AppTheme/CompileDebounceInterval, without triggering a redundant save/apply.</summary>
    private void LoadCurrentValues()
    {
        _initializing = true;
        try
        {
            int familyIndex = Array.FindIndex(FontFamilyChoices, c => c.Family == _shell.EditorFontFamily);
            FontFamilyCombo.SelectedIndex = familyIndex >= 0 ? familyIndex : 0;

            FontSizeSlider.Value = _shell.EditorFontSize;
            UpdateFontSizeReadout();
            UpdateFontSample();

            ThemeCombo.SelectedIndex = AppTheme.Choice switch
            {
                AppThemeChoice.Light => 1,
                AppThemeChoice.Dark => 2,
                _ => 0,
            };

            DebounceSlider.Value = _shell.CompileDebounceInterval.TotalMilliseconds;
            UpdateDebounceReadout();
        }
        finally
        {
            _initializing = false;
        }
    }

    private void OnFontFamilyChanged()
    {
        UpdateFontSample();
        if (_initializing)
        {
            return;
        }
        string? family = SelectedFontFamily();
        _shell.EditorFontFamily = family;
        SaveCurrentSettings();
    }

    private void OnFontSizeChanged()
    {
        UpdateFontSizeReadout();
        UpdateFontSample();
        if (_initializing)
        {
            return;
        }
        _shell.EditorFontSize = FontSizeSlider.Value;
        SaveCurrentSettings();
    }

    private void OnThemeChanged()
    {
        if (_initializing)
        {
            return;
        }
        AppTheme.Apply(SelectedTheme());
        SaveCurrentSettings();
    }

    private void OnDebounceChanged()
    {
        UpdateDebounceReadout();
        if (_initializing)
        {
            return;
        }
        _shell.CompileDebounceInterval = TimeSpan.FromMilliseconds(DebounceSlider.Value);
        SaveCurrentSettings();
    }

    private void RestoreDefaults()
    {
        AppSettingsData defaults = AppSettingsData.Default;
        _shell.EditorFontFamily = defaults.EditorFontFamily;
        _shell.EditorFontSize = defaults.EditorFontSize;
        _shell.CompileDebounceInterval = TimeSpan.FromMilliseconds(defaults.CompileDebounceMs);
        AppTheme.Apply(defaults.Theme);
        LoadCurrentValues();
        AppSettings.Save(defaults);
    }

    private string? SelectedFontFamily() =>
        (FontFamilyCombo.SelectedItem as ComboBoxItem)?.Tag as string;

    private AppThemeChoice SelectedTheme() => ThemeCombo.SelectedIndex switch
    {
        1 => AppThemeChoice.Light,
        2 => AppThemeChoice.Dark,
        _ => AppThemeChoice.System,
    };

    private void SaveCurrentSettings() => AppSettings.Save(new AppSettingsData(
        _shell.EditorFontSize, _shell.EditorFontFamily, AppTheme.Choice, _shell.CompileDebounceInterval.TotalMilliseconds));

    private void UpdateFontSizeReadout() => FontSizeReadout.Text = $"{FontSizeSlider.Value:0} pt";

    private void UpdateDebounceReadout() => DebounceReadout.Text = $"{DebounceSlider.Value:0} ms";

    /// <summary>Re-fonts the live sample text so the size/family choice is visible immediately, matching the Mac port's font-sample row.</summary>
    private void UpdateFontSample()
    {
        FontSampleText.FontFamily = new FontFamily(SelectedFontFamily() ?? "Cascadia Code, Consolas, monospace");
        FontSampleText.FontSize = FontSizeSlider.Value;
    }

    private void OnAppThemeChanged(ElementTheme theme) => DispatcherQueue.TryEnqueue(() => SettingsRootGrid.RequestedTheme = theme);

    private void OnClosed(object sender, WindowEventArgs args) => AppTheme.Changed -= OnAppThemeChanged;
}
