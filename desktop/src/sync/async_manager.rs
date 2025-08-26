use super::{SyncEngine, SyncMessage};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, SyncSender},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Default interval to wait for additional messages before processing a batch.
pub const DEFAULT_BATCH_DELAY: Duration = Duration::from_millis(50);

/// Default capacity for the internal sync channel.
pub const DEFAULT_CHANNEL_CAPACITY: usize = 32;

/// Manages asynchronous processing of [`SyncMessage`]s.
///
/// Messages are queued and processed on a background thread. Rapid sequences of
/// messages are batched together, providing basic debouncing so costly
/// operations like [`meta::read_all`](multicode_core::meta::read_all) and
/// [`meta::upsert`](multicode_core::meta::upsert) are executed off the main
/// thread and at a controlled rate.
///
/// The manager also supports pausing and resuming of message processing.
pub struct AsyncManager {
    tx: Option<SyncSender<SyncMessage>>,
    handle: Option<JoinHandle<()>>,
    paused: Arc<AtomicBool>,
}

impl AsyncManager {
    /// Spawns a new background worker wrapping the provided [`SyncEngine`].
    ///
    /// Use [`DEFAULT_BATCH_DELAY`] for a reasonable default value and
    /// [`DEFAULT_CHANNEL_CAPACITY`] for a typical channel size.
    pub fn new(engine: SyncEngine, batch_delay: Duration, capacity: usize) -> Self {
        let (tx, rx) = mpsc::sync_channel(capacity);
        let paused = Arc::new(AtomicBool::new(false));
        let worker_paused = paused.clone();
        let handle = thread::spawn(move || run_worker(engine, rx, worker_paused, batch_delay));
        Self {
            tx: Some(tx),
            handle: Some(handle),
            paused,
        }
    }

    /// Enqueues a [`SyncMessage`] for asynchronous processing.
    pub fn send(&self, msg: SyncMessage) -> Result<(), mpsc::SendError<SyncMessage>> {
        if let Some(tx) = &self.tx {
            tx.send(msg)
        } else {
            Err(mpsc::SendError(msg))
        }
    }

    /// Pauses processing of incoming messages.
    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
    }

    /// Resumes processing of incoming messages.
    pub fn resume(&self) {
        self.paused.store(false, Ordering::SeqCst);
    }
}

fn run_worker(
    mut engine: SyncEngine,
    rx: Receiver<SyncMessage>,
    paused: Arc<AtomicBool>,
    batch_delay: Duration,
) {
    while let Ok(first) = rx.recv() {
        if paused.load(Ordering::SeqCst) {
            continue;
        }
        let mut batch = vec![first];
        loop {
            match rx.recv_timeout(batch_delay) {
                Ok(msg) => {
                    if paused.load(Ordering::SeqCst) {
                        batch.clear();
                        break;
                    }
                    batch.push(msg);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
            }
        }
        if paused.load(Ordering::SeqCst) {
            continue;
        }
        for msg in batch {
            let _ = engine.handle(msg);
        }
    }
}

impl Drop for AsyncManager {
    fn drop(&mut self) {
        // Dropping the sender will make the worker exit; join to clean up.
        self.tx.take();
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
