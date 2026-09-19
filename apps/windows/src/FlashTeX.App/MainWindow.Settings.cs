// name: MainWindow.Settings.cs
// purpose: Wires CommandIds.Settings to a singleton SettingsWindow -- reactivated
//   (never recreated) on repeated invocation, matching MainWindow.EditHistory.cs's
//   established pattern for EditHistoryWindow.
// author: Claude Sonnet 5
// date: 2026-09-18

namespace FlashTeX.App;

public sealed partial class MainWindow
{
    private SettingsWindow? _settingsWindow;

    private bool TryToggleSettings()
    {
        if (_settingsWindow is null)
        {
            _settingsWindow = new SettingsWindow(_shell);
            _settingsWindow.Closed += (_, _) => _settingsWindow = null;
        }
        _settingsWindow.Activate();
        return true;
    }
}
