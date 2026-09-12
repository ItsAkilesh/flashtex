//! Required FIFO plus one optional frame. An already-started write is not preemptible.
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
#[derive(Default)]
struct State {
    required: usize,
    epoch: u64,
    optional: Option<Vec<u8>>,
}
#[derive(Clone)]
pub struct Sender {
    tx: mpsc::SyncSender<Vec<u8>>,
    state: Arc<Mutex<State>>,
}
pub struct Receiver {
    rx: mpsc::Receiver<Vec<u8>>,
    state: Arc<Mutex<State>>,
}
pub struct Frame {
    pub bytes: Vec<u8>,
    required: bool,
}
pub fn channel(slots: usize) -> (Sender, Receiver) {
    let (tx, rx) = mpsc::sync_channel(slots);
    let state = Arc::new(Mutex::new(State::default()));
    (
        Sender {
            tx,
            state: state.clone(),
        },
        Receiver { rx, state },
    )
}
impl Sender {
    pub fn try_send(&self, bytes: Vec<u8>) -> Result<(), mpsc::TrySendError<Vec<u8>>> {
        let mut state = self.state.lock().unwrap();
        // Never retain optional output while a required response is pending.
        state.optional = None;
        self.tx.try_send(bytes)?;
        state.required += 1;
        Ok(())
    }
    pub fn reset_optional(&self) -> u64 {
        let mut state = self.state.lock().unwrap();
        state.optional = None;
        state.epoch = state.epoch.saturating_add(1);
        state.epoch
    }
    pub fn can_offer(&self, epoch: u64) -> bool {
        let state = self.state.lock().unwrap();
        state.required == 0 && epoch == state.epoch && epoch != u64::MAX
    }
    pub fn optional(&self, epoch: u64, bytes: Vec<u8>) -> bool {
        let mut state = self.state.lock().unwrap();
        if state.required != 0 || epoch != state.epoch || epoch == u64::MAX {
            return false;
        }
        state.optional = Some(bytes);
        true
    }
}
impl Receiver {
    pub fn next(&self, timeout: Duration) -> Result<Frame, mpsc::RecvTimeoutError> {
        match self.rx.recv_timeout(timeout) {
            Ok(bytes) => Ok(Frame {
                bytes,
                required: true,
            }),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let mut state = self.state.lock().unwrap();
                if state.required == 0 {
                    if let Some(bytes) = state.optional.take() {
                        return Ok(Frame {
                            bytes,
                            required: false,
                        });
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout)
            }
            Err(error) => Err(error),
        }
    }
    pub fn written(&self, frame: &Frame) {
        if frame.required {
            self.state.lock().unwrap().required -= 1;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn required_pending_and_inflight_both_exclude_optional() {
        let (tx, rx) = channel(2);
        assert!(tx.optional(0, b"history\n".to_vec()));
        tx.try_send(b"ack\n".to_vec()).unwrap();
        assert!(!tx.optional(0, b"late\n".to_vec()));
        let ack = rx.next(Duration::ZERO).unwrap();
        assert_eq!(ack.bytes, b"ack\n");
        assert!(!tx.optional(0, b"while-writing\n".to_vec()));
        rx.written(&ack);
        assert!(rx.next(Duration::ZERO).is_err());
        assert!(tx.optional(0, b"new\n".to_vec()));
        assert_eq!(rx.next(Duration::ZERO).unwrap().bytes, b"new\n");
    }
    #[test]
    fn bounded_optional_replacement_reset_and_required_full_preserve_fifo() {
        let (tx, rx) = channel(1);
        for _ in 0..100 {
            assert!(tx.optional(0, b"history\n".to_vec()));
        }
        let epoch = tx.reset_optional();
        assert!(!tx.optional(0, b"old\n".to_vec()));
        assert!(rx.next(Duration::ZERO).is_err());
        assert!(tx.optional(epoch, b"new\n".to_vec()));
        tx.try_send(b"ack1\n".to_vec()).unwrap();
        let error = tx.try_send(b"ack2\n".to_vec()).unwrap_err();
        let mpsc::TrySendError::Full(bytes) = error else {
            panic!("expected full")
        };
        assert_eq!(bytes, b"ack2\n");
        let first = rx.next(Duration::ZERO).unwrap();
        rx.written(&first);
        tx.try_send(bytes).unwrap();
        let second = rx.next(Duration::ZERO).unwrap();
        assert_eq!(second.bytes, b"ack2\n");
        rx.written(&second);
        assert!(rx.next(Duration::ZERO).is_err());
    }
}
