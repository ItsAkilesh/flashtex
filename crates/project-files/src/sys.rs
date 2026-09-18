//! Directory-relative file operations (`openat`, `renameat`, `unlinkat`,
//! `mkdirat`, `fstatat`, `flock`) on top of each platform's native
//! primitives.
//!
//! Rust `std` exposes no `openat` family, so a path-string API cannot pin a
//! walked directory handle: every `std::fs` call re-resolves the whole path
//! string, and a prefix swapped for a symlink between `symlink_metadata` and
//! `open` is followed. Rather than approximate, this module talks to the
//! platform directly.
//!
//! * **POSIX** goes through the [`libc`] crate: every symbol, flag, errno
//!   value and struct layout used here comes from it, and `libc` maintains
//!   them per target (including the macOS `mode_t` width, the `$INODE64`
//!   symbol variants and each Linux architecture's `struct stat`); nothing is
//!   declared by hand. Rooted operations are enabled on macOS and on Linux
//!   with glibc or musl.
//! * **Windows** emulates the same operations with the NT native API
//!   (`NtCreateFile`/`NtSetInformationFile` with `OBJECT_ATTRIBUTES.
//!   RootDirectory` set to the pinned directory handle) via `windows-sys`.
//!   See the [`imp`] module docs on that path for the mapping, and in
//!   particular for the places Windows cannot mirror POSIX exactly: `".."` is
//!   not a resolvable relative name, so [`verify_parent`] compares canonical
//!   paths instead of `..` identity; POSIX permission bits do not exist, so
//!   `mode` arguments are ignored and [`EntryStat::mode`] carries only
//!   synthesized file-type bits; there is no `fstatat`, so
//!   [`stat_at_nofollow`] opens the entry for attributes alone; and hard
//!   links are not offered as an install primitive (see [`link_at`]), which
//!   costs nothing because [`rename_at_noreplace`] is always available there.
//!
//! Every other target gets an `io::ErrorKind::Unsupported` error, which the
//! save layer surfaces as [`Refused::Unsupported`](crate::save::Refused::Unsupported).

use std::fs::File;
use std::io;

#[cfg(any(
    target_os = "macos",
    all(target_os = "linux", any(target_env = "gnu", target_env = "musl"))
))]
mod imp {
    use std::ffi::{CStr, CString};
    use std::fs::File;
    use std::io;
    use std::mem::MaybeUninit;
    use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd};
    use std::os::raw::c_int;

    pub const SUPPORTED: bool = true;
    pub use libc::{
        EEXIST, EINVAL, ELOOP, ENOENT, ENOSYS, ENOTDIR, ENOTSUP, EOPNOTSUPP, EPERM, O_CREAT,
        O_DIRECTORY, O_EXCL, O_NOFOLLOW, O_NONBLOCK, O_RDONLY, O_RDWR, O_WRONLY,
    };

    /// The calling thread's `errno` lvalue.
    fn errno_ptr() -> *mut c_int {
        // SAFETY: both functions only return the calling thread's errno
        // location.
        #[cfg(target_os = "macos")]
        unsafe {
            libc::__error()
        }
        #[cfg(target_os = "linux")]
        unsafe {
            libc::__errno_location()
        }
    }

    fn cstr(name: impl AsRef<[u8]>) -> io::Result<CString> {
        CString::new(name.as_ref())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "NUL in path component"))
    }

    fn check(rc: c_int) -> io::Result<()> {
        if rc != 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    /// `openat(dirfd, name, flags | O_CLOEXEC | O_NONBLOCK, mode)`.
    ///
    /// `O_NONBLOCK` is always added so that opening a name which is (or was
    /// swapped to) a FIFO returns at once instead of blocking until a writer
    /// appears. It has no effect on reads or writes of regular files or on
    /// directories. For a device node it is best-effort only: whether a
    /// driver's open honours it, and what the open itself does, is up to the
    /// driver. It is a backstop, not the check: callers first classify an
    /// existing name with [`stat_at_nofollow`] and open only a regular file,
    /// then `fstat` the descriptor and refuse anything but a regular file
    /// (the name can change between the two).
    pub fn open_at(dir: &File, name: &str, flags: c_int, mode: u32) -> io::Result<File> {
        open_at_bytes(dir, name.as_bytes(), flags, mode)
    }

    /// [`open_at`] for a name that need not be UTF-8.
    pub fn open_at_bytes(dir: &File, name: &[u8], flags: c_int, mode: u32) -> io::Result<File> {
        let c = cstr(name)?;
        // SAFETY: `c` is a valid NUL-terminated string that outlives the call
        // and `dir` is an open descriptor; the variadic mode is passed as the
        // promoted `c_uint`. The returned fd is owned by exactly one `File`.
        let fd = unsafe {
            libc::openat(
                dir.as_raw_fd(),
                c.as_ptr(),
                flags | libc::O_CLOEXEC | libc::O_NONBLOCK,
                mode as libc::c_uint,
            )
        };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: `fd` is a freshly opened descriptor nothing else owns.
        Ok(unsafe { File::from_raw_fd(fd) })
    }

    pub fn rename_at(dir: &File, old: &str, new: &str) -> io::Result<()> {
        let (o, n) = (cstr(old)?, cstr(new)?);
        // SAFETY: valid C strings and an open directory descriptor.
        check(unsafe { libc::renameat(dir.as_raw_fd(), o.as_ptr(), dir.as_raw_fd(), n.as_ptr()) })
    }

    /// Renames `old` to `new` inside `dir` only if `new` does not exist
    /// (`renameatx_np(RENAME_EXCL)` on macOS, `renameat2(RENAME_NOREPLACE)`
    /// on Linux): an entry created at `new` after the caller checked is never
    /// replaced, and the call fails with `EEXIST`. Filesystems or kernels
    /// without the primitive fail with an error for which
    /// [`noreplace_unsupported`] is true; nothing was renamed then.
    pub fn rename_at_noreplace(dir: &File, old: &str, new: &str) -> io::Result<()> {
        let (o, n) = (cstr(old)?, cstr(new)?);
        let fd = dir.as_raw_fd();
        // SAFETY: valid C strings and an open directory descriptor.
        #[cfg(target_os = "macos")]
        let rc = unsafe { libc::renameatx_np(fd, o.as_ptr(), fd, n.as_ptr(), libc::RENAME_EXCL) };
        // SAFETY: as above.
        #[cfg(target_os = "linux")]
        let rc = unsafe { libc::renameat2(fd, o.as_ptr(), fd, n.as_ptr(), libc::RENAME_NOREPLACE) };
        check(rc)
    }

    /// Whether a [`rename_at_noreplace`] failure means the primitive is not
    /// available here (rather than a real rename failure). Callers must not
    /// fall back to a plain rename on it; see `save::install_new`.
    pub fn noreplace_unsupported(err: &io::Error) -> bool {
        [EINVAL, ENOSYS, ENOTSUP, EOPNOTSUPP]
            .iter()
            .any(|&c| err.raw_os_error() == Some(c))
    }

    /// `linkat(dir, old, dir, new, 0)`: a second name `new` for the entry
    /// `old` in `dir`. Fails with `EEXIST`, atomically, if `new` exists (of
    /// any type, including a dangling symlink), so it never replaces an
    /// entry. Flags are 0, so a symlink at `old` is linked, never followed.
    pub fn link_at(dir: &File, old: &str, new: &str) -> io::Result<()> {
        let (o, n) = (cstr(old)?, cstr(new)?);
        let fd = dir.as_raw_fd();
        // SAFETY: valid C strings and an open directory descriptor.
        check(unsafe { libc::linkat(fd, o.as_ptr(), fd, n.as_ptr(), 0) })
    }

    /// A new name `new` in `dir` for the open file `file` itself, not for
    /// whatever a name currently points to, so no rename or swap of the
    /// file's old name can redirect it. Fails with `EEXIST`, atomically, if
    /// `new` exists.
    ///
    /// Linux only: `linkat(fd, "", dirfd, new, AT_EMPTY_PATH)`, which older
    /// kernels allow only with `CAP_DAC_READ_SEARCH`, then
    /// `linkat(AT_FDCWD, "/proc/self/fd/N", dirfd, new, AT_SYMLINK_FOLLOW)`.
    /// Any other failure (and every call on macOS, which has neither) means
    /// "not available here"; callers fall back to a checked name-based link.
    pub fn link_fd_at(file: &File, dir: &File, new: &str) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            let n = cstr(new)?;
            // SAFETY: an open file descriptor, an empty C string, an open
            // directory descriptor and a valid C string.
            let rc = unsafe {
                libc::linkat(
                    file.as_raw_fd(),
                    c"".as_ptr(),
                    dir.as_raw_fd(),
                    n.as_ptr(),
                    libc::AT_EMPTY_PATH,
                )
            };
            let first = match check(rc) {
                Ok(()) => return Ok(()),
                Err(e) if e.raw_os_error() == Some(EEXIST) => return Err(e),
                Err(e) => e,
            };
            let proc_path = cstr(format!("/proc/self/fd/{}", file.as_raw_fd()))?;
            // SAFETY: valid C strings and an open directory descriptor.
            let rc = unsafe {
                libc::linkat(
                    libc::AT_FDCWD,
                    proc_path.as_ptr(),
                    dir.as_raw_fd(),
                    n.as_ptr(),
                    libc::AT_SYMLINK_FOLLOW,
                )
            };
            match check(rc) {
                Ok(()) => Ok(()),
                Err(e) if e.raw_os_error() == Some(EEXIST) => Err(e),
                Err(_) => Err(first),
            }
        }
        #[cfg(target_os = "macos")]
        {
            let _ = (file, dir, new);
            Err(io::Error::from_raw_os_error(ENOTSUP))
        }
    }

    /// Whether a [`link_at`] failure means the filesystem has no hard links
    /// (Linux reports `EPERM`, others `ENOTSUP`/`EOPNOTSUPP`/`ENOSYS`).
    pub fn link_unsupported(err: &io::Error) -> bool {
        [EPERM, ENOSYS, ENOTSUP, EOPNOTSUPP]
            .iter()
            .any(|&c| err.raw_os_error() == Some(c))
    }

    /// Names of the entries of the open directory `dir`, excluding `.` and
    /// `..`, as raw bytes.
    ///
    /// The directory is read with `fdopendir` on a fresh descriptor opened as
    /// `openat(dir, ".")`: the same directory object `dir` refers to, with
    /// its own read offset, never re-resolved from a path string. Entries
    /// are only named, not followed or stat'ed.
    pub fn list_dir(dir: &File) -> io::Result<Vec<Vec<u8>>> {
        let fresh = open_at_bytes(dir, b".", O_RDONLY | O_DIRECTORY, 0)?;
        let fd = fresh.into_raw_fd();
        // SAFETY: `fd` is an open directory descriptor; on success the DIR
        // stream owns it and `closedir` closes it.
        let dirp = unsafe { libc::fdopendir(fd) };
        if dirp.is_null() {
            let err = io::Error::last_os_error();
            // SAFETY: fdopendir failed, so `fd` is still ours to close.
            drop(unsafe { File::from_raw_fd(fd) });
            return Err(err);
        }
        let mut names = Vec::new();
        let result = loop {
            // SAFETY: clearing the thread's errno lets a NULL return be told
            // apart as end-of-directory or error.
            unsafe { *errno_ptr() = 0 };
            // SAFETY: `dirp` is a live DIR stream.
            let ent = unsafe { libc::readdir(dirp) };
            if ent.is_null() {
                // SAFETY: as above.
                let errno = unsafe { *errno_ptr() };
                break if errno == 0 {
                    Ok(())
                } else {
                    Err(io::Error::from_raw_os_error(errno))
                };
            }
            // SAFETY: `ent` points at a `libc::dirent` whose `d_name` is
            // NUL-terminated; it stays valid until the next `readdir` on this
            // stream, and is copied before that.
            let name = unsafe { CStr::from_ptr((*ent).d_name.as_ptr()) };
            let name = name.to_bytes();
            if name != b"." && name != b".." {
                names.push(name.to_vec());
            }
        };
        // SAFETY: `dirp` is a live DIR stream, closed exactly once.
        unsafe { libc::closedir(dirp) };
        result.map(|()| names)
    }

    pub fn unlink_at(dir: &File, name: &str) -> io::Result<()> {
        let c = cstr(name)?;
        // SAFETY: valid C string and an open directory descriptor.
        check(unsafe { libc::unlinkat(dir.as_raw_fd(), c.as_ptr(), 0) })
    }

    pub fn mkdir_at(dir: &File, name: &str, mode: u32) -> io::Result<()> {
        let c = cstr(name)?;
        // SAFETY: valid C string and an open directory descriptor.
        check(unsafe { libc::mkdirat(dir.as_raw_fd(), c.as_ptr(), mode as libc::mode_t) })
    }

    /// `fstatat(dirfd, name, AT_SYMLINK_NOFOLLOW)`: classifies the entry
    /// `name` of the open directory `dir` without following a symlink and
    /// without opening it (so a device's open side effects never run).
    #[allow(clippy::unnecessary_cast)] // field widths differ per target
    pub fn stat_at_nofollow(dir: &File, name: &[u8]) -> io::Result<super::EntryStat> {
        let c = cstr(name)?;
        let mut st = MaybeUninit::<libc::stat>::uninit();
        // SAFETY: valid C string, an open directory descriptor, and a
        // `libc::stat` buffer of the target's own layout.
        check(unsafe {
            libc::fstatat(
                dir.as_raw_fd(),
                c.as_ptr(),
                st.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        })?;
        // SAFETY: `fstatat` returned 0, so it filled the whole struct.
        let st = unsafe { st.assume_init() };
        Ok(super::EntryStat {
            dev: st.st_dev as u64,
            ino: st.st_ino as u64,
            mode: st.st_mode as u32,
        })
    }

    /// Fills `buf` from the OS cryptographic random source:
    /// `arc4random_buf` on macOS; on Linux the `getrandom` system call
    /// (called directly, so no minimum libc version is needed), falling back
    /// to reading `/dev/urandom` on kernels without it. Errors if no source
    /// is available; callers fail rather than use a predictable value.
    pub fn random_bytes(buf: &mut [u8]) -> io::Result<()> {
        #[cfg(target_os = "macos")]
        {
            // SAFETY: `buf` is writable for `buf.len()` bytes;
            // `arc4random_buf` cannot fail.
            unsafe { libc::arc4random_buf(buf.as_mut_ptr().cast(), buf.len()) };
            Ok(())
        }
        #[cfg(target_os = "linux")]
        {
            let mut filled = 0;
            while filled < buf.len() {
                let rest = &mut buf[filled..];
                // SAFETY: `rest` is writable for `rest.len()` bytes.
                let n = unsafe {
                    libc::syscall(
                        libc::SYS_getrandom,
                        rest.as_mut_ptr(),
                        rest.len(),
                        0 as libc::c_uint,
                    )
                };
                if n > 0 {
                    filled += n as usize;
                    continue;
                }
                let err = io::Error::last_os_error();
                if err.raw_os_error() == Some(libc::EINTR) {
                    continue;
                }
                if err.raw_os_error() == Some(libc::ENOSYS) {
                    use std::io::Read;
                    return File::open("/dev/urandom")?.read_exact(&mut buf[filled..]);
                }
                return Err(err);
            }
            Ok(())
        }
    }

    /// Opens `path` as a directory with plain `std`, for the one bootstrap
    /// open that has no directory handle to be relative to.
    ///
    /// `O_DIRECTORY | O_NONBLOCK` is carried so a FIFO at that path is
    /// refused instead of blocking the open.
    pub fn open_dir_std(path: &std::path::Path) -> io::Result<File> {
        use std::os::unix::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(O_DIRECTORY | O_NONBLOCK)
            .open(path)
    }

    /// Whether `child` is a direct entry of `dir`.
    ///
    /// The POSIX answer is exact: open `child`'s `..` and compare it with
    /// `dir` by device/inode, so a directory swapped or moved out from under
    /// the walk is caught.
    pub fn verify_parent(dir: &File, child: &File) -> io::Result<bool> {
        use std::os::unix::fs::MetadataExt;
        let up = open_at(child, "..", O_RDONLY | O_DIRECTORY, 0)?;
        let (up, dir) = (up.metadata()?, dir.metadata()?);
        Ok(up.dev() == dir.dev() && up.ino() == dir.ino())
    }

    /// Advisory exclusive lock; `Ok(false)` when another open file
    /// description (any process, or another handle in this one) holds it.
    pub fn try_lock_exclusive(file: &File) -> io::Result<bool> {
        // SAFETY: `file` is an open descriptor.
        let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
        if rc == 0 {
            return Ok(true);
        }
        let err = io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::EWOULDBLOCK) {
            Ok(false)
        } else {
            Err(err)
        }
    }

    pub fn unlock(file: &File) {
        // SAFETY: `file` is an open descriptor; failure is irrelevant at drop.
        unsafe {
            libc::flock(file.as_raw_fd(), libc::LOCK_UN);
        }
    }
}

/// Windows `openat` emulation over the NT native API.
///
/// # Why the NT API and not Win32
///
/// Win32 has no directory-relative open at all, and
/// `SetFileInformationByHandle(FileRenameInfo)` — the API usually recommended
/// for this — was measured on Windows 11 26300 to reject a non-NULL
/// `RootDirectory` with `ERROR_INVALID_PARAMETER`. `NtCreateFile` and
/// `NtSetInformationFile` accept `OBJECT_ATTRIBUTES.RootDirectory` /
/// `FILE_RENAME_INFORMATION.RootDirectory` and are what actually pins the
/// walked directory the way `openat`/`renameat` do.
///
/// # Where Windows cannot mirror POSIX
///
/// * **`".."` is not openable relative to a directory handle.**
///   `NtCreateFile` with `RootDirectory` set and `ObjectName` `".."` fails
///   with `STATUS_OBJECT_NAME_INVALID` — the object manager rejects the name
///   rather than walking up (measured, not assumed). [`verify_parent`]
///   therefore proves containment with `GetFinalPathNameByHandleW` instead:
///   the child's canonical path must be the parent's canonical path plus
///   exactly one more component. That is a *stronger* statement than the
///   POSIX `..` identity check (it pins the whole chain to the volume root),
///   and it is equally race-aware: a directory moved since it was opened
///   reports its new path and is refused.
/// * **POSIX permission bits do not exist.** Every `mode` argument is
///   ignored; new files and directories inherit ACLs from their parent, which
///   is the Windows equivalent of the umask behaviour the POSIX path relies
///   on. No fake mode-to-ACL translation is attempted.
/// * **Renaming over a file that another handle has open** fails with
///   `ERROR_ACCESS_DENIED` under the classic `FileRenameInformation`. The
///   POSIX-semantics `FileRenameInformationEx` (Windows 10 1709+) succeeds,
///   so it is tried first and the classic form is only a fallback.
///
/// Sharing: every open requests `FILE_SHARE_READ | FILE_SHARE_WRITE |
/// FILE_SHARE_DELETE`. Without share-delete, Windows would refuse to rename
/// over or unlink any file this crate itself still had open — which is every
/// file it is in the middle of saving.
#[cfg(windows)]
mod imp {
    use std::fs::File;
    use std::io;
    use std::os::raw::c_int;
    use std::os::windows::io::{AsRawHandle, FromRawHandle, RawHandle};

    use windows_sys::Wdk::Foundation::OBJECT_ATTRIBUTES;
    use windows_sys::Wdk::Storage::FileSystem::{
        FILE_CREATE, FILE_DIRECTORY_FILE, FILE_OPEN, FILE_OPEN_IF, FILE_OPEN_REPARSE_POINT,
        FILE_SYNCHRONOUS_IO_NONALERT, FileRenameInformation, FileRenameInformationEx, NtCreateFile,
        NtSetInformationFile,
    };
    use windows_sys::Win32::Foundation::{
        ERROR_ACCESS_DENIED, ERROR_ALREADY_EXISTS, ERROR_CANT_RESOLVE_FILENAME, ERROR_DIRECTORY,
        ERROR_FILE_NOT_FOUND, ERROR_INVALID_FUNCTION, ERROR_LOCK_VIOLATION, ERROR_NO_MORE_FILES,
        ERROR_NOT_SUPPORTED, HANDLE, NTSTATUS, OBJ_CASE_INSENSITIVE, RtlNtStatusToDosError,
        STATUS_DELETE_PENDING, STATUS_INVALID_DEVICE_REQUEST, STATUS_INVALID_INFO_CLASS,
        STATUS_INVALID_PARAMETER, STATUS_NOT_A_DIRECTORY, STATUS_NOT_SUPPORTED,
        STATUS_OBJECT_NAME_COLLISION, STATUS_OBJECT_NAME_NOT_FOUND, STATUS_OBJECT_PATH_NOT_FOUND,
        STATUS_REPARSE_POINT_ENCOUNTERED, UNICODE_STRING,
    };
    use windows_sys::Win32::Security::Cryptography::ProcessPrng;
    use windows_sys::Win32::Storage::FileSystem::{
        DELETE, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_NORMAL, FILE_ATTRIBUTE_REPARSE_POINT,
        FILE_ATTRIBUTE_TAG_INFO, FILE_DISPOSITION_FLAG_DELETE,
        FILE_DISPOSITION_FLAG_POSIX_SEMANTICS, FILE_DISPOSITION_INFO, FILE_DISPOSITION_INFO_EX,
        FILE_FULL_DIR_INFO, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_ID_INFO,
        FILE_LIST_DIRECTORY, FILE_READ_ATTRIBUTES, FILE_RENAME_INFO, FILE_SHARE_DELETE,
        FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_TRAVERSE, FILE_WRITE_DATA, FileAttributeTagInfo,
        FileDispositionInfo, FileDispositionInfoEx, FileFullDirectoryInfo,
        FileFullDirectoryRestartInfo, FileIdInfo, GetFileInformationByHandleEx,
        GetFinalPathNameByHandleW, LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY, LockFileEx,
        SYNCHRONIZE, SetFileInformationByHandle, UnlockFileEx, VOLUME_NAME_DOS,
    };
    use windows_sys::Win32::System::IO::{IO_STATUS_BLOCK, OVERLAPPED};

    pub const SUPPORTED: bool = true;

    // Module-local sentinel flags. Nothing outside this module inspects their
    // numeric value — `save.rs` only ORs them together and hands them back to
    // `open_at` — so they need not match any POSIX numbering. Unlike POSIX,
    // `O_RDONLY` is a real bit rather than 0, so access intent is always
    // explicit.
    pub const O_RDONLY: c_int = 0x0001;
    pub const O_WRONLY: c_int = 0x0002;
    pub const O_RDWR: c_int = 0x0004;
    pub const O_CREAT: c_int = 0x0008;
    pub const O_EXCL: c_int = 0x0010;
    pub const O_NOFOLLOW: c_int = 0x0020;
    pub const O_DIRECTORY: c_int = 0x0040;

    // "errno" sentinels. `save.rs` compares `io::Error::raw_os_error()`
    // against these through `errno_is`, and on Windows a raw OS error *is* a
    // Win32 code, so these are the canonical Win32 codes this module
    // normalizes onto (see `status_to_error`). Normalization matters: the NT
    // layer distinguishes name-not-found from path-not-found, but `save.rs`
    // matches a single value, so both collapse to `ENOENT`.
    pub const ELOOP: i32 = ERROR_CANT_RESOLVE_FILENAME as i32; // 1921
    pub const ENOENT: i32 = ERROR_FILE_NOT_FOUND as i32; // 2
    pub const EEXIST: i32 = ERROR_ALREADY_EXISTS as i32; // 183
    pub const ENOTDIR: i32 = ERROR_DIRECTORY as i32; // 267
    /// "The operation is not implemented here." Reported by [`link_at`] and
    /// [`link_fd_at`], and recognized by [`link_unsupported`] /
    /// [`noreplace_unsupported`].
    pub const ENOTSUP: i32 = ERROR_NOT_SUPPORTED as i32; // 50
    /// The Win32 code a volume returns for a request its driver does not
    /// implement; the closest counterpart of the `EPERM` POSIX filesystems
    /// report for "no hard links here".
    pub const EPERM: i32 = ERROR_INVALID_FUNCTION as i32; // 1

    /// `IsReparseTagNameSurrogate`: the tag stands in for another name, i.e.
    /// it is a symlink or a junction/mount point rather than a filter's
    /// storage trick (OneDrive placeholders, dedup, WIM-backed files).
    const REPARSE_TAG_NAME_SURROGATE: u32 = 0x2000_0000;

    /// `FileRenameInformationEx` flags.
    const RENAME_REPLACE_IF_EXISTS: u32 = 0x1;
    const RENAME_POSIX_SEMANTICS: u32 = 0x2;
    const RENAME_IGNORE_READONLY_ATTRIBUTE: u32 = 0x40;
    /// The flag set a replacing rename uses; [`rename_at_noreplace`] clears
    /// `RENAME_REPLACE_IF_EXISTS` from it.
    const RENAME_EX_FLAGS: u32 =
        RENAME_REPLACE_IF_EXISTS | RENAME_POSIX_SEMANTICS | RENAME_IGNORE_READONLY_ATTRIBUTE;

    const SHARE_ALL: u32 = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;

    /// Maps an `NTSTATUS` onto the `io::Error` whose `raw_os_error()` the
    /// `ELOOP`/`ENOENT`/`EEXIST`/`ENOTDIR` sentinels above compare equal to.
    /// Statuses without a POSIX counterpart keep whatever Win32 code
    /// `RtlNtStatusToDosError` assigns.
    fn status_to_error(status: NTSTATUS) -> io::Error {
        let code = match status {
            STATUS_OBJECT_NAME_NOT_FOUND | STATUS_OBJECT_PATH_NOT_FOUND => ENOENT,
            // A name whose last handle is closing is gone as far as any
            // caller is concerned; POSIX would simply report it missing.
            STATUS_DELETE_PENDING => ENOENT,
            STATUS_OBJECT_NAME_COLLISION => EEXIST,
            STATUS_NOT_A_DIRECTORY => ENOTDIR,
            STATUS_REPARSE_POINT_ENCOUNTERED => ELOOP,
            // SAFETY: a pure lookup table in ntdll with no memory operands.
            other => (unsafe { RtlNtStatusToDosError(other) }) as i32,
        };
        io::Error::from_raw_os_error(code)
    }

    fn loop_error() -> io::Error {
        io::Error::from_raw_os_error(ELOOP)
    }

    /// One `NtCreateFile` against `dir` (or an absolute NT path when `dir` is
    /// `None`), with no reparse-point interpretation.
    fn nt_create(
        dir: Option<&File>,
        name: &str,
        access: u32,
        disposition: u32,
        options: u32,
        attributes: u32,
    ) -> io::Result<File> {
        let mut wide: Vec<u16> = name.encode_utf16().collect();
        let len = match u16::try_from(wide.len() * 2) {
            Ok(n) => n,
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "path component is too long for a UNICODE_STRING",
                ));
            }
        };
        let mut object_name = UNICODE_STRING {
            Length: len,
            MaximumLength: len,
            Buffer: wide.as_mut_ptr(),
        };
        let mut attrs = OBJECT_ATTRIBUTES {
            Length: size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: dir.map_or(std::ptr::null_mut(), |d| d.as_raw_handle() as HANDLE),
            ObjectName: &mut object_name,
            // NTFS is case-insensitive; matching its own lookup rules keeps a
            // differently-cased spelling from looking like a missing file.
            Attributes: OBJ_CASE_INSENSITIVE,
            SecurityDescriptor: std::ptr::null(),
            SecurityQualityOfService: std::ptr::null(),
        };
        let mut iosb: IO_STATUS_BLOCK = unsafe { std::mem::zeroed() };
        let mut handle: HANDLE = std::ptr::null_mut();
        // SAFETY: `wide`, `object_name`, `attrs`, `iosb` and `handle` all
        // outlive the call; `dir`, when given, is an open handle. On success
        // the kernel writes exactly one handle into `handle`, which becomes
        // owned by exactly one `File`.
        let status = unsafe {
            NtCreateFile(
                &mut handle,
                access,
                &mut attrs,
                &mut iosb,
                std::ptr::null(),
                attributes,
                SHARE_ALL,
                disposition,
                // Synchronous so the resulting handle behaves like a POSIX
                // fd: `std`'s `Read`/`Write`/`sync_all` all assume a
                // synchronous handle with a kernel-maintained file position.
                options | FILE_SYNCHRONOUS_IO_NONALERT,
                std::ptr::null(),
                0,
            )
        };
        if status < 0 {
            return Err(status_to_error(status));
        }
        Ok(unsafe { File::from_raw_handle(handle as RawHandle) })
    }

    /// `(FileAttributes, ReparseTag)` for an open handle.
    fn attribute_tag(file: &File) -> io::Result<(u32, u32)> {
        let mut info: FILE_ATTRIBUTE_TAG_INFO = unsafe { std::mem::zeroed() };
        // SAFETY: `info` is a live, correctly sized `FILE_ATTRIBUTE_TAG_INFO`
        // and `file` is an open handle.
        let ok = unsafe {
            GetFileInformationByHandleEx(
                file.as_raw_handle() as HANDLE,
                FileAttributeTagInfo,
                (&raw mut info).cast(),
                size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok((info.FileAttributes, info.ReparseTag))
    }

    /// Volume serial number and 128-bit file ID of an open handle — the
    /// closest Windows analogue of POSIX `(st_dev, st_ino)`.
    ///
    /// `FILE_ID_INFO` is used rather than `BY_HANDLE_FILE_INFORMATION`'s
    /// 64-bit `nFileIndex*` pair because ReFS file IDs do not fit in 64 bits;
    /// on ReFS the truncated form is not unique, which would let a swapped
    /// file pass the post-rename identity check this feeds.
    pub fn file_id(file: &File) -> io::Result<(u64, u128)> {
        let mut info: FILE_ID_INFO = unsafe { std::mem::zeroed() };
        // SAFETY: as in `attribute_tag`.
        let ok = unsafe {
            GetFileInformationByHandleEx(
                file.as_raw_handle() as HANDLE,
                FileIdInfo,
                (&raw mut info).cast(),
                size_of::<FILE_ID_INFO>() as u32,
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok((
            info.VolumeSerialNumber,
            u128::from_le_bytes(info.FileId.Identifier),
        ))
    }

    /// Canonical `\\?\`-prefixed path of an open handle.
    fn final_path(file: &File) -> io::Result<Vec<u16>> {
        let mut buf = vec![0u16; 512];
        loop {
            // SAFETY: `buf` is a live buffer of exactly the declared length.
            let n = unsafe {
                GetFinalPathNameByHandleW(
                    file.as_raw_handle() as HANDLE,
                    buf.as_mut_ptr(),
                    buf.len() as u32,
                    VOLUME_NAME_DOS,
                )
            };
            if n == 0 {
                return Err(io::Error::last_os_error());
            }
            let n = n as usize;
            if n < buf.len() {
                buf.truncate(n);
                return Ok(buf);
            }
            buf.resize(n + 1, 0);
        }
    }

    /// Opens `path` as a directory with plain `std`, for the one bootstrap
    /// open that has no directory handle to be relative to.
    ///
    /// `File::open` alone cannot open a directory on Windows; that needs
    /// `FILE_FLAG_BACKUP_SEMANTICS`, which is what this adds. The handle is
    /// only ever used as an `OBJECT_ATTRIBUTES.RootDirectory`, so it needs no
    /// more than read access.
    pub fn open_dir_std(path: &std::path::Path) -> io::Result<File> {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
        std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
            .open(path)
    }

    /// Whether `child` is a direct entry of `dir`.
    ///
    /// Windows has no openable `".."` (see the module docs), so this compares
    /// canonical paths: `child`'s must be `dir`'s plus exactly one more
    /// component. Comparison is case-insensitive because NTFS is, and both
    /// strings come from the kernel's own canonical spelling.
    pub fn verify_parent(dir: &File, child: &File) -> io::Result<bool> {
        const SEP: u16 = b'\\' as u16;
        /// NTFS upcases with the full Unicode table, but both operands come
        /// from the kernel's own canonical spelling of the same directory, so
        /// they differ at most in the caller-supplied ASCII case.
        fn lower(c: u16) -> u16 {
            match u8::try_from(c) {
                Ok(b) => b.to_ascii_lowercase() as u16,
                Err(_) => c,
            }
        }

        let (parent, child) = (final_path(dir)?, final_path(child)?);
        // A volume root ("\\?\C:\") already ends in the separator.
        let parent = match parent.last() {
            Some(&c) if c == SEP => &parent[..parent.len() - 1],
            _ => &parent[..],
        };
        let Some((prefix, rest)) = child.split_at_checked(parent.len()) else {
            return Ok(false);
        };
        if !prefix
            .iter()
            .zip(parent)
            .all(|(&a, &b)| a == b || lower(a) == lower(b))
        {
            return Ok(false);
        }
        // Exactly one further component: a leading separator and no other.
        Ok(matches!(rest.split_first(),
            Some((&sep, name))
                if sep == SEP && !name.is_empty() && !name.contains(&SEP)))
    }

    /// Directory-relative open. See the module docs for the flag mapping.
    ///
    /// `mode` is ignored: Windows has no POSIX permission bits, and a new
    /// file or directory inherits its parent's ACL.
    pub fn open_at(dir: &File, name: &str, flags: c_int, mode: u32) -> io::Result<File> {
        let _ = mode;
        let wants_dir = flags & O_DIRECTORY != 0;
        let nofollow = flags & O_NOFOLLOW != 0;

        let mut access = SYNCHRONIZE | FILE_READ_ATTRIBUTES;
        if flags & (O_RDONLY | O_RDWR) != 0 {
            access |= FILE_GENERIC_READ;
        }
        if flags & (O_WRONLY | O_RDWR) != 0 {
            access |= FILE_GENERIC_WRITE;
        }
        if wants_dir {
            access |= FILE_LIST_DIRECTORY | FILE_TRAVERSE;
        }

        let disposition = match (flags & O_CREAT != 0, flags & O_EXCL != 0) {
            (true, true) => FILE_CREATE,
            (true, false) => FILE_OPEN_IF,
            (false, _) => FILE_OPEN,
        };
        // Deliberately *not* `FILE_NON_DIRECTORY_FILE` when `O_DIRECTORY` is
        // absent: POSIX lets `open(O_RDONLY)` succeed on a directory and
        // leaves it to the caller to reject it, which is exactly what
        // `save.rs`'s `NotARegularFile` refusal does.
        let mut options = if wants_dir { FILE_DIRECTORY_FILE } else { 0 };
        if nofollow {
            options |= FILE_OPEN_REPARSE_POINT;
        }
        let attributes = if wants_dir { 0 } else { FILE_ATTRIBUTE_NORMAL };

        // A directory handle that `save.rs` will later `fsync` (`sync_all` ->
        // `FlushFileBuffers`) needs write access; without `FILE_WRITE_DATA`
        // the flush fails with `ERROR_ACCESS_DENIED` (measured). Ask for it,
        // but never let a read-only project directory become unopenable.
        let file = match nt_create(
            Some(dir),
            name,
            if wants_dir {
                access | FILE_WRITE_DATA
            } else {
                access
            },
            disposition,
            options,
            attributes,
        ) {
            Ok(f) => f,
            // Only the extra `FILE_WRITE_DATA` can turn an otherwise fine
            // open into a denial, so retry without it on exactly that error
            // and let every other failure (a missing component above all,
            // which `walk` hits routinely) report itself immediately.
            Err(e)
                if wants_dir && e.raw_os_error() == Some(ERROR_ACCESS_DENIED as i32) =>
            {
                nt_create(Some(dir), name, access, disposition, options, attributes)?
            }
            Err(e) => return Err(e),
        };

        if !nofollow {
            return Ok(file);
        }
        // `FILE_OPEN_REPARSE_POINT` opened whatever reparse point sits here
        // instead of following it. Refuse the ones that redirect to another
        // name (symlinks, junctions) — that is what `O_NOFOLLOW` means — but
        // do not mistake a storage filter's reparse point (OneDrive
        // placeholders, dedup) for a symlink; reopen those normally so the
        // filter can serve real content.
        let (file_attributes, tag) = attribute_tag(&file)?;
        if file_attributes & FILE_ATTRIBUTE_REPARSE_POINT == 0 {
            return Ok(file);
        }
        if tag & REPARSE_TAG_NAME_SURROGATE != 0 {
            return Err(loop_error());
        }
        drop(file);
        nt_create(
            Some(dir),
            name,
            access,
            // It demonstrably exists, so never re-run a create disposition.
            FILE_OPEN,
            options & !FILE_OPEN_REPARSE_POINT,
            attributes,
        )
    }

    /// Opens `name` in `dir` for deletion, without following or classifying a
    /// reparse point — `unlinkat`/`renameat` act on the link itself.
    fn open_for_delete(dir: &File, name: &str) -> io::Result<File> {
        nt_create(
            Some(dir),
            name,
            SYNCHRONIZE | DELETE | FILE_READ_ATTRIBUTES,
            FILE_OPEN,
            FILE_OPEN_REPARSE_POINT,
            0,
        )
    }

    /// Builds a variable-length `FILE_RENAME_INFO` naming `new` under `dir`.
    ///
    /// The buffer is over-aligned through `Vec<u64>` because the structure
    /// contains a `HANDLE`, and sized from `offset_of!(FileName)` rather than
    /// `size_of` — the trailing `WCHAR[1]` sits four bytes before the padded
    /// end of the struct on 64-bit, so `size_of` would place the name too far
    /// along and the kernel would read two leading NULs as the name.
    fn rename_info(dir: &File, new: &str, flags: Option<u32>, replace: bool) -> (Vec<u64>, u32) {
        let name_offset = std::mem::offset_of!(FILE_RENAME_INFO, FileName);
        let wide: Vec<u16> = new.encode_utf16().collect();
        let name_bytes = wide.len() * 2;
        let total = name_offset + name_bytes;
        let mut buf = vec![0u64; total.div_ceil(size_of::<u64>()) + 1];
        // SAFETY: `buf` is zeroed, at least `total` bytes long and aligned to
        // `u64`, which covers `FILE_RENAME_INFO`'s alignment; the name is
        // copied entirely within it.
        unsafe {
            let base = buf.as_mut_ptr().cast::<u8>();
            let info = base.cast::<FILE_RENAME_INFO>();
            match flags {
                Some(f) => (*info).Anonymous.Flags = f,
                None => (*info).Anonymous.ReplaceIfExists = replace,
            }
            (*info).RootDirectory = dir.as_raw_handle() as HANDLE;
            (*info).FileNameLength = name_bytes as u32;
            std::ptr::copy_nonoverlapping(wide.as_ptr().cast::<u8>(), base.add(name_offset), name_bytes);
        }
        (buf, total as u32)
    }

    fn set_info(file: &File, buf: &[u64], len: u32, class: i32) -> NTSTATUS {
        let mut iosb: IO_STATUS_BLOCK = unsafe { std::mem::zeroed() };
        // SAFETY: `buf` holds at least `len` initialized bytes laid out for
        // `class`, and `file` is an open handle.
        unsafe {
            NtSetInformationFile(
                file.as_raw_handle() as HANDLE,
                &mut iosb,
                buf.as_ptr().cast(),
                len,
                class,
            )
        }
    }

    /// Atomic rename of `old` over `new`, both relative to `dir`.
    ///
    /// `FileRenameInformationEx`'s POSIX semantics are tried first: unlike
    /// the classic form they replace a target that another handle still has
    /// open, which is the normal case here (a reader, or this crate's own
    /// pre-rename observation). The classic form is the fallback for
    /// pre-1709 Windows and filesystems that reject the new class.
    pub fn rename_at(dir: &File, old: &str, new: &str) -> io::Result<()> {
        rename_impl(dir, old, new, true)
    }

    /// Renames `old` to `new` inside `dir` only if `new` does not exist.
    ///
    /// This is the same call as [`rename_at`] with
    /// `FILE_RENAME_REPLACE_IF_EXISTS` cleared, so an entry created at `new`
    /// after the caller checked is never replaced: the kernel reports
    /// `STATUS_OBJECT_NAME_COLLISION`, normalized here to [`EEXIST`]. The
    /// classic `FileRenameInformation` fallback carries the same guarantee
    /// through `ReplaceIfExists = false`, so unlike POSIX (where
    /// `renameat2`/`renameatx_np` can be missing) no Windows filesystem
    /// leaves this primitive unavailable, and [`noreplace_unsupported`] is
    /// never true here.
    pub fn rename_at_noreplace(dir: &File, old: &str, new: &str) -> io::Result<()> {
        rename_impl(dir, old, new, false)
    }

    fn rename_impl(dir: &File, old: &str, new: &str, replace: bool) -> io::Result<()> {
        let src = open_for_delete(dir, old)?;
        let ex_flags = if replace {
            RENAME_EX_FLAGS
        } else {
            RENAME_EX_FLAGS & !RENAME_REPLACE_IF_EXISTS
        };
        let (buf, len) = rename_info(dir, new, Some(ex_flags), replace);
        let status = set_info(&src, &buf, len, FileRenameInformationEx);
        if status >= 0 {
            return Ok(());
        }
        // Fall back only when the *class* was rejected. Any other failure is
        // a real one — a delete-pending target, a denial, an existing target
        // under `!replace` — and the classic form would only fail again,
        // reporting a less accurate reason.
        if !matches!(
            status,
            STATUS_INVALID_PARAMETER
                | STATUS_INVALID_INFO_CLASS
                | STATUS_INVALID_DEVICE_REQUEST
                | STATUS_NOT_SUPPORTED
        ) {
            return Err(status_to_error(status));
        }
        let (buf, len) = rename_info(dir, new, None, replace);
        let status = set_info(&src, &buf, len, FileRenameInformation);
        if status < 0 {
            return Err(status_to_error(status));
        }
        Ok(())
    }

    /// Whether a [`rename_at_noreplace`] failure means the primitive is not
    /// available here (rather than a real rename failure).
    ///
    /// Windows' own [`rename_at_noreplace`] never reports this: it falls
    /// back internally to the classic `FileRenameInformation`, which every
    /// filesystem implements. The predicate still recognizes the same
    /// "not implemented here" codes it does on POSIX, so an injected or
    /// driver-reported one is classified the same way everywhere.
    pub fn noreplace_unsupported(err: &io::Error) -> bool {
        [ENOTSUP, EPERM]
            .iter()
            .any(|&c| err.raw_os_error() == Some(c))
    }

    /// Not offered on Windows; always fails with [`ENOTSUP`], for which
    /// [`link_unsupported`] is true.
    ///
    /// `save::install_new` uses a hard link only as the fallback for POSIX
    /// filesystems whose kernel has no no-replace rename. Windows always has
    /// one ([`rename_at_noreplace`]), so the fallback is dead weight here,
    /// and the honest way to say so is an error the caller already knows how
    /// to interpret rather than an emulation. (`CreateHardLinkW` resolves a
    /// path string instead of a pinned directory handle, so it would not
    /// preserve the rooted guarantee anyway; `FILE_LINK_INFORMATION` would,
    /// but nothing would exercise it.)
    pub fn link_at(_: &File, _: &str, _: &str) -> io::Result<()> {
        Err(io::Error::from_raw_os_error(ENOTSUP))
    }

    /// Not offered on Windows; see [`link_at`]. A plain failure here makes
    /// `save::install_new` fall through to its name-based link attempt,
    /// which reports [`ENOTSUP`] in turn.
    pub fn link_fd_at(_: &File, _: &File, _: &str) -> io::Result<()> {
        Err(io::Error::from_raw_os_error(ENOTSUP))
    }

    /// Whether a [`link_at`] failure means hard links are unavailable here.
    /// Always true for what [`link_at`] itself returns.
    pub fn link_unsupported(err: &io::Error) -> bool {
        [ENOTSUP, EPERM]
            .iter()
            .any(|&c| err.raw_os_error() == Some(c))
    }

    /// [`open_at`] for a name that is not already a `&str`.
    ///
    /// Windows paths are UTF-16 and this crate's [`crate::ProjectPath`] is
    /// UTF-8, so the bytes must be valid UTF-8; the one caller that does not
    /// start from a `&str` ([`super::open_dir_nofollow`]) only ever passes a
    /// name it obtained through `OsStr::to_str`.
    pub fn open_at_bytes(dir: &File, name: &[u8], flags: c_int, mode: u32) -> io::Result<File> {
        open_at(dir, str_name(name)?, flags, mode)
    }

    fn str_name(name: &[u8]) -> io::Result<&str> {
        std::str::from_utf8(name).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "a Windows file name must be valid UTF-8 here",
            )
        })
    }

    /// Windows' stand-in for `fstatat(AT_SYMLINK_NOFOLLOW)`.
    ///
    /// There is no query-by-name-relative-to-a-handle API, so the entry is
    /// opened relative to `dir` — with `FILE_OPEN_REPARSE_POINT`, so a
    /// symlink or junction is described rather than followed, and with
    /// `FILE_READ_ATTRIBUTES` as the only access, so no read or write
    /// permission is required. The [`super::EntryStat::mode`] returned
    /// carries synthesized POSIX *type* bits only (see
    /// [`super::S_IFMT`]); Windows has no permission bits to report.
    ///
    /// **Difference from POSIX.** `fstatat` never opens, so it cannot be
    /// refused by a sharing mode and cannot run a device's open side
    /// effects. This does open, so an entry another process holds with a
    /// restrictive sharing mode reports `ERROR_SHARING_VIOLATION` where
    /// POSIX would have reported the entry's type. It is still the pinned,
    /// no-follow, attributes-only observation the callers need, and it never
    /// reads or writes content.
    pub fn stat_at_nofollow(dir: &File, name: &[u8]) -> io::Result<super::EntryStat> {
        let file = nt_create(
            Some(dir),
            str_name(name)?,
            SYNCHRONIZE | FILE_READ_ATTRIBUTES,
            FILE_OPEN,
            FILE_OPEN_REPARSE_POINT,
            0,
        )?;
        let (attributes, tag) = attribute_tag(&file)?;
        let mode = if attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
            && tag & REPARSE_TAG_NAME_SURROGATE != 0
        {
            // A symlink or a junction: both redirect to another name, which
            // is exactly what POSIX calls a symlink for these callers.
            super::S_IFLNK
        } else if attributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
            super::S_IFDIR
        } else {
            super::S_IFREG
        };
        let (volume_serial, file_index) = file_id(&file)?;
        Ok(super::EntryStat {
            volume_serial,
            file_index,
            mode,
        })
    }

    /// Names of the entries of the open directory `dir`, excluding `.` and
    /// `..`, as UTF-8 bytes.
    ///
    /// `GetFileInformationByHandleEx(FileFullDirectoryInfo)` enumerates the
    /// directory `dir` itself refers to, never a re-resolved path string, so
    /// a parent renamed away after it was walked is not enumerated. Names
    /// that are not representable as UTF-8 (unpaired surrogates) are skipped
    /// rather than lossily transcoded: the only caller matches them against
    /// UTF-8 [`crate::ProjectPath`]s, which such a name can never equal.
    pub fn list_dir(dir: &File) -> io::Result<Vec<Vec<u8>>> {
        // Enough for a few dozen entries per call; the loop refills it.
        let mut buf = vec![0u64; 8192 / size_of::<u64>()];
        let mut class = FileFullDirectoryRestartInfo;
        let mut names = Vec::new();
        loop {
            // SAFETY: `buf` is a live, `u64`-aligned buffer of exactly the
            // declared byte length, and `dir` is an open directory handle
            // carrying `FILE_LIST_DIRECTORY` (see `open_at`).
            let ok = unsafe {
                GetFileInformationByHandleEx(
                    dir.as_raw_handle() as HANDLE,
                    class,
                    buf.as_mut_ptr().cast(),
                    (buf.len() * size_of::<u64>()) as u32,
                )
            };
            if ok == 0 {
                let err = io::Error::last_os_error();
                if err.raw_os_error() == Some(ERROR_NO_MORE_FILES as i32) {
                    return Ok(names);
                }
                return Err(err);
            }
            class = FileFullDirectoryInfo;
            let mut offset = 0usize;
            loop {
                // SAFETY: the kernel filled `buf` with a chain of
                // `FILE_FULL_DIR_INFO` records, each at `offset` bytes from
                // the start and each followed by `FileNameLength` bytes of
                // name; `NextEntryOffset` is 0 on the last one.
                let (next, name) = unsafe {
                    let base = buf.as_ptr().cast::<u8>().add(offset);
                    let info = &*base.cast::<FILE_FULL_DIR_INFO>();
                    let chars = info.FileNameLength as usize / 2;
                    let name = std::slice::from_raw_parts(info.FileName.as_ptr(), chars);
                    (info.NextEntryOffset as usize, name)
                };
                if let Ok(name) = String::from_utf16(name)
                    && name != "."
                    && name != ".."
                {
                    names.push(name.into_bytes());
                }
                if next == 0 {
                    break;
                }
                offset += next;
            }
        }
    }

    /// Fills `buf` from the OS cryptographic random source. `ProcessPrng` is
    /// the documented user-mode entry point to it and cannot fail in a
    /// process that has one; a failure is still reported rather than
    /// papered over with a predictable value.
    pub fn random_bytes(buf: &mut [u8]) -> io::Result<()> {
        // SAFETY: `buf` is writable for exactly `buf.len()` bytes.
        let ok = unsafe { ProcessPrng(buf.as_mut_ptr(), buf.len()) };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    /// Deletes `name` from `dir`, acting on a reparse point itself rather
    /// than its target, as `unlinkat` does.
    pub fn unlink_at(dir: &File, name: &str) -> io::Result<()> {
        let file = open_for_delete(dir, name)?;
        // POSIX semantics unlink the name immediately instead of deferring
        // until the last handle closes, so a later open of the same name
        // reports `ENOENT` rather than `STATUS_DELETE_PENDING`.
        let ex = FILE_DISPOSITION_INFO_EX {
            Flags: FILE_DISPOSITION_FLAG_DELETE | FILE_DISPOSITION_FLAG_POSIX_SEMANTICS,
        };
        // SAFETY: `ex` is a live, correctly sized `FILE_DISPOSITION_INFO_EX`.
        let ok = unsafe {
            SetFileInformationByHandle(
                file.as_raw_handle() as HANDLE,
                FileDispositionInfoEx,
                (&raw const ex).cast(),
                size_of::<FILE_DISPOSITION_INFO_EX>() as u32,
            )
        };
        if ok != 0 {
            return Ok(());
        }
        let classic = FILE_DISPOSITION_INFO { DeleteFile: true };
        // SAFETY: as above, for the pre-1709 structure.
        let ok = unsafe {
            SetFileInformationByHandle(
                file.as_raw_handle() as HANDLE,
                FileDispositionInfo,
                (&raw const classic).cast(),
                size_of::<FILE_DISPOSITION_INFO>() as u32,
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    /// Creates directory `name` inside `dir`. `mode` is ignored; see the
    /// module docs. An existing entry reports [`EEXIST`], which `save.rs`'s
    /// walk relies on to tolerate a concurrently created directory.
    pub fn mkdir_at(dir: &File, name: &str, mode: u32) -> io::Result<()> {
        let _ = mode;
        open_at(dir, name, O_RDONLY | O_DIRECTORY | O_CREAT | O_EXCL, 0).map(drop)
    }

    fn full_range() -> OVERLAPPED {
        // SAFETY: `OVERLAPPED` is a plain data structure and an all-zero
        // value is the documented "offset 0, no event" state.
        unsafe { std::mem::zeroed() }
    }

    /// Advisory exclusive lock over the whole file; `Ok(false)` when another
    /// handle (any process, or another handle in this one) holds it.
    ///
    /// Windows byte-range locks are per-handle, so this matches `flock`'s
    /// per-open-file-description exclusion, including within one process.
    pub fn try_lock_exclusive(file: &File) -> io::Result<bool> {
        let mut overlapped = full_range();
        // SAFETY: `file` is an open handle and `overlapped` outlives the
        // call, which cannot block thanks to `LOCKFILE_FAIL_IMMEDIATELY`.
        let ok = unsafe {
            LockFileEx(
                file.as_raw_handle() as HANDLE,
                LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
                0,
                !0u32,
                !0u32,
                &mut overlapped,
            )
        };
        if ok != 0 {
            return Ok(true);
        }
        let err = io::Error::last_os_error();
        if err.raw_os_error() == Some(ERROR_LOCK_VIOLATION as i32) {
            Ok(false)
        } else {
            Err(err)
        }
    }

    pub fn unlock(file: &File) {
        let mut overlapped = full_range();
        // SAFETY: `file` is an open handle; failure is irrelevant at drop.
        unsafe {
            UnlockFileEx(
                file.as_raw_handle() as HANDLE,
                0,
                !0u32,
                !0u32,
                &mut overlapped,
            );
        }
    }
}

#[cfg(not(any(
    windows,
    target_os = "macos",
    all(target_os = "linux", any(target_env = "gnu", target_env = "musl"))
)))]
mod imp {
    use std::fs::File;
    use std::io;
    use std::os::raw::c_int;

    pub const SUPPORTED: bool = false;
    // Inert placeholders so the rest of the crate type-checks: nothing on
    // this target reaches the OS with them, because `ProjectRoot::open`
    // refuses with `Refused::Unsupported` before any rooted call is made.
    pub const O_RDONLY: c_int = 0;
    pub const O_WRONLY: c_int = 0;
    pub const O_RDWR: c_int = 0;
    pub const O_CREAT: c_int = 0;
    pub const O_EXCL: c_int = 0;
    pub const O_NOFOLLOW: c_int = 0;
    pub const O_DIRECTORY: c_int = 0;
    pub const O_NONBLOCK: c_int = 0;
    pub const ELOOP: i32 = -1;
    pub const ENOENT: i32 = -1;
    pub const EEXIST: i32 = -1;
    pub const ENOTDIR: i32 = -1;

    fn unsupported() -> io::Error {
        io::Error::new(
            io::ErrorKind::Unsupported,
            "rooted file operations are not available on this target",
        )
    }
    pub fn open_at(_: &File, _: &str, _: c_int, _: u32) -> io::Result<File> {
        Err(unsupported())
    }
    pub fn open_at_bytes(_: &File, _: &[u8], _: c_int, _: u32) -> io::Result<File> {
        Err(unsupported())
    }
    pub fn stat_at_nofollow(_: &File, _: &[u8]) -> io::Result<super::EntryStat> {
        Err(unsupported())
    }
    pub fn rename_at(_: &File, _: &str, _: &str) -> io::Result<()> {
        Err(unsupported())
    }
    pub fn rename_at_noreplace(_: &File, _: &str, _: &str) -> io::Result<()> {
        Err(unsupported())
    }
    pub fn noreplace_unsupported(_: &io::Error) -> bool {
        false
    }
    pub fn link_at(_: &File, _: &str, _: &str) -> io::Result<()> {
        Err(unsupported())
    }
    pub fn link_unsupported(_: &io::Error) -> bool {
        false
    }
    pub fn link_fd_at(_: &File, _: &File, _: &str) -> io::Result<()> {
        Err(unsupported())
    }
    pub fn list_dir(_: &File) -> io::Result<Vec<Vec<u8>>> {
        Err(unsupported())
    }
    pub fn unlink_at(_: &File, _: &str) -> io::Result<()> {
        Err(unsupported())
    }
    pub fn mkdir_at(_: &File, _: &str, _: u32) -> io::Result<()> {
        Err(unsupported())
    }
    pub fn try_lock_exclusive(_: &File) -> io::Result<bool> {
        Err(unsupported())
    }
    pub fn random_bytes(_: &mut [u8]) -> io::Result<()> {
        Err(unsupported())
    }
    pub fn unlock(_: &File) {}
    pub fn open_dir_std(_: &std::path::Path) -> io::Result<File> {
        Err(unsupported())
    }
    pub fn verify_parent(_: &File, _: &File) -> io::Result<bool> {
        Err(unsupported())
    }
}

pub use imp::*;

pub fn errno_is(err: &io::Error, code: i32) -> bool {
    code >= 0 && err.raw_os_error() == Some(code)
}

/// Opens `path` itself as a directory without following a final symlink.
///
/// **Symlinks above the root are followed, deliberately.** Only the final
/// component is opened with `O_NOFOLLOW`; its parent is opened by path string
/// with ordinary resolution, so a symlink among the root's *ancestors*
/// (`/Users/me/link/project` with `link -> /Volumes/work`) is followed and
/// the directory it leads to becomes the pinned root. The caller chose that
/// path, and symlinked home directories, checkouts and volume mounts are
/// common; refusing them would reject real projects without protecting
/// anything the caller did not already select. The refuse-all-symlinks
/// policy applies *inside* the root: once the root descriptor is pinned,
/// every project path is walked from it with `O_NOFOLLOW` at each component,
/// and a later change to the ancestors does not move the pinned root.
pub fn open_dir_nofollow(path: &std::path::Path) -> io::Result<File> {
    // `std` has no O_NOFOLLOW knob, so open the parent with std and the last
    // component with `openat`.
    let (parent, name) = match (path.parent(), path.file_name().and_then(final_name)) {
        (Some(p), Some(n)) if !n.is_empty() => (p, n),
        // "/", "C:\" or similar: nothing to refuse.
        _ => return open_dir_std(path),
    };
    let parent = open_dir_std(if parent.as_os_str().is_empty() {
        std::path::Path::new(".")
    } else {
        parent
    })?;
    open_dir_at_nofollow_bytes(&parent, name)
}

/// The root's final component as the bytes [`open_dir_at_nofollow_bytes`]
/// takes.
///
/// On unix these are the name's own bytes, so a root whose final component is
/// not valid UTF-8 still gets the `O_NOFOLLOW` open rather than a
/// symlink-following fallback. Windows names are UTF-16 and the NT emulation
/// takes UTF-8, so a name that does not survive that conversion (unpaired
/// surrogates) has no rooted open available and falls back to the plain `std`
/// one, exactly as an unnamed root does.
#[cfg(unix)]
fn final_name(name: &std::ffi::OsStr) -> Option<&[u8]> {
    use std::os::unix::ffi::OsStrExt;
    Some(name.as_bytes())
}

#[cfg(not(unix))]
fn final_name(name: &std::ffi::OsStr) -> Option<&[u8]> {
    name.to_str().map(str::as_bytes)
}

/// Opens directory `name` inside `dir` with `O_DIRECTORY | O_NOFOLLOW`.
///
/// symlink-to-directory in this mode. Windows agrees with macOS: a *file*
/// symlink opened with `FILE_DIRECTORY_FILE` reports `STATUS_NOT_A_DIRECTORY`
/// (measured), while a *directory* symlink opens as its own reparse point and
/// is caught directly as `ELOOP`. All three refuse to follow; to classify the
/// refusal consistently, an `ENOTDIR` result is classified with
/// [`stat_at_nofollow`] (on unix never a second open, which could run a
/// device's open side effects) and mapped to `ELOOP` when the entry is a
/// symlink.
pub fn open_dir_at_nofollow(dir: &File, name: &str) -> io::Result<File> {
    open_dir_at_nofollow_bytes(dir, name.as_bytes())
}

fn open_dir_at_nofollow_bytes(dir: &File, name: &[u8]) -> io::Result<File> {
    match open_at_bytes(dir, name, O_RDONLY | O_DIRECTORY | O_NOFOLLOW, 0) {
        Err(e) if errno_is(&e, ENOTDIR) => match stat_at_nofollow(dir, name) {
            Ok(st) if st.is_symlink() => Err(io::Error::from_raw_os_error(ELOOP)),
            _ => Err(e),
        },
        other => other,
    }
}

/// File type bits of a [`EntryStat::mode`].
///
/// On unix these come from `libc` (`mode_t` is 16 bits on macOS and 32 on
/// Linux; the values are widened, never truncated) and are the real
/// `st_mode` bits. Elsewhere they are the same well-known POSIX numbering,
/// used purely as this crate's own encoding of "regular file / directory /
/// symlink" — see the Windows [`stat_at_nofollow`], which synthesizes them
/// from the file attributes, so [`EntryStat::is_file`] and friends need no
/// `cfg` of their own.
#[cfg(unix)]
#[allow(clippy::unnecessary_cast)]
pub const S_IFMT: u32 = libc::S_IFMT as u32;
#[cfg(unix)]
#[allow(clippy::unnecessary_cast)]
pub const S_IFREG: u32 = libc::S_IFREG as u32;
#[cfg(unix)]
#[allow(clippy::unnecessary_cast)]
pub const S_IFDIR: u32 = libc::S_IFDIR as u32;
#[cfg(unix)]
#[allow(clippy::unnecessary_cast)]
pub const S_IFLNK: u32 = libc::S_IFLNK as u32;

#[cfg(not(unix))]
pub const S_IFMT: u32 = 0o170000;
#[cfg(not(unix))]
pub const S_IFREG: u32 = 0o100000;
#[cfg(not(unix))]
pub const S_IFDIR: u32 = 0o040000;
#[cfg(not(unix))]
pub const S_IFLNK: u32 = 0o120000;

/// What [`stat_at_nofollow`] reports about one directory entry.
///
/// The identity fields are platform-specific for exactly the reason
/// [`crate::FileIdentity`]'s are, and carry the same values: on unix
/// `(st_dev, st_ino)`, on Windows the `FILE_ID_INFO` pair that
/// [`file_id`] returns for an open handle to the same file. Only equality is
/// meaningful; `save.rs` turns a whole `EntryStat` into a `FileIdentity`
/// rather than reading a field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntryStat {
    /// `st_dev`, widened the same way `std::os::unix::fs::MetadataExt::dev` does.
    #[cfg(unix)]
    pub dev: u64,
    #[cfg(unix)]
    pub ino: u64,
    /// `FILE_ID_INFO::VolumeSerialNumber`.
    #[cfg(windows)]
    pub volume_serial: u64,
    /// `FILE_ID_INFO::FileId`, the 128-bit form (ReFS IDs do not fit in 64).
    #[cfg(windows)]
    pub file_index: u128,
    /// File type and, on unix, permission bits. Windows has no permission
    /// bits, so only the type bits above are set there.
    pub mode: u32,
}

impl EntryStat {
    pub fn is_file(&self) -> bool {
        self.mode & S_IFMT == S_IFREG
    }
    pub fn is_dir(&self) -> bool {
        self.mode & S_IFMT == S_IFDIR
    }
    pub fn is_symlink(&self) -> bool {
        self.mode & S_IFMT == S_IFLNK
    }
}

/// Windows-only tests for the parts of `imp` that are a *different
/// algorithm* from the POSIX path rather than a different spelling of the
/// same one, and for the `NTSTATUS`-to-sentinel mapping the save layer's
/// [`errno_is`] comparisons depend on. Every expectation here was
/// established by running it, not by reading documentation: the mapping is
/// not guessable, and a wrong one fails silently, turning a refusal into an
/// unclassified I/O error.
#[cfg(all(test, windows))]
mod windows_tests {
    use super::*;
    use std::path::{Path, PathBuf};

    struct Temp(PathBuf);
    impl Temp {
        fn new(tag: &str) -> Self {
            use std::sync::atomic::{AtomicU64, Ordering};
            static N: AtomicU64 = AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!(
                "flashtex-sys-{tag}-{}-{}",
                std::process::id(),
                N.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(&p).unwrap();
            Temp(p)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// [`verify_parent`] stands in for POSIX's "open `..` and compare
    /// device/inode", which Windows cannot do at all — `NtCreateFile` with a
    /// `RootDirectory` rejects the name `".."` outright. This pins the
    /// answers `walk` relies on, including the one that matters for
    /// `Refused::EscapesRoot`: a directory moved out of the root mid-walk
    /// stops counting as its child, even though the handle stays valid and
    /// open across the move.
    #[test]
    fn verify_parent_accepts_a_child_and_rejects_a_grandchild_or_a_moved_directory() {
        let t = Temp::new("verify-parent");
        std::fs::create_dir_all(t.path().join("a/b")).unwrap();
        let root = open_dir_nofollow(t.path()).unwrap();
        let a = open_dir_at_nofollow(&root, "a").unwrap();
        let b = open_dir_at_nofollow(&a, "b").unwrap();

        assert!(verify_parent(&root, &a).unwrap(), "a is a child of root");
        assert!(verify_parent(&a, &b).unwrap(), "b is a child of a");
        assert!(
            !verify_parent(&root, &b).unwrap(),
            "b is a grandchild of root, not a child: one component too deep"
        );
        assert!(
            !verify_parent(&b, &a).unwrap(),
            "the relationship is not symmetric"
        );

        // The escape `Refused::EscapesRoot` exists to catch: the directory is
        // moved outside the root while the walk still holds its handle.
        // Windows refuses to move a directory that has open handles *inside*
        // it, so the grandchild is closed first; `a`'s own handle stays open
        // across the move, which is the whole point.
        drop(b);
        let outside = Temp::new("verify-parent-outside");
        std::fs::rename(t.path().join("a"), outside.path().join("a")).unwrap();
        assert!(
            !verify_parent(&root, &a).unwrap(),
            "a directory moved out of the root must stop counting as its child"
        );
    }

    /// The four sentinels `save.rs` matches on, each produced by actually
    /// causing the condition.
    #[test]
    fn nt_status_codes_map_onto_the_sentinels_save_rs_matches() {
        let t = Temp::new("errno");
        let root = open_dir_nofollow(t.path()).unwrap();

        let missing = open_at(&root, "nope.tex", O_RDONLY | O_NOFOLLOW, 0).unwrap_err();
        assert!(errno_is(&missing, ENOENT), "{missing:?}");

        mkdir_at(&root, "sub", 0o755).unwrap();
        let twice = mkdir_at(&root, "sub", 0o755).unwrap_err();
        assert!(errno_is(&twice, EEXIST), "{twice:?}");

        std::fs::write(t.path().join("file.tex"), b"x").unwrap();
        let not_dir =
            open_at(&root, "file.tex", O_RDONLY | O_DIRECTORY | O_NOFOLLOW, 0).unwrap_err();
        assert!(errno_is(&not_dir, ENOTDIR), "{not_dir:?}");

        let exists = open_at(
            &root,
            "file.tex",
            O_WRONLY | O_CREAT | O_EXCL | O_NOFOLLOW,
            0o644,
        )
        .unwrap_err();
        assert!(errno_is(&exists, EEXIST), "{exists:?}");
    }

    /// A junction is a reparse point tagged `IO_REPARSE_TAG_MOUNT_POINT`
    /// rather than `IO_REPARSE_TAG_SYMLINK`, and unlike a symlink it needs
    /// no Developer Mode or elevation — so it is the escape route most
    /// likely to be reachable in the wild. Both are name surrogates and both
    /// must report [`ELOOP`].
    #[test]
    fn a_junction_reports_eloop_exactly_as_a_symlink_would() {
        let outside = Temp::new("junction-target");
        std::fs::write(outside.path().join("victim.tex"), b"external").unwrap();
        let t = Temp::new("junction-holder");
        let link = t.path().join("linked");
        let made = std::process::Command::new("cmd")
            .args(["/c", "mklink", "/J"])
            .arg(&link)
            .arg(outside.path())
            .output()
            .is_ok_and(|o| o.status.success());
        assert!(made, "`mklink /J` is required to create the junction");

        let root = open_dir_nofollow(t.path()).unwrap();
        let err = open_dir_at_nofollow(&root, "linked").unwrap_err();
        assert!(
            errno_is(&err, ELOOP),
            "a junction must not be followed: {err:?}"
        );
        // Without `O_NOFOLLOW` the junction *is* followed, which is what
        // proves the refusal above came from the flag rather than from a
        // broken open.
        assert!(open_at(&root, "linked", O_RDONLY | O_DIRECTORY, 0).is_ok());
    }

    /// `save.rs` fsyncs the directory it renamed into. On Windows that is
    /// `FlushFileBuffers`, which fails with `ERROR_ACCESS_DENIED` unless the
    /// handle carries write access — hence the `FILE_WRITE_DATA` in
    /// `open_at`'s directory path. Were that to regress, every save would
    /// return `SaveError::DirectorySync` and claim durability was unknown.
    #[test]
    fn directory_handles_can_be_fsynced() {
        let t = Temp::new("dirsync");
        std::fs::create_dir(t.path().join("sub")).unwrap();
        let root = open_dir_nofollow(t.path()).unwrap();
        root.sync_all()
            .expect("the root directory handle must be flushable");
        let sub = open_dir_at_nofollow(&root, "sub").unwrap();
        sub.sync_all()
            .expect("a walked directory handle must be flushable");
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::MetadataExt;

    /// The `fstatat` fields must agree with `std`'s own `lstat` for every
    /// file type this crate distinguishes.
    #[test]
    fn stat_list_and_noreplace_match_std() {
        if !SUPPORTED {
            return;
        }
        let dir = std::env::temp_dir().join(format!(
            "flashtex-sys-stat-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("file"), "x").unwrap();
        std::os::unix::fs::symlink(dir.join("file"), dir.join("link")).unwrap();
        std::os::unix::fs::symlink(dir.join("absent"), dir.join("broken")).unwrap();
        let handle = open_dir_nofollow(&dir).unwrap();
        for name in ["file", "sub", "link", "broken"] {
            let st = stat_at_nofollow(&handle, name.as_bytes()).unwrap();
            let meta = std::fs::symlink_metadata(dir.join(name)).unwrap();
            assert_eq!(st.mode, meta.mode(), "{name}");
            assert_eq!(st.ino, meta.ino(), "{name}");
            assert_eq!(st.dev, meta.dev(), "{name}");
            let ft = meta.file_type();
            assert_eq!(
                (st.is_file(), st.is_dir(), st.is_symlink()),
                (ft.is_file(), ft.is_dir(), ft.is_symlink()),
                "{name}"
            );
        }
        let err = stat_at_nofollow(&handle, b"absent").unwrap_err();
        assert!(errno_is(&err, ENOENT), "{err:?}");

        // `list_dir` agrees with `std::fs::read_dir`, and a second listing
        // of the same handle is complete (its own read offset each time).
        let mut expected: Vec<Vec<u8>> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| {
                use std::os::unix::ffi::OsStrExt;
                e.unwrap().file_name().as_bytes().to_vec()
            })
            .collect();
        expected.sort();
        for _ in 0..2 {
            let mut names = list_dir(&handle).unwrap();
            names.sort();
            assert_eq!(names, expected);
        }

        // `rename_at_noreplace` never replaces an existing entry.
        std::fs::write(dir.join("other"), "y").unwrap();
        match rename_at_noreplace(&handle, "other", "file") {
            Err(e) if noreplace_unsupported(&e) => {}
            Err(e) => assert!(errno_is(&e, EEXIST), "{e:?}"),
            Ok(()) => panic!("replaced an existing entry"),
        }
        assert_eq!(std::fs::read_to_string(dir.join("file")).unwrap(), "x");
        rename_at_noreplace(&handle, "other", "fresh")
            .or_else(|e| {
                if noreplace_unsupported(&e) {
                    rename_at(&handle, "other", "fresh")
                } else {
                    Err(e)
                }
            })
            .unwrap();
        assert_eq!(std::fs::read_to_string(dir.join("fresh")).unwrap(), "y");

        // `link_at` never replaces an existing entry, dangling symlinks
        // included, and links a regular file under a new name.
        for existing in ["file", "broken"] {
            let err = link_at(&handle, "fresh", existing).unwrap_err();
            assert!(errno_is(&err, EEXIST), "{existing}: {err:?}");
        }
        assert_eq!(std::fs::read_to_string(dir.join("file")).unwrap(), "x");
        link_at(&handle, "fresh", "linked").unwrap();
        assert_eq!(std::fs::read_to_string(dir.join("linked")).unwrap(), "y");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
