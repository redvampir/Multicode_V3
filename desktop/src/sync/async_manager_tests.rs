use super::{
    AsyncManager, SyncEngine, SyncMessage, SyncSettings, DEFAULT_BATCH_DELAY,
    DEFAULT_CHANNEL_CAPACITY,
};
use multicode_core::parser::Lang;
use std::sync::{Arc, Mutex};

#[test]
fn messages_within_delay_processed_in_one_batch() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let engine = SyncEngine::new(Lang::Rust, SyncSettings::default());
    let manager = AsyncManager::new_with_logger(
        engine,
        DEFAULT_BATCH_DELAY,
        DEFAULT_CHANNEL_CAPACITY,
        log.clone(),
        true,
    );
    manager
        .send(SyncMessage::TextChanged("a".into(), Lang::Rust))
        .unwrap();
    manager
        .send(SyncMessage::TextChanged("b".into(), Lang::Rust))
        .unwrap();
    manager
        .send(SyncMessage::TextChanged("c".into(), Lang::Rust))
        .unwrap();
    std::thread::sleep(DEFAULT_BATCH_DELAY * 3);
    manager.shutdown();
    let batches = log.lock().unwrap();
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].len(), 3);
}

#[test]
fn pause_stops_processing_and_resume_restarts() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let engine = SyncEngine::new(Lang::Rust, SyncSettings::default());
    let manager = AsyncManager::new_with_logger(
        engine,
        DEFAULT_BATCH_DELAY,
        DEFAULT_CHANNEL_CAPACITY,
        log.clone(),
        true,
    );
    manager.pause();
    manager
        .send(SyncMessage::TextChanged("a".into(), Lang::Rust))
        .unwrap();
    manager
        .send(SyncMessage::TextChanged("b".into(), Lang::Rust))
        .unwrap();
    std::thread::sleep(DEFAULT_BATCH_DELAY * 3);
    {
        let batches = log.lock().unwrap();
        assert!(batches.is_empty());
    }
    manager.resume();
    std::thread::sleep(DEFAULT_BATCH_DELAY * 3);
    manager.shutdown();
    let batches = log.lock().unwrap();
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].len(), 2);
}
