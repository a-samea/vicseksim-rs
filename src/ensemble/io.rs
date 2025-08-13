use super::*;
use crate::io::{bin, DataPersistence, DataType};
use std::sync::mpsc::Receiver;
use std::thread;

impl DataPersistence for EntryResult {
    fn data_type() -> DataType {
        DataType::Ensemble
    }

    fn id(&self) -> usize {
        self.id
    }

    fn tag(&self) -> usize {
        self.tag
    }
}

pub fn start_receiver_thread(rx: Receiver<EntryResult>) -> thread::JoinHandle<Result<(), String>> {
    thread::spawn(move || {
        while let Ok(entry_result) = rx.recv() {
            bin::save_file(&entry_result).map_err(|e| e.to_string())?;
        }
        Ok(())
    })
}
