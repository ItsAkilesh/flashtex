// name: MainWindow.StatusBar.cs
// purpose: The bottom status bar: word count, the active document's dirty
//   state, compile/worker status and diagnostic counts. Reads exclusively
//   through ShellChrome's throttled, change-only mirror
//   (FlashTeX.Shell.ShellChrome) rather than ShellModel's raw properties
//   directly, per the FT-071 perf lesson ShellChrome's own doc comments
//   describe: reading raw model state from chrome-shown UI caused 3-4 whole-
//   window re-layouts per keystroke on the Mac original. ShellModel only
//   registers wordCount/activeDocumentDirty/hasCompileResult on its Chrome by
//   default (ShellModel.cs); this registers two more sources (error/warning
//   counts, worker status) the same way — Chrome.Register is public exactly
//   so a consuming UI layer can extend the mirror's surface without changing
//   FlashTeX.Shell itself.
// author: Claude Sonnet 5
// date: 2026-09-14

using FlashTeX.Shell;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Controls;
using Windows.System;

namespace FlashTeX.App;

public sealed partial class MainWindow
{
    private static class StatusChromeKeys
    {
        public const string ErrorCount = "app.errorCount";
        public const string WarningCount = "app.warningCount";
        public const string WorkerStatus = "app.workerStatus";
    }

    private readonly TextBlock _wordCountText = new();
    private readonly TextBlock _dirtyText = new();
    private readonly TextBlock _workerStatusText = new();
    private readonly TextBlock _diagnosticsText = new();

    private void WireStatusBar()
    {
        StatusBarPanel.Children.Add(_wordCountText);
        StatusBarPanel.Children.Add(_dirtyText);
        StatusBarPanel.Children.Add(_workerStatusText);
        StatusBarPanel.Children.Add(_diagnosticsText);

        // VS Code-style polish: clicking the diagnostics summary toggles the Problems panel
        // (MainWindow.Problems.cs) rather than only being reachable via the shortcut/menu item.
        // A plain TextBlock is not a tab stop and Tapped only fires for pointer/touch input, so
        // without the three lines below this was a real keyboard-only-operability gap (item 7's
        // scan-and-patch check): a control that responds only to a pointer gesture and was never
        // given IsTabStop is invisible to Tab/Shift+Tab navigation, even though the exact same
        // action (CommandIds.ToggleProblems, Ctrl+Shift+M) is already reachable from the View
        // menu/keyboard accelerator -- this just makes this shortcut path itself reachable too,
        // rather than leaving it as a mouse-only dead end.
        _diagnosticsText.Tapped += (_, _) => ToggleProblemsPanel();
        _diagnosticsText.IsTabStop = true;
        _diagnosticsText.UseSystemFocusVisuals = true;
        _diagnosticsText.KeyDown += (_, e) =>
        {
            if (e.Key is VirtualKey.Enter or VirtualKey.Space)
            {
                ToggleProblemsPanel();
                e.Handled = true;
            }
        };
        ToolTipService.SetToolTip(_diagnosticsText, "Toggle Problems panel");
        // Not AutomationProperties.Name -- that is left to default to the TextBlock's own Text
        // (the diagnostics count, e.g. "3 errors, 2 warnings"), which is already the correct
        // accessible name; HelpText adds the interactivity hint without overwriting it.
        AutomationProperties.SetHelpText(_diagnosticsText, "Activate to toggle the Problems panel");

        _shell.Chrome.Register(StatusChromeKeys.ErrorCount, () => _shell.ErrorCount);
        _shell.Chrome.Register(StatusChromeKeys.WarningCount, () => _shell.WarningCount);
        _shell.Chrome.Register(StatusChromeKeys.WorkerStatus, () => _shell.WorkerStatus);

        // ShellChrome's coalescing timer (RealTimeChromeScheduler) is a plain
        // System.Timers.Timer, so PublishPendingChanges - and therefore this
        // PropertyChanged event - fires on a ThreadPool thread, never the UI
        // thread. Setting a TextBlock.Text from off the UI thread is a cross-
        // thread COM call WinUI3 rejects; since nothing observed that failure
        // (a background Timer.Elapsed exception has no caller to propagate to),
        // it was silently swallowed and the status bar never updated. Marshal
        // back via DispatcherQueue before touching any control.
        _shell.Chrome.PropertyChanged += (_, _) => DispatcherQueue.TryEnqueue(RefreshStatusBarText);

        // ShellModel only calls Chrome.NotifyPossibleChange() when WordCount, ActiveDocumentPath
        // or HasCompileResult change (ShellModel.cs's OnModelPropertyChanged); attaching/detaching
        // a worker changes none of those, so the two extra sources above need an explicit nudge.
        _shell.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName is nameof(ShellModel.HasWorkerAttached) or nameof(ShellModel.WorkerStatus))
            {
                _shell.Chrome.NotifyPossibleChange();
            }
        };

        RefreshStatusBarText();
    }

    private void RefreshStatusBarText()
    {
        var wordCount = _shell.Chrome.Get<int>(ShellChromeKeys.WordCount);
        _wordCountText.Text = $"{wordCount} words";

        var isDirty = _shell.Chrome.Get<bool>(ShellChromeKeys.ActiveDocumentDirty);
        _dirtyText.Text = isDirty ? "edited" : "saved";

        _workerStatusText.Text = _shell.Chrome.Get<string>(StatusChromeKeys.WorkerStatus, "no worker attached");

        var errors = _shell.Chrome.Get<int>(StatusChromeKeys.ErrorCount);
        var warnings = _shell.Chrome.Get<int>(StatusChromeKeys.WarningCount);
        _diagnosticsText.Text = errors == 0 && warnings == 0 ? "no diagnostics" : $"{errors} errors, {warnings} warnings";
    }
}
