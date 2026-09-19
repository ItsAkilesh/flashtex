// name: MainWindow.Tabs.cs
// purpose: The document tab strip: one tab per FlashTeX.Shell.ShellModel.Documents
//   entry, the active one highlighted, a dirty dot, and a close button.
//   Rebuilds the strip on ShellModel.Documents.CollectionChanged (open/close)
//   and re-highlights on ActiveDocumentPath changes (switching tabs) — both
//   read directly from ShellModel rather than through ShellChrome's throttled
//   mirror, matching the Mac source's DocumentTabBar.swift: tab identity and
//   the active highlight must never lag a click, unlike the status bar's
//   word count (MainWindow.StatusBar.cs), which FT-071 measured as needing
//   throttling but tab switching was never implicated in that regression.
//   This is the one piece of the shell meant to feel genuinely functional
//   this milestone (ShellModel's document lifecycle is already fully tested),
//   not a placeholder.
// author: Claude Sonnet 5
// date: 2026-09-14

using System.Collections.Specialized;
using FlashTeX.Shell;
using Microsoft.UI;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Windows.System;

namespace FlashTeX.App;

public sealed partial class MainWindow
{
    private const double TabCornerRadius = 4;
    private const double DirtyDotDiameter = 6;

    private void WireTabStrip()
    {
        _shell.Documents.CollectionChanged += (_, _) => RebuildTabStrip();
        _shell.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName == nameof(ShellModel.ActiveDocumentPath))
            {
                RebuildTabStrip();
            }
        };
        RebuildTabStrip();
    }

    private void RebuildTabStrip()
    {
        TabStripPanel.Children.Clear();
        foreach (var document in _shell.Documents)
        {
            TabStripPanel.Children.Add(CreateTabButton(document));
        }
    }

    private Border CreateTabButton(ShellDocument document)
    {
        var isActive = document.Path == _shell.ActiveDocumentPath;
        var label = new TextBlock
        {
            Text = document.Path,
            VerticalAlignment = VerticalAlignment.Center,
            FontWeight = isActive ? Microsoft.UI.Text.FontWeights.SemiBold : Microsoft.UI.Text.FontWeights.Normal,
        };

        var content = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 6 };
        content.Children.Add(label);
        if (document.IsDirty)
        {
            content.Children.Add(CreateDirtyDot());
        }
        content.Children.Add(CreateCloseButton(document.Path));

        var tab = new Border
        {
            Child = content,
            Padding = new Thickness(10, 6, 6, 6),
            Margin = new Thickness(2, 4, 2, 0),
            CornerRadius = new CornerRadius(TabCornerRadius),
            Background = isActive
                ? new SolidColorBrush(Colors.SteelBlue) { Opacity = 0.25 }
                : new SolidColorBrush(Colors.Transparent),
            // Border is a FrameworkElement, not a Control, so it is neither a tab stop nor
            // Automation-named by default -- this hand-built tab would otherwise be invisible
            // to both keyboard-only users (Tab/Shift+Tab would skip straight over it) and
            // Narrator (an anonymous, nameless element). UIElement.IsTabStop/TabIndex/Focus
            // work on any UIElement (not just Control subclasses), so this does not require
            // promoting the tab to a real Button.
            IsTabStop = true,
            UseSystemFocusVisuals = true,
        };
        AutomationProperties.SetName(tab, document.IsDirty ? $"{document.Path}, unsaved changes" : document.Path);
        tab.PointerPressed += (_, _) => _shell.SwitchActiveDocument(document.Path);
        tab.KeyDown += (_, e) =>
        {
            if (e.Key is VirtualKey.Enter or VirtualKey.Space)
            {
                _shell.SwitchActiveDocument(document.Path);
                e.Handled = true;
            }
        };
        return tab;
    }

    private static Border CreateDirtyDot() => new()
    {
        Width = DirtyDotDiameter,
        Height = DirtyDotDiameter,
        CornerRadius = new CornerRadius(DirtyDotDiameter / 2),
        Background = new SolidColorBrush(Colors.Orange),
        VerticalAlignment = VerticalAlignment.Center,
    };

    private Button CreateCloseButton(string path)
    {
        var button = new Button
        {
            Content = "✕",
            Padding = new Thickness(4),
            FontSize = 10,
            Background = new SolidColorBrush(Colors.Transparent),
            BorderThickness = new Thickness(0),
        };
        // A bare "✕" glyph as Content would otherwise be read literally (e.g. "multiplication
        // sign") rather than announcing what the button actually does.
        AutomationProperties.SetName(button, $"Close {Path.GetFileName(path)}");
        button.Click += (_, _) => _ = _shell.CloseDocumentAsync(path);
        return button;
    }
}
