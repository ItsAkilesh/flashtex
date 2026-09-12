//! One decoding/validation permit, independent of the four-frame raw queue.
//! Shutdown joins the worker after any current bounded-frame serde parse finishes.
use serde_json::Value;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TryRecvError},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
pub(crate) enum RawInput {
    Frame(Vec<u8>, Instant, Instant),
    Failure(String),
}
pub(crate) enum Input {
    Frame(DecodedFrame),
    Failure(String),
}
struct Permit(SyncSender<()>);
impl Drop for Permit {
    fn drop(&mut self) {
        let _ = self.0.try_send(());
    }
}
pub(crate) struct DecodedFrame {
    pub value: Option<Value>,
    pub raw: Option<Box<crate::raw_display::Parsed>>,
    pub response_bytes: usize,
    pub first_byte: Instant,
    pub reader_done: Instant,
    pub decoded_at: Instant,
    pub parse_ms: f64,
    pub decode_queue_wait_ms: f64,
    _permit: Permit,
}
pub(crate) struct Decoder {
    reader: Option<Receiver<Input>>,
    stopped: Arc<AtomicBool>,
    wake: SyncSender<()>,
    worker: Option<JoinHandle<()>>,
    #[cfg(test)]
    published: Receiver<()>,
}
impl Decoder {
    #[cfg(test)]
    pub fn spawn(raw: Receiver<RawInput>) -> Self {
        Self::spawn_mode(raw, false)
    }
    pub fn spawn_mode(raw: Receiver<RawInput>, raw_display: bool) -> Self {
        let (out, reader) = mpsc::sync_channel(1);
        let (wake, permits) = mpsc::sync_channel(1);
        wake.send(()).expect("initial decoder permit");
        let stopped = Arc::new(AtomicBool::new(false));
        let stop = stopped.clone();
        let release = wake.clone();
        #[cfg(test)]
        let (published_tx, published) = mpsc::channel();
        let worker = thread::spawn(move || {
            loop {
                if stop.load(Ordering::Acquire) {
                    break;
                }
                match permits.recv_timeout(Duration::from_millis(10)) {
                    Ok(()) => {}
                    Err(mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
                let message = loop {
                    if stop.load(Ordering::Acquire) {
                        return;
                    }
                    match raw.recv_timeout(Duration::from_millis(10)) {
                        Ok(message) => break message,
                        Err(mpsc::RecvTimeoutError::Timeout) => continue,
                        Err(mpsc::RecvTimeoutError::Disconnected) => return,
                    }
                };
                match message {
                    RawInput::Failure(reason) => {
                        let _ = out.send(Input::Failure(reason));
                        break;
                    }
                    RawInput::Frame(bytes, first_byte, reader_done) => {
                        let response_bytes = bytes.len();
                        let start = Instant::now();
                        let decode_queue_wait_ms =
                            start.saturating_duration_since(reader_done).as_secs_f64() * 1000.0;
                        let raw_kind = if raw_display {
                            crate::raw_display::is_display(&bytes)
                        } else {
                            Ok(false)
                        };
                        let parsed = match raw_kind {
                            Err(e) => Err(e),
                            Ok(true) => crate::raw_display::Parsed::parse(bytes)
                                .map(|raw| (None, Some(Box::new(raw)))),
                            Ok(false) => std::str::from_utf8(&bytes)
                                .map_err(|e| e.to_string())
                                .and_then(|text| {
                                    serde_json::from_str(text).map_err(|e| e.to_string())
                                })
                                .map(|value| (Some(value), None)),
                        };
                        let parse_ms = start.elapsed().as_secs_f64() * 1000.0;
                        if stop.load(Ordering::Acquire) {
                            break;
                        }
                        match parsed {
                            Ok((value, raw)) => {
                                let packet = DecodedFrame {
                                    value,
                                    raw,
                                    response_bytes,
                                    first_byte,
                                    reader_done,
                                    decoded_at: Instant::now(),
                                    parse_ms,
                                    decode_queue_wait_ms,
                                    _permit: Permit(release.clone()),
                                };
                                if out.send(Input::Frame(packet)).is_err() {
                                    break;
                                }
                                #[cfg(test)]
                                let _ = published_tx.send(());
                                // The packet keeps the only permit through owner validation.
                            }
                            Err(_) => {
                                let _ = out.send(Input::Failure(
                                    "compiler returned malformed JSON".into(),
                                ));
                                break;
                            }
                        }
                    }
                }
            }
        });
        Self {
            reader: Some(reader),
            stopped,
            wake,
            worker: Some(worker),
            #[cfg(test)]
            published,
        }
    }
    pub fn try_recv(&self) -> Result<Input, TryRecvError> {
        self.reader
            .as_ref()
            .expect("live decoder receiver")
            .try_recv()
    }
}
impl Drop for Decoder {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Release);
        // Release a blocked output send and wake a worker whose packet is held
        // by the caller. No guard must be dropped before shutdown can complete.
        self.reader.take();
        let _ = self.wake.try_send(());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn raw(bytes: &[u8]) -> RawInput {
        RawInput::Frame(bytes.to_vec(), Instant::now(), Instant::now())
    }
    fn receive(decoder: &Decoder) -> Input {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match decoder.try_recv() {
                Ok(input) => return input,
                Err(TryRecvError::Empty) => {
                    assert!(Instant::now() < deadline);
                    thread::yield_now();
                }
                Err(TryRecvError::Disconnected) => panic!("decoder disconnected"),
            }
        }
    }
    #[test]
    fn sole_permit_covers_queued_and_owner_held_values_with_four_raw_slots() {
        let (tx, rx) = mpsc::sync_channel(4);
        let decoder = Decoder::spawn(rx);
        tx.send(raw(br#"{"id":"first"}"#)).unwrap();
        let Input::Frame(mut first) = receive(&decoder) else {
            panic!("frame")
        };
        assert_eq!(first.value.take().unwrap()["id"], "first");
        for _ in 0..4 {
            tx.try_send(raw(b"{}")).unwrap();
        }
        assert!(matches!(
            tx.try_send(raw(b"{}")),
            Err(mpsc::TrySendError::Full(_))
        ));
        assert!(matches!(decoder.try_recv(), Err(TryRecvError::Empty)));
        drop(first);
        assert!(matches!(receive(&decoder), Input::Frame(_)));
    }
    #[test]
    fn malformed_and_invalid_utf8_are_failures_not_partial_values() {
        for bytes in [b"{bad".as_slice(), b"\xff".as_slice()] {
            let (tx, rx) = mpsc::sync_channel(4);
            let decoder = Decoder::spawn(rx);
            tx.send(raw(bytes)).unwrap();
            assert!(
                matches!(receive(&decoder), Input::Failure(reason) if reason.contains("malformed JSON"))
            );
        }
    }
    #[test]
    fn shutdown_joins_with_owner_permit_held_and_raw_queue_full() {
        let (tx, rx) = mpsc::sync_channel(4);
        let decoder = Decoder::spawn(rx);
        tx.send(raw(b"{}")).unwrap();
        let held = receive(&decoder);
        for _ in 0..4 {
            tx.try_send(raw(b"{}")).unwrap();
        }
        drop(decoder); // Would deadlock if shutdown waited for the caller's permit.
        assert!(matches!(
            tx.try_send(raw(b"{}")),
            Err(mpsc::TrySendError::Disconnected(_))
        ));
        drop(held);
    }
    #[test]
    fn shutdown_joins_when_raw_producer_stays_open_but_idle() {
        let (tx, rx) = mpsc::sync_channel(4);
        let decoder = Decoder::spawn(rx);
        drop(decoder);
        assert!(tx.send(raw(b"{}")).is_err());
    }
    #[test]
    fn shutdown_joins_when_decoded_and_raw_queues_are_both_full() {
        let (tx, rx) = mpsc::sync_channel(4);
        let decoder = Decoder::spawn(rx);
        tx.send(raw(b"{}")).unwrap();
        // Notification is sent only after output publication succeeds. Leave the
        // decoded packet in its queue while filling the independent raw queue.
        decoder
            .published
            .recv_timeout(Duration::from_secs(2))
            .unwrap();
        for _ in 0..4 {
            tx.try_send(raw(b"{}")).unwrap();
        }
        assert!(matches!(
            tx.try_send(raw(b"{}")),
            Err(mpsc::TrySendError::Full(_))
        ));
        drop(decoder);
        assert!(matches!(
            tx.try_send(raw(b"{}")),
            Err(mpsc::TrySendError::Disconnected(_))
        ));
    }
}
