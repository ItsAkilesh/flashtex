// name: MainWindow.Accessibility.cs
// purpose: Landmark/region roles for Narrator's landmark navigation, applied once
//   over the window's already-named top-level regions (toolbar, project
//   tree/outline, document tabs, editor, preview, Problems panel, status bar).
//   This is the chosen substitute for macOS VoiceOver's rotor for this port: the
//   original architecture plan explicitly scoped a full rotor-parity rebuild OUT
//   of this Windows port, in favor of Narrator's landmark/heading navigation plus
//   the existing Outline panel (MainWindow.Outline.cs) as an accepted substitute.
//   See apps/windows/HANDOFF.md's "Accessibility" section for the full writeup.
//   Per-control fixes (toolbar button names, tab strip names/keyboard operability,
//   project-tree/outline heading levels and row names, Problems panel row names)
//   live in each area's own file (MainWindow.Toolbar.cs, MainWindow.Tabs.cs,
//   MainWindow.ProjectFiles.cs, MainWindow.Outline.cs, MainWindow.Problems.cs) --
//   this file is only the window-level landmark structure, called once from the
//   constructor over elements that already exist and never get rebuilt.
// author: Claude Sonnet 5
// date: 2026-09-18

using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Automation.Peers;

namespace FlashTeX.App;

public sealed partial class MainWindow
{
    private void WireAccessibilityLandmarks()
    {
        AutomationProperties.SetLandmarkType(Toolbar, AutomationLandmarkType.Custom);
        AutomationProperties.SetName(Toolbar, "Toolbar");

        AutomationProperties.SetLandmarkType(ProjectTreeHost, AutomationLandmarkType.Navigation);
        AutomationProperties.SetName(ProjectTreeHost, "Project files and outline");

        // TabStripPanel is declared once in MainWindow.xaml and never replaced (only its
        // Children are rebuilt on RebuildTabStrip), so this landmark only needs setting once
        // here rather than on every rebuild.
        AutomationProperties.SetLandmarkType(TabStripPanel, AutomationLandmarkType.Navigation);
        AutomationProperties.SetName(TabStripPanel, "Open document tabs");

        AutomationProperties.SetLandmarkType(EditorPaneHost, AutomationLandmarkType.Main);
        AutomationProperties.SetName(EditorPaneHost, "Editor");

        AutomationProperties.SetLandmarkType(PreviewPaneHost, AutomationLandmarkType.Custom);
        AutomationProperties.SetName(PreviewPaneHost, "Preview");

        AutomationProperties.SetLandmarkType(ProblemsPanelHost, AutomationLandmarkType.Custom);
        AutomationProperties.SetName(ProblemsPanelHost, "Problems");

        AutomationProperties.SetLandmarkType(StatusBarPanel, AutomationLandmarkType.Custom);
        AutomationProperties.SetName(StatusBarPanel, "Status bar");
    }
}
