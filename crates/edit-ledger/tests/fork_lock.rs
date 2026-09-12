//! A concurrently spawned compiler can inherit an open-file description between
//! fork and exec. Closing only the parent's descriptor must not retain ownership.
#![cfg(unix)]
use flashtex_edit_ledger::Store;

struct ChildGuard {
    pid: libc::pid_t,
    wake: libc::c_int,
}
impl Drop for ChildGuard {
    fn drop(&mut self) {
        // SAFETY: parent owns this valid pipe writer and exact child PID. The
        // child executes only async-signal-safe calls before _exit.
        unsafe {
            let byte = [1u8];
            libc::write(self.wake, byte.as_ptr().cast(), 1);
            libc::close(self.wake);
            libc::waitpid(self.pid, std::ptr::null_mut(), 0);
        }
    }
}

#[test]
fn dropping_owner_unlocks_even_when_fork_child_inherits_descriptor() {
    let dir = tempfile::tempdir().unwrap();
    let owner = Store::open(dir.path()).unwrap();
    let mut pipe = [-1; 2];
    // SAFETY: pipe points to two writable descriptors; the fork child performs
    // no allocation, Rust cleanup or locking in this multithreaded test process.
    let pid = unsafe {
        assert_eq!(libc::pipe(pipe.as_mut_ptr()), 0);
        let pid = libc::fork();
        if pid == 0 {
            libc::close(pipe[1]);
            let mut byte = 0u8;
            libc::read(pipe[0], (&mut byte as *mut u8).cast(), 1);
            libc::_exit(0);
        }
        pid
    };
    assert!(pid > 0);
    // SAFETY: only the child needs the read end after a successful fork.
    unsafe {
        libc::close(pipe[0]);
    }
    let _child = ChildGuard { pid, wake: pipe[1] };
    drop(owner);
    // Child is deliberately alive with the inherited lock descriptor.
    let reopened = Store::open(dir.path());
    assert!(
        reopened.is_ok(),
        "parent drop retained a child-inherited lock: {:?}",
        reopened.err()
    );
}
