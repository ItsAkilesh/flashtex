// name: MainWindow.ProjectFiles.cs
// purpose: Native open/save/rename/delete flows backed by flashtex-project-files.
// The helper owns rooted, no-follow reads, optimistic-concurrency saves, unlinks
// and single-syscall no-replace renames; the WinUI window owns picker ownership,
// confirmation prompts and user-facing conflict feedback.

using FlashTeX.ProjectFiles;
using FlashTeX.Protocol.ProjectFilesV1;
using FlashTeX.Shell;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Automation.Peers;
using Microsoft.UI.Xaml.Controls;
using Windows.Storage.Pickers;
using WinRT.Interop;

namespace FlashTeX.App;

public sealed partial class MainWindow
{
    private sealed record OpenFileBinding(DocumentFilesClient Client, string Root, string RelativePath, string? Sha256);

    private readonly Dictionary<string, DocumentFilesClient> _fileClientsByRoot = new(StringComparer.OrdinalIgnoreCase);
    private readonly Dictionary<string, OpenFileBinding> _fileBindingsByDocument = new(StringComparer.OrdinalIgnoreCase);
    private string? _activeProjectRoot;
    private string? _projectFilesToolMissingReason;

    /// <summary>
    /// True once a project root was established via <see cref="OpenFolderAsync"/>
    /// (as opposed to <see cref="OpenLatexFileAsync"/>/Save As/New File, which
    /// only ever establish a root as the side effect of one file). Gates the
    /// extra "FOLDER" section <see cref="RebuildProjectTree"/> adds — the
    /// single-file flow's tree (open documents + include rows) is otherwise
    /// completely unchanged.
    /// </summary>
    private bool _isFolderProject;

    /// <summary>
    /// The most recent folder listing (flashtex-project-files' <c>list</c>
    /// operation), root-relative paths, sorted. Populated by
    /// <see cref="OpenFolderAsync"/>; stale once files are added or removed on
    /// disk outside FlashTeX — there is no live folder watch for this yet
    /// (HANDOFF.md's "Folder-level project membership" section says so
    /// explicitly), only the existing per-open-document external-change watch.
    /// </summary>
    private IReadOnlyList<string> _folderProjectFiles = [];

    private void WireProjectTree()
    {
        _shell.Documents.CollectionChanged += (_, _) => RebuildProjectTree();
        _shell.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName == nameof(ShellModel.ActiveDocumentPath))
            {
                RebuildProjectTree();
            }
        };
        WireDocumentWatchers();
        RebuildProjectTree();
    }

    private void RebuildProjectTree()
    {
        var panel = new StackPanel { Spacing = 2, Padding = new Thickness(6) };
        var projectHeader = new TextBlock { Text = "PROJECT", FontSize = 12, Opacity = 0.65, Margin = new Thickness(6, 4, 6, 6) };
        // Gives Narrator's heading navigation (the plan's chosen substitute for a rebuilt
        // macOS-rotor equivalent -- see HANDOFF.md's Accessibility section) a real landmark to
        // jump to; a plain TextBlock has no heading semantics by default.
        AutomationProperties.SetHeadingLevel(projectHeader, AutomationHeadingLevel.Level1);
        panel.Children.Add(projectHeader);
        if (_shell.Documents.Count == 0)
        {
            panel.Children.Add(new TextBlock { Text = "No files open", Opacity = 0.65, Margin = new Thickness(6) });
        }
        foreach (var document in _shell.Documents)
        {
            string label = Path.GetFileName(document.Path);
            var button = new Button
            {
                Content = document.IsDirty ? label + " •" : label,
                HorizontalAlignment = HorizontalAlignment.Stretch,
                HorizontalContentAlignment = HorizontalAlignment.Left,
                Padding = new Thickness(8, 5, 8, 5),
                FontWeight = document.Path == _shell.ActiveDocumentPath
                    ? Microsoft.UI.Text.FontWeights.SemiBold
                    : Microsoft.UI.Text.FontWeights.Normal,
            };
            string path = document.Path;
            button.Click += (_, _) => _shell.SwitchActiveDocument(path);
            button.ContextFlyout = BuildProjectRowMenu(path);
            // Match what is actually shown: the dirty marker is a visual-only "•" glyph, so a
            // screen-reader user needs the unsaved-changes state spelled out in the name instead.
            AutomationProperties.SetName(button, document.IsDirty ? $"{label}, unsaved changes" : label);
            panel.Children.Add(button);
        }

        if (_activeProjectRoot is not null && ActiveDocumentOrNull() is { } active && _fileBindingsByDocument.TryGetValue(active.Path, out var binding))
        {
            foreach (var reference in ProjectIncludes.Scan(active.Text).Where(reference => reference.Literal))
            {
                foreach (string candidate in ProjectIncludes.Candidates(reference.Argument))
                {
                    if (_shell.Documents.Any(document => document.Path == candidate))
                    {
                        continue;
                    }
                    var include = new Button
                    {
                        Content = "↳ " + candidate,
                        HorizontalAlignment = HorizontalAlignment.Stretch,
                        HorizontalContentAlignment = HorizontalAlignment.Left,
                        Padding = new Thickness(16, 4, 8, 4),
                        Opacity = 0.8,
                    };
                    include.Click += (_, _) => _ = OpenProjectRelativeFileAsync(binding, candidate);
                    AutomationProperties.SetName(include, $"Open referenced file {candidate}");
                    panel.Children.Add(include);
                    break; // candidate order is .tex then literal; opening probes both.
                }
            }
        }

        AppendFolderSection(panel);
        AppendOutlineSection(panel, ActiveDocumentOrNull());
        ProjectTreeHost.Child = panel;
    }

    /// <summary>
    /// The folder-backed project tree: every file <see cref="OpenFolderAsync"/>'s
    /// last listing found, as a flat, sorted list of clickable rows — chosen
    /// over a fully general, expand/collapse <c>TreeView</c> because a project
    /// this size (dozens to a few hundred <c>.tex</c>/<c>.bib</c> files, not
    /// thousands of arbitrary assets) reads perfectly well as one alphabetized
    /// list, and a flat list needs no lazy-loading/expansion-state plumbing to
    /// stay honest about what is actually on disk. No-op when no folder
    /// project is open (<see cref="_isFolderProject"/> false), so the
    /// single-file-open tree above is completely unaffected. A no-op for the
    /// empty-folder case too (nothing to add). This never re-lists the folder
    /// itself; see <see cref="_folderProjectFiles"/> for why files added or
    /// removed outside FlashTeX are only picked up by reopening the folder.
    /// </summary>
    private void AppendFolderSection(StackPanel panel)
    {
        if (!_isFolderProject || _folderProjectFiles.Count == 0)
        {
            return;
        }

        var folderHeader = new TextBlock { Text = "FOLDER", FontSize = 12, Opacity = 0.65, Margin = new Thickness(6, 10, 6, 6) };
        AutomationProperties.SetHeadingLevel(folderHeader, AutomationHeadingLevel.Level1);
        panel.Children.Add(folderHeader);

        foreach (string relativePath in _folderProjectFiles)
        {
            bool isOpen = _shell.Documents.Any(document => document.Path == relativePath);
            var button = new Button
            {
                Content = relativePath,
                HorizontalAlignment = HorizontalAlignment.Stretch,
                HorizontalContentAlignment = HorizontalAlignment.Left,
                Padding = new Thickness(8, 4, 8, 4),
                Opacity = isOpen ? 1.0 : 0.85,
                FontWeight = relativePath == _shell.ActiveDocumentPath
                    ? Microsoft.UI.Text.FontWeights.SemiBold
                    : Microsoft.UI.Text.FontWeights.Normal,
            };
            string capturedPath = relativePath;
            button.Click += (_, _) => _ = RunProjectFileActionAsync(() => OpenFolderProjectFileAsync(capturedPath));
            AutomationProperties.SetName(button, isOpen ? $"{relativePath}, open" : relativePath);
            panel.Children.Add(button);
        }
    }

    /// <summary>
    /// The right-click menu on a project-tree row. This is the app's first
    /// <see cref="MenuFlyout"/> outside the <c>MenuBar</c> (nothing else had a
    /// context menu yet), so it deliberately mirrors
    /// <see cref="CreateMenuFlyoutItem"/>'s shape: a plain
    /// <see cref="MenuFlyoutItem.Text"/> per action — which, unlike a button
    /// whose content is a panel, UI Automation does expose a usable
    /// <c>Name</c> for — dispatching through the same
    /// <see cref="RunProjectFileCommandAsync"/> entry point the File menu's
    /// Rename/Delete commands use, so there is one implementation and one
    /// error-reporting path rather than two.
    /// </summary>
    private MenuFlyout BuildProjectRowMenu(string documentPath)
    {
        var flyout = new MenuFlyout();
        var rename = new MenuFlyoutItem { Text = "Rename…" };
        rename.Click += (_, _) => _ = RunProjectFileActionAsync(() => RenameDocumentFileAsync(documentPath));
        var delete = new MenuFlyoutItem { Text = "Delete…" };
        delete.Click += (_, _) => _ = RunProjectFileActionAsync(() => DeleteDocumentFileAsync(documentPath));
        flyout.Items.Add(rename);
        flyout.Items.Add(delete);
        return flyout;
    }

    private static bool IsProjectFileCommand(string id) =>
        id is CommandIds.OpenLatexFile or CommandIds.Save or CommandIds.SaveAs or CommandIds.NewFile
            or CommandIds.RenameFile or CommandIds.DeleteFile or CommandIds.OpenFolder;

    private bool TryStartProjectFileCommand(string commandId)
    {
        var command = CommandRegistry.All.First(c => c.Id == commandId);
        if (!command.CanExecute(_shell))
        {
            return false;
        }
        _ = RunProjectFileCommandAsync(commandId);
        return true;
    }

    private Task RunProjectFileCommandAsync(string commandId) => RunProjectFileActionAsync(() => commandId switch
    {
        CommandIds.OpenLatexFile => OpenLatexFileAsync(),
        CommandIds.Save => SaveActiveFileAsync(saveAs: false),
        CommandIds.SaveAs => SaveActiveFileAsync(saveAs: true),
        CommandIds.NewFile => CreateNewFileAsync(),
        CommandIds.RenameFile => RenameDocumentFileAsync(ActiveDocumentOrThrow().Path),
        CommandIds.DeleteFile => DeleteDocumentFileAsync(ActiveDocumentOrThrow().Path),
        CommandIds.OpenFolder => OpenFolderAsync(),
        _ => Task.CompletedTask,
    });

    /// <summary>
    /// The one place every project-file flow's failure is surfaced. Each of
    /// these is dispatched fire-and-forget from a synchronous handler (a menu
    /// item's <c>Click</c>, <see cref="ExecuteCommand"/>), so an escaping
    /// exception would otherwise become an unobserved task exception and the
    /// action would appear to silently do nothing — the exact failure mode
    /// rename and delete must not have.
    /// </summary>
    private async Task RunProjectFileActionAsync(Func<Task> action)
    {
        try
        {
            await action().ConfigureAwait(true);
        }
        catch (ProjectFilesErrorException ex)
        {
            await ShowProjectFileDialogAsync(
                "File operation refused",
                $"The project-file helper refused this operation ({ex.Code}).\n\n{ex.Message}").ConfigureAwait(true);
        }
        catch (Exception ex)
        {
            await ShowProjectFileDialogAsync("File operation failed", ex.Message).ConfigureAwait(true);
        }
        finally
        {
            RefreshCommandEnabledState();
        }
    }

    private async Task OpenLatexFileAsync()
    {
        var picker = new FileOpenPicker { SuggestedStartLocation = PickerLocationId.DocumentsLibrary };
        picker.FileTypeFilter.Add(".tex");
        picker.FileTypeFilter.Add(".bib");
        InitializeWithWindow.Initialize(picker, WindowNative.GetWindowHandle(this));
        var file = await picker.PickSingleFileAsync();
        if (file is null)
        {
            return;
        }

        string selectedPath = file.Path;
        string root = Path.GetDirectoryName(selectedPath) ?? throw new InvalidOperationException("The selected file has no parent directory.");
        if (!await SetProjectRootForSaveAsync(root).ConfigureAwait(true))
        {
            return;
        }
        string documentPath = Path.GetFileName(selectedPath);
        var client = GetOrStartFileClient(root);
        var read = await client.ReadAsync(documentPath).ConfigureAwait(true);
        if (!read.Exists || read.Text is null)
        {
            throw new IOException($"The selected file no longer exists: {selectedPath}");
        }

        await _shell.OpenDocumentAsync(documentPath, read.Text).ConfigureAwait(true);
        _fileBindingsByDocument[documentPath] = new OpenFileBinding(client, root, read.Path, read.Sha256);
    }

    /// <summary>
    /// Opens a whole folder as the project root (as opposed to
    /// <see cref="OpenLatexFileAsync"/>, which only ever derives a root as the
    /// parent of one picked file) and populates a real, persistent
    /// folder-backed project tree via flashtex-project-files' <c>list</c>
    /// operation — see <see cref="RebuildProjectTree"/>'s "FOLDER" section.
    ///
    /// This deliberately uses <see cref="NativeFolderPicker"/> (raw
    /// <c>IFileOpenDialog</c>/<c>FOS_PICKFOLDERS</c> COM interop), not
    /// <see cref="Windows.Storage.Pickers.FolderPicker"/> — the same
    /// <c>InitializeWithWindow</c>-owned WinRT picker
    /// <see cref="OpenLatexFileAsync"/>/<see cref="SaveActiveFileAsync"/> use
    /// successfully for <c>FileOpenPicker</c>/<c>FileSavePicker</c>. Tried
    /// first for consistency, it reliably threw
    /// <c>COMException 0x80004005</c> (E_FAIL) from <c>PickSingleFolderAsync()</c>
    /// in this unpackaged app when confirmed with a real UI-Automation-driven
    /// picker interaction — see <see cref="NativeFolderPicker"/>'s header
    /// comment for the full diagnosis. The dialog the user sees is identical
    /// either way (the WinRT picker is itself a thin wrapper over the same
    /// native dialog); only the broken post-pick marshaling is bypassed.
    /// </summary>
    private async Task OpenFolderAsync()
    {
        // A synchronous, blocking call on the calling (UI/STA) thread — exactly
        // how IFileOpenDialog::Show is meant to be driven, and consistent with
        // this being a modal dialog; wrapping it in Task.Run would move it onto
        // a thread-pool (MTA) thread, which is the wrong apartment for a modal
        // common item dialog parented to this window.
        string? root = NativeFolderPicker.PickFolder(WindowNative.GetWindowHandle(this), "Select a project folder");
        if (root is null)
        {
            return;
        }

        if (!await EstablishProjectRootAsync(root).ConfigureAwait(true))
        {
            return;
        }

        var client = GetOrStartFileClient(root);
        ListPayload listing = await client.ListAsync().ConfigureAwait(true);
        _isFolderProject = true;
        _folderProjectFiles = listing.Files;
        RebuildProjectTree();

        if (listing.Truncated)
        {
            await ShowProjectFileDialogAsync(
                "Large project — list truncated",
                $"This folder has more project files than FlashTeX lists at once; showing the first {listing.Files.Count}. " +
                "Files beyond that are still on disk and can still be opened with Open LaTeX File.").ConfigureAwait(true);
        }
    }

    /// <summary>
    /// Opens <paramref name="relativePath"/> (a row from the folder tree's
    /// "FOLDER" section) against the current folder project root. Switches to
    /// the tab instead of opening a second copy if it is already open,
    /// mirroring <see cref="OpenProjectRelativeFileAsync"/>'s include-row
    /// behavior but without requiring an already-open source document to scan
    /// references from.
    /// </summary>
    private async Task OpenFolderProjectFileAsync(string relativePath)
    {
        if (_activeProjectRoot is not { } root)
        {
            return;
        }
        if (_shell.Documents.Any(document => document.Path == relativePath))
        {
            _shell.SwitchActiveDocument(relativePath);
            return;
        }

        var client = GetOrStartFileClient(root);
        var read = await client.ReadAsync(relativePath).ConfigureAwait(true);
        if (!read.Exists || read.Text is null)
        {
            await ShowProjectFileDialogAsync(
                "File not found",
                $"“{relativePath}” no longer exists on disk. It will disappear from this list next time the folder is reopened.").ConfigureAwait(true);
            return;
        }
        await _shell.OpenDocumentAsync(read.Path, read.Text).ConfigureAwait(true);
        _fileBindingsByDocument[read.Path] = new OpenFileBinding(client, root, read.Path, read.Sha256);
    }

    private async Task SaveActiveFileAsync(bool saveAs)
    {
        var document = ActiveDocumentOrThrow();
        if (!saveAs && _fileBindingsByDocument.TryGetValue(document.Path, out var binding))
        {
            await SaveBoundDocumentAsync(document, binding).ConfigureAwait(true);
            return;
        }

        var picker = new FileSavePicker
        {
            SuggestedStartLocation = PickerLocationId.DocumentsLibrary,
            SuggestedFileName = Path.GetFileNameWithoutExtension(document.Path),
        };
        picker.FileTypeChoices.Add("LaTeX source", new List<string> { ".tex" });
        InitializeWithWindow.Initialize(picker, WindowNative.GetWindowHandle(this));
        var destination = await picker.PickSaveFileAsync();
        if (destination is null)
        {
            return;
        }

        string selectedPath = destination.Path;
        string root = Path.GetDirectoryName(selectedPath) ?? throw new InvalidOperationException("The selected location has no parent directory.");
        if (!await SetProjectRootForSaveAsync(root).ConfigureAwait(true))
        {
            return;
        }
        string destinationPath = Path.GetFileName(selectedPath);
        var client = GetOrStartFileClient(root);
        var onDisk = await client.ReadAsync(destinationPath).ConfigureAwait(true);
        var newBinding = new OpenFileBinding(client, root, destinationPath, onDisk.Sha256);
        var savedBinding = await SaveBoundDocumentAsync(document, newBinding).ConfigureAwait(true);
        if (savedBinding is null)
        {
            return;
        }

        if (!string.Equals(document.Path, destinationPath, StringComparison.OrdinalIgnoreCase))
        {
            await _shell.OpenDocumentAsync(destinationPath, document.Text).ConfigureAwait(true);
            _fileBindingsByDocument[destinationPath] = savedBinding;
            _shell.MarkDocumentSaved(destinationPath);
            _fileBindingsByDocument.Remove(document.Path);
            await _shell.CloseDocumentAsync(document.Path).ConfigureAwait(true);
        }
    }

    private async Task CreateNewFileAsync()
    {
        var picker = new FileSavePicker
        {
            SuggestedStartLocation = PickerLocationId.DocumentsLibrary,
            SuggestedFileName = "untitled",
        };
        picker.FileTypeChoices.Add("LaTeX source", new List<string> { ".tex" });
        InitializeWithWindow.Initialize(picker, WindowNative.GetWindowHandle(this));
        var destination = await picker.PickSaveFileAsync();
        if (destination is null)
        {
            return;
        }

        string selectedPath = destination.Path;
        string root = Path.GetDirectoryName(selectedPath) ?? throw new InvalidOperationException("The selected location has no parent directory.");
        if (!await EstablishProjectRootAsync(root).ConfigureAwait(true))
        {
            return;
        }
        string path = Path.GetFileName(selectedPath);
        var client = GetOrStartFileClient(root);
        var existing = await client.ReadAsync(path).ConfigureAwait(true);
        if (existing.Exists)
        {
            await ShowProjectFileDialogAsync("File already exists", "Choose a new filename rather than replacing an existing project source file.").ConfigureAwait(true);
            return;
        }

        const string template = "\\documentclass{article}\n\\begin{document}\n\n\\end{document}\n";
        var binding = new OpenFileBinding(client, root, path, null);
        var transient = new ShellDocument(path, template, 1, false);
        var savedBinding = await SaveBoundDocumentAsync(transient, binding).ConfigureAwait(true);
        if (savedBinding is null)
        {
            return;
        }
        await _shell.OpenDocumentAsync(path, template).ConfigureAwait(true);
        _fileBindingsByDocument[path] = savedBinding;
        _shell.MarkDocumentSaved(path);
    }

    /// <summary>
    /// Renames <paramref name="documentPath"/>'s file inside its own project
    /// directory, then re-keys everything that tracked the old path.
    ///
    /// The on-disk step is <see cref="DocumentFilesClient.RenameAsync"/>, which
    /// is a single no-replace rename in the helper
    /// (crates/project-files <c>ProjectLock::rename</c>): it either moves the
    /// entry or changes nothing, so an interrupted rename can never leave two
    /// copies or none, and an existing file at the new name is refused rather
    /// than clobbered. Only once that has succeeded is the shell's own state
    /// moved, via <see cref="ShellModel.RenameDocument"/> — which keeps the
    /// tab, its unsaved edits and its durable undo/redo history instead of
    /// closing and reopening it. The file binding is re-keyed *before* the
    /// shell rename, because <see cref="ReconcileDocumentWatchers"/> runs off
    /// <c>Documents.CollectionChanged</c> and drops the binding and watcher of
    /// any path that is no longer open.
    /// </summary>
    private async Task RenameDocumentFileAsync(string documentPath)
    {
        if (!_fileBindingsByDocument.TryGetValue(documentPath, out var binding))
        {
            await ShowProjectFileDialogAsync(
                "Nothing to rename yet",
                $"“{documentPath}” is only open in memory — it has no file in a project on disk. Use Save As to give it one first.").ConfigureAwait(true);
            return;
        }

        string currentName = ProjectRelativeFileName(binding.RelativePath);
        string? entered = await PromptForFileNameAsync("Rename file", $"New name for “{currentName}”:", currentName).ConfigureAwait(true);
        if (entered is null)
        {
            return; // cancelled
        }
        string newName = entered.Trim();
        if (newName.Length == 0 || newName == currentName)
        {
            return;
        }
        if (InvalidFileNameReason(newName) is { } reason)
        {
            await ShowProjectFileDialogAsync("Not a usable file name", reason).ConfigureAwait(true);
            return;
        }

        string newRelativePath = ReplaceProjectRelativeFileName(binding.RelativePath, newName);
        if (_shell.Documents.Any(d => d.Path == newRelativePath))
        {
            await ShowProjectFileDialogAsync(
                "That name is already open",
                $"“{newRelativePath}” is already open in another tab. Close it first, or choose a different name.").ConfigureAwait(true);
            return;
        }

        RenameOutcome outcome = await binding.Client.RenameAsync(binding.RelativePath, newRelativePath).ConfigureAwait(true);
        if (outcome is RenameOutcome.Conflict conflict)
        {
            await ShowProjectFileDialogAsync("Rename refused", DescribeRenameConflict(conflict.Details, binding.RelativePath, newRelativePath)).ConfigureAwait(true);
            return;
        }

        var renamed = (RenameOutcome.Renamed)outcome;
        _fileBindingsByDocument[renamed.To] = binding with { RelativePath = renamed.To };
        if (!_shell.RenameDocument(documentPath, renamed.To))
        {
            // The tab closed while the helper was answering. The file did move,
            // so the new binding above is the truthful record of it; the tab
            // simply is not there to re-key.
            _fileBindingsByDocument.Remove(renamed.To);
        }
        _fileBindingsByDocument.Remove(documentPath);
        RebuildProjectTree();
    }

    /// <summary>
    /// Deletes <paramref name="documentPath"/>'s file after an explicit
    /// confirmation, then closes its tab. A file that another process already
    /// deleted reports <c>removed:false</c> rather than failing — the tab is
    /// still closed (the intent is satisfied) and the difference is stated
    /// rather than hidden.
    /// </summary>
    private async Task DeleteDocumentFileAsync(string documentPath)
    {
        if (!_fileBindingsByDocument.TryGetValue(documentPath, out var binding))
        {
            await ShowProjectFileDialogAsync(
                "Nothing to delete yet",
                $"“{documentPath}” is only open in memory — it has no file in a project on disk.").ConfigureAwait(true);
            return;
        }

        bool isDirty = _shell.Documents.Any(d => d.Path == documentPath && d.IsDirty);
        var confirm = new ContentDialog
        {
            Title = "Delete file?",
            Content = $"“{Path.Combine(binding.Root, binding.RelativePath)}” will be deleted from disk and its tab closed."
                + (isDirty ? " This tab has unsaved changes, which will be lost." : string.Empty)
                + " This cannot be undone from FlashTeX.",
            PrimaryButtonText = "Delete",
            CloseButtonText = "Cancel",
            DefaultButton = ContentDialogButton.Close,
            XamlRoot = Content.XamlRoot,
        };
        if (await confirm.ShowAsync() != ContentDialogResult.Primary)
        {
            return;
        }

        RemovePayload removed = await binding.Client.RemoveAsync(binding.RelativePath).ConfigureAwait(true);
        _fileBindingsByDocument.Remove(documentPath);
        await _shell.CloseDocumentAsync(documentPath).ConfigureAwait(true);
        RebuildProjectTree();
        if (!removed.Removed)
        {
            await ShowProjectFileDialogAsync(
                "File was already gone",
                $"“{removed.Path}” no longer existed on disk, so nothing was deleted. Its tab has been closed.").ConfigureAwait(true);
        }
    }

    /// <summary>
    /// A one-field prompt, matching <see cref="ShowProjectFileDialogAsync"/>'s
    /// plain <see cref="ContentDialog"/> shape. Returns null when the user
    /// cancels (as opposed to an empty string, which a caller treats as "no
    /// change"). The <see cref="TextBox"/> carries an explicit
    /// <see cref="AutomationProperties"/> name because WinUI computes none for
    /// a bare text box, and without one UI Automation cannot find it to drive
    /// this dialog.
    /// </summary>
    private async Task<string?> PromptForFileNameAsync(string title, string prompt, string current)
    {
        var input = new TextBox { Text = current, SelectionStart = 0, SelectionLength = current.Length };
        AutomationProperties.SetName(input, "New file name");
        var panel = new StackPanel { Spacing = 8 };
        panel.Children.Add(new TextBlock { Text = prompt, TextWrapping = TextWrapping.Wrap });
        panel.Children.Add(input);

        var dialog = new ContentDialog
        {
            Title = title,
            Content = panel,
            PrimaryButtonText = "Rename",
            CloseButtonText = "Cancel",
            DefaultButton = ContentDialogButton.Primary,
            XamlRoot = Content.XamlRoot,
        };
        return await dialog.ShowAsync() == ContentDialogResult.Primary ? input.Text : null;
    }

    /// <summary>
    /// Why <paramref name="name"/> cannot be a project file name, or null if it
    /// can. The helper's own <c>ProjectPath</c> normalization is the authority
    /// and rejects all of these too; this exists only so the user reads a
    /// sentence about the name they typed instead of an <c>invalid_path</c>
    /// error code.
    /// </summary>
    private static string? InvalidFileNameReason(string name)
    {
        if (name is "." or "..")
        {
            return $"“{name}” names a directory, not a file.";
        }
        if (name.IndexOfAny(['/', '\\']) >= 0)
        {
            return "Rename changes a file's name inside its own folder, so the new name cannot contain a path separator.";
        }
        if (name.Contains(':') || name.IndexOfAny(['*', '?', '"', '<', '>', '|']) >= 0)
        {
            return "A project file name cannot contain any of : * ? \" < > |";
        }
        if (name.Any(char.IsControl))
        {
            return "A project file name cannot contain control characters.";
        }
        return null;
    }

    /// <summary>
    /// A rename conflict in the words of the thing the user did. Nothing moved
    /// in either case; the old file is exactly as it was.
    /// </summary>
    private static string DescribeRenameConflict(SaveConflict conflict, string from, string to) => conflict.Kind switch
    {
        ConflictKind.already_exists =>
            $"“{to}” already exists in this project, and FlashTeX will not overwrite it. Nothing was renamed — choose another name, or remove that file first.",
        ConflictKind.deleted_externally =>
            $"“{from}” is no longer on disk, so there was nothing to rename. Another program may have moved or deleted it.",
        var kind => $"The rename of “{from}” to “{to}” was refused ({kind}) and nothing was changed.",
    };

    /// <summary>The last segment of a helper-side project-relative path, which always uses forward slashes (never <see cref="Path.DirectorySeparatorChar"/>).</summary>
    private static string ProjectRelativeFileName(string relativePath)
    {
        int slash = relativePath.LastIndexOf('/');
        return slash < 0 ? relativePath : relativePath[(slash + 1)..];
    }

    /// <summary>The sibling of <paramref name="relativePath"/> named <paramref name="fileName"/>, keeping its project-relative directory.</summary>
    private static string ReplaceProjectRelativeFileName(string relativePath, string fileName)
    {
        int slash = relativePath.LastIndexOf('/');
        return slash < 0 ? fileName : string.Concat(relativePath.AsSpan(0, slash + 1), fileName);
    }

    private async Task<OpenFileBinding?> SaveBoundDocumentAsync(ShellDocument document, OpenFileBinding binding)
    {
        Expected expected = binding.Sha256 is { } hash ? Expected.Hash(hash) : Expected.NewFile;
        SaveOutcome outcome = await binding.Client.SaveAsync(binding.RelativePath, document.Text, expected).ConfigureAwait(true);
        if (outcome is SaveOutcome.Conflict conflict)
        {
            await ShowProjectFileDialogAsync("Save conflict", "The file changed on disk and was not overwritten (" + conflict.Details.Kind + ").").ConfigureAwait(true);
            return null;
        }

        var saved = (SaveOutcome.Saved)outcome;
        var updated = binding with { Sha256 = saved.Receipt.Sha256 };
        _fileBindingsByDocument[document.Path] = updated;
        _shell.MarkDocumentSaved(document.Path);
        return updated;
    }

    private DocumentFilesClient GetOrStartFileClient(string root)
    {
        if (_fileClientsByRoot.TryGetValue(root, out var existing))
        {
            return existing;
        }

        string? executable = ProjectFilesToolLocator.FindExecutable();
        if (executable is null || !File.Exists(executable))
        {
            _projectFilesToolMissingReason = "flashtex-project-files.exe was not found. Build it with `cargo build --release` in crates/project-files.";
            throw new FileNotFoundException(_projectFilesToolMissingReason, executable);
        }
        var client = DocumentFilesClient.Start(executable, root, line => _shell.WorkerStatus = "file helper: " + line);
        _fileClientsByRoot.Add(root, client);
        return client;
    }

    private async Task OpenProjectRelativeFileAsync(OpenFileBinding sourceBinding, string candidate)
    {
        try
        {
            var read = await sourceBinding.Client.ReadAsync(candidate).ConfigureAwait(true);
            if (!read.Exists || read.Text is null)
            {
                return;
            }
            await _shell.OpenDocumentAsync(read.Path, read.Text).ConfigureAwait(true);
            _fileBindingsByDocument[read.Path] = new OpenFileBinding(sourceBinding.Client, sourceBinding.Root, read.Path, read.Sha256);
        }
        catch (Exception ex)
        {
            await ShowProjectFileDialogAsync("Could not open included file", ex.Message).ConfigureAwait(true);
        }
    }

    private async Task<bool> EstablishProjectRootAsync(string root)
    {
        if (string.Equals(_activeProjectRoot, root, StringComparison.OrdinalIgnoreCase))
        {
            return true;
        }
        if (_shell.Documents.Any(document => document.IsDirty))
        {
            await ShowProjectFileDialogAsync("Save your changes first", "Opening a different project would replace the current unsaved workspace.").ConfigureAwait(true);
            return false;
        }
        foreach (string path in _shell.Documents.Select(document => document.Path).ToList())
        {
            await _shell.CloseDocumentAsync(path).ConfigureAwait(true);
        }
        _fileBindingsByDocument.Clear();
        _activeProjectRoot = root;
        _shell.ProjectRoot = root;
        // A previous folder project's flat listing must not linger against a
        // new root; New File's own caller re-derives this only when the root
        // it just established is itself a folder-project root (OpenFolderAsync
        // sets both back to true/populated right after this returns).
        _isFolderProject = false;
        _folderProjectFiles = [];
        return true;
    }

    /// <summary>
    /// Save As/New File may be the operation that turns the seeded in-memory document
    /// into a real project. In that case retain the live buffer while establishing its
    /// first root; switching an already-rooted workspace through Save As is deliberately
    /// refused so a failed optimistic save can never discard an open document.
    /// </summary>
    private async Task<bool> SetProjectRootForSaveAsync(string root)
    {
        if (_activeProjectRoot is null)
        {
            _activeProjectRoot = root;
            _shell.ProjectRoot = root;
            return true;
        }
        if (string.Equals(_activeProjectRoot, root, StringComparison.OrdinalIgnoreCase))
        {
            return true;
        }
        await ShowProjectFileDialogAsync("Save As stays in this project", "Choose a location under the currently opened project, or open the other project first.").ConfigureAwait(true);
        return false;
    }

    private ShellDocument? ActiveDocumentOrNull() => _shell.Documents.FirstOrDefault(d => d.Path == _shell.ActiveDocumentPath);

    private ShellDocument ActiveDocumentOrThrow() => ActiveDocumentOrNull()
        ?? throw new InvalidOperationException("There is no active document to save.");

    private async Task ShowProjectFileDialogAsync(string title, string message)
    {
        var dialog = new ContentDialog { Title = title, Content = message, CloseButtonText = "OK", XamlRoot = Content.XamlRoot };
        await dialog.ShowAsync();
    }

    private async Task DisposeProjectFilesClientsAsync()
    {
        foreach (var client in _fileClientsByRoot.Values)
        {
            await client.DisposeAsync().ConfigureAwait(true);
        }
        _fileClientsByRoot.Clear();
        _fileBindingsByDocument.Clear();
    }
}
