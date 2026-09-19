// name: AppTheme.cs
// purpose: Process-wide "which color theme is the user's choice right now"
//   broadcast, so every open Window (MainWindow, EditHistoryWindow,
//   SettingsWindow) applies the same ElementTheme and stays in sync live when
//   the user changes it in Settings -- no restart needed. WinUI3 desktop apps
//   have no runtime-settable Application-wide theme (Application.RequestedTheme
//   is fixed at startup); the idiomatic seam is FrameworkElement.RequestedTheme
//   on each window's root element, which cascades ActualTheme down to every
//   descendant (including EditorHost's WebView2 control, whose own
//   ActualThemeChanged handler already re-sends `set_theme` to CodeMirror --
//   see EditorHost.xaml.cs/EditorHost.Bridge.cs). This class does not touch
//   XAML itself; each window subscribes to Changed and sets its own root's
//   RequestedTheme, and unsubscribes on Closed.
// author: Claude Sonnet 5
// date: 2026-09-18

using Microsoft.UI.Xaml;

namespace FlashTeX.App;

/// <summary>The single process-wide current theme choice, plus a change notification every open window subscribes to.</summary>
internal static class AppTheme
{
    /// <summary>The <see cref="AppThemeChoice"/> most recently applied via <see cref="Apply"/>; <see cref="AppThemeChoice.System"/> until anything is applied.</summary>
    public static AppThemeChoice Choice { get; private set; } = AppThemeChoice.System;

    /// <summary><see cref="Choice"/> translated to the WinUI3 enum every window's root element's <c>RequestedTheme</c> is set to.</summary>
    public static ElementTheme Current => ToElementTheme(Choice);

    /// <summary>Raised after <see cref="Choice"/>/<see cref="Current"/> change, with the new <see cref="ElementTheme"/>; every open window's constructor and Closed handler (de)registers a listener here.</summary>
    public static event Action<ElementTheme>? Changed;

    /// <summary>Sets the current theme and notifies every listening window immediately.</summary>
    public static void Apply(AppThemeChoice choice)
    {
        Choice = choice;
        Changed?.Invoke(Current);
    }

    private static ElementTheme ToElementTheme(AppThemeChoice choice) => choice switch
    {
        AppThemeChoice.Light => ElementTheme.Light,
        AppThemeChoice.Dark => ElementTheme.Dark,
        _ => ElementTheme.Default,
    };
}
