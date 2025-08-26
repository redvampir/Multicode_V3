use super::{SyncEngine, SyncMessage};
use std::sync::{
    mpsc::{self, Receiver, SyncSender},
    Arc, Condvar, Mutex,
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
/// The manager also supports pausing and resuming of message processing and can
/// be shut down explicitly via [`shutdown`](AsyncManager::shutdown).
///
/// # Example
/// ```ignore
/// use desktop::sync::{
///     AsyncManager, ResolutionPolicy, SyncEngine, SyncMessage, DEFAULT_BATCH_DELAY,
///     DEFAULT_CHANNEL_CAPACITY,
/// };
/// use multicode_core::parser::Lang;
///
/// let engine = SyncEngine::new(Lang::Rust, ResolutionPolicy::PreferText);
/// let manager = AsyncManager::new(engine, DEFAULT_BATCH_DELAY, DEFAULT_CHANNEL_CAPACITY);
/// manager
///     .send(SyncMessage::TextChanged(String::new(), Lang::Rust))
///     .unwrap();
/// manager.shutdown();
/// ```
pub struct AsyncManager {
    tx: Option<SyncSender<Option<SyncMessage>>>,
    handle: Option<JoinHandle<()>>,
    paused: Arc<(Mutex<bool>, Condvar)>,
}

impl AsyncManager {
    /// Spawns a new background worker wrapping the provided [`SyncEngine`].
    ///
    /// Use [`DEFAULT_BATCH_DELAY`] for a reasonable default value and
    /// [`DEFAULT_CHANNEL_CAPACITY`] for a typical channel size.
    pub fn new(engine: SyncEngine, batch_delay: Duration, capacity: usize) -> Self {
        let (tx, rx) = mpsc::sync_channel(capacity);
        let paused = Arc::new((Mutex::new(false), Condvar::new()));
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
            tx.send(Some(msg)).map_err(|e| match e.0 {
                Some(m) => mpsc::SendError(m),
                None => unreachable!("sender returned shutdown marker"),
            })
        } else {
            Err(mpsc::SendError(msg))
        }
    }

    /// Pauses processing of incoming messages.
    pub fn pause(&self) {
        let (lock, cvar) = &*self.paused;
        let mut paused = lock.lock().unwrap();
        *paused = true;
        cvar.notify_all();
    }

    /// Resumes processing of incoming messages.
    pub fn resume(&self) {
        let (lock, cvar) = &*self.paused;
        let mut paused = lock.lock().unwrap();
        *paused = false;
        cvar.notify_all();
    }

    /// Gracefully stops the background worker and waits for it to finish.
    ///
    /// This method is idempotent; dropping [`AsyncManager`] will also attempt to
    /// shut down the worker if it hasn't been called explicitly.
    pub fn shutdown(mut self) {
        self.shutdown_inner();
    }

    fn shutdown_inner(&mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(None);
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
impl AsyncManager {
    /// Creates a new manager with an additional logger for processed batches.
    pub(crate) fn new_with_logger(
        engine: SyncEngine,
        batch_delay: Duration,
        capacity: usize,
        log: Arc<Mutex<Vec<Vec<SyncMessage>>>>,
    ) -> Self {
        let (tx, rx) = mpsc::sync_channel(capacity);
        let paused = Arc::new((Mutex::new(false), Condvar::new()));
        let worker_paused = paused.clone();
        let handle = thread::spawn(move || {
            run_worker_logged(engine, rx, worker_paused, batch_delay, log)
        });
        Self {
            tx: Some(tx),
            handle: Some(handle),
            paused,
        }
    }
}

fn run_worker(
    mut engine: SyncEngine,
    rx: Receiver<Option<SyncMessage>>,
    paused: Arc<(Mutex<bool>, Condvar)>,
    batch_delay: Duration,
) {
    while let Ok(first) = rx.recv() {
        let first = match first {
            Some(msg) => msg,
            None => break,
        };
        {
            let (lock, cvar) = &*paused;
            let mut paused_guard = lock.lock().unwrap();
            while *paused_guard {
                paused_guard = cvar.wait(paused_guard).unwrap();
            }
        }
        let mut batch = vec![first];
        loop {
            match rx.recv_timeout(batch_delay) {
                Ok(Some(msg)) => {
                    {
                        let (lock, cvar) = &*paused;
                        let mut paused_guard = lock.lock().unwrap();
                        while *paused_guard {
                            batch.clear();
                            paused_guard = cvar.wait(paused_guard).unwrap();
                        }
                    }
                    batch.push(msg);
                }
                Ok(None) => return,
                Err(mpsc::RecvTimeoutError::Timeout) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
            }
        }
        {
            let (lock, cvar) = &*paused;
            let mut paused_guard = lock.lock().unwrap();
            while *paused_guard {
                paused_guard = cvar.wait(paused_guard).unwrap();
            }
        }
        for msg in batch {
            let _ = engine.handle(msg);
        }
    }
}

#[cfg(test)]
fn run_worker_logged(
    mut engine: SyncEngine,
    rx: Receiver<Option<SyncMessage>>,
    paused: Arc<(Mutex<bool>, Condvar)>,
    batch_delay: Duration,
    log: Arc<Mutex<Vec<Vec<SyncMessage>>>>,
) {
    while let Ok(first) = rx.recv() {
        let first = match first {
            Some(msg) => msg,
            None => break,
        };
        {
            let (lock, cvar) = &*paused;
            let mut paused_guard = lock.lock().unwrap();
            while *paused_guard {
                paused_guard = cvar.wait(paused_guard).unwrap();
            }
        }
        let mut batch = vec![first];
        loop {
            match rx.recv_timeout(batch_delay) {
                Ok(Some(msg)) => {
                    {
                        let (lock, cvar) = &*paused;
                        let mut paused_guard = lock.lock().unwrap();
                        while *paused_guard {
                            batch.clear();
                            paused_guard = cvar.wait(paused_guard).unwrap();
                        }
                    }
                    batch.push(msg);
                }
                Ok(None) => return,
                Err(mpsc::RecvTimeoutError::Timeout) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
            }
        }
        {
            let (lock, cvar) = &*paused;
            let mut paused_guard = lock.lock().unwrap();
            while *paused_guard {
                paused_guard = cvar.wait(paused_guard).unwrap();
            }
        }
        log.lock().unwrap().push(batch.clone());
        for msg in batch {
            let _ = engine.handle(msg);
        }
    }
}

impl Drop for AsyncManager {
    fn drop(&mut self) {
        // Ensure the worker thread terminates even if `shutdown` wasn't called.
        self.shutdown_inner();
    }
}
