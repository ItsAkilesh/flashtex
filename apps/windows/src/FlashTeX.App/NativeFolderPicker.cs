// name: NativeFolderPicker.cs
// purpose: A folder-only Win32 common-item-dialog picker (raw IFileOpenDialog +
//   FOS_PICKFOLDERS COM interop), used by MainWindow.ProjectFiles.cs's
//   OpenFolderAsync instead of Windows.Storage.Pickers.FolderPicker.
//
//   Windows.Storage.Pickers.FolderPicker was tried first, following the exact
//   InitializeWithWindow pattern OpenLatexFileAsync/SaveActiveFileAsync already
//   use successfully for FileOpenPicker/FileSavePicker. It reliably throws
//   System.Runtime.InteropServices.COMException (0x80004005, E_FAIL) from
//   PickSingleFolderAsync() in this app — confirmed via a real UI-Automation-driven
//   run: the native "Select Folder" dialog opens, navigates and closes normally
//   (the user's click on its own "Select Folder" button succeeds), but the WinRT
//   broker then fails to marshal the picked item back into this unpackaged
//   (WindowsPackageType=None) process as a StorageFolder. This matches a known
//   limitation of Windows.Storage.Pickers in unpackaged Win32 apps: FileOpenPicker/
//   FileSavePicker have a working unpackaged fallback path, but FolderPicker's does
//   not reliably resolve an arbitrary (not previously Explorer-indexed) directory
//   without a package identity's moniker cache. A freshly created temp directory —
//   exactly what this feature's own verification and any first-time "Open Folder"
//   on a new project both do — reproduces it every time.
//
//   The dialog the user actually sees either way is the same native shell folder
//   browser (Windows.Storage.Pickers.FolderPicker is itself a thin WinRT wrapper
//   over IFileOpenDialog with FOS_PICKFOLDERS): calling IFileOpenDialog directly
//   and reading the result via IShellItem::GetDisplayName(SIGDN_FILESYSPATH)
//   bypasses only the broken WinRT-to-StorageFolder marshaling step, not the UI.
// author: Claude Sonnet 5
// date: 2026-09-19

using System.Runtime.InteropServices;

namespace FlashTeX.App;

/// <summary>
/// Shows the native "pick a folder" common item dialog and returns the chosen
/// folder's file-system path, or <c>null</c> if the user cancelled.
/// </summary>
internal static class NativeFolderPicker
{
    private const uint FOS_PICKFOLDERS = 0x00000020;
    private const uint FOS_FORCEFILESYSTEM = 0x00000040;
    private const uint FOS_PATHMUSTEXIST = 0x00000800;
    private const int SIGDN_FILESYSPATH = unchecked((int)0x80058000);

    /// <summary><c>HRESULT</c> for the user closing the dialog without picking anything (<c>ERROR_CANCELLED</c> as an <c>HRESULT</c>).</summary>
    private const int HResultCancelled = unchecked((int)0x800704C7);

    /// <summary>
    /// Shows the dialog owned by <paramref name="ownerHwnd"/>. Returns the
    /// selected folder's path, or <c>null</c> if the user cancelled (never
    /// throws for a plain cancel).
    /// </summary>
    public static string? PickFolder(IntPtr ownerHwnd, string title)
    {
        var dialog = (IFileOpenDialog)new FileOpenDialogRCW();
        try
        {
            dialog.GetOptions(out uint options);
            dialog.SetOptions(options | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST);
            dialog.SetTitle(title);

            int hr = dialog.Show(ownerHwnd);
            if (hr == HResultCancelled)
            {
                return null;
            }
            Marshal.ThrowExceptionForHR(hr);

            dialog.GetResult(out IShellItem item);
            try
            {
                item.GetDisplayName(SIGDN_FILESYSPATH, out IntPtr pathPtr);
                try
                {
                    return Marshal.PtrToStringUni(pathPtr);
                }
                finally
                {
                    Marshal.FreeCoTaskMem(pathPtr);
                }
            }
            finally
            {
                Marshal.ReleaseComObject(item);
            }
        }
        finally
        {
            Marshal.ReleaseComObject(dialog);
        }
    }

    [ComImport]
    [Guid("DC1C5A9C-E88A-4dde-A5A1-60F82A20AEF7")]
    private class FileOpenDialogRCW
    {
    }

    /// <summary>
    /// The subset of <c>IFileDialog</c>/<c>IFileOpenDialog</c> this picker
    /// actually calls, declared in the interfaces' real vtable order (COM
    /// interop requires every slot before the last one used to be present,
    /// in order, even if unused here) — <c>IModalWindow::Show</c>, then
    /// <c>IFileDialog</c>'s members through <c>GetResult</c>, then
    /// <c>IFileOpenDialog</c>'s own <c>GetResults</c>/<c>GetSelectedItems</c>
    /// (declared for vtable completeness; not called).
    /// </summary>
    [ComImport]
    [Guid("d57c7288-d4ad-4768-be02-9d969532d960")]
    [InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    private interface IFileOpenDialog
    {
        // IModalWindow
        [PreserveSig]
        int Show(IntPtr parent);

        // IFileDialog
        void SetFileTypes(uint cFileTypes, IntPtr rgFilterSpec);
        void SetFileTypeIndex(uint iFileType);
        void GetFileTypeIndex(out uint piFileType);
        void Advise(IntPtr pfde, out uint pdwCookie);
        void Unadvise(uint dwCookie);
        void SetOptions(uint fos);
        void GetOptions(out uint pfos);
        void SetDefaultFolder(IShellItem psi);
        void SetFolder(IShellItem psi);
        void GetFolder(out IShellItem ppsi);
        void GetCurrentSelection(out IShellItem ppsi);
        void SetFileName([MarshalAs(UnmanagedType.LPWStr)] string pszName);
        void GetFileName([MarshalAs(UnmanagedType.LPWStr)] out string pszName);
        void SetTitle([MarshalAs(UnmanagedType.LPWStr)] string pszTitle);
        void SetOkButtonLabel([MarshalAs(UnmanagedType.LPWStr)] string pszText);
        void SetFileNameLabel([MarshalAs(UnmanagedType.LPWStr)] string pszLabel);
        void GetResult(out IShellItem ppsi);
        void AddPlace(IShellItem psi, uint fdap);
        void SetDefaultExtension([MarshalAs(UnmanagedType.LPWStr)] string pszDefaultExtension);
        void Close(int hr);
        void SetClientGuid(ref Guid guid);
        void ClearClientData();
        void SetFilter(IntPtr pFilter);

        // IFileOpenDialog
        void GetResults(out IntPtr ppenum);
        void GetSelectedItems(out IntPtr ppsai);
    }

    [ComImport]
    [Guid("43826d1e-e718-42ee-bc55-a1e261c37bfe")]
    [InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    private interface IShellItem
    {
        void BindToHandler(IntPtr pbc, ref Guid bhid, ref Guid riid, out IntPtr ppv);
        void GetParent(out IShellItem ppsi);
        void GetDisplayName(int sigdnName, out IntPtr ppszName);
        void GetAttributes(uint sfgaoMask, out uint psfgaoAttribs);
        void Compare(IShellItem psi, uint hint, out int piOrder);
    }
}
