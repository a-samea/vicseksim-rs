//! IO submodule for ensemble data persistence

use super::*;
use crate::io::bin::BinIO;
use crate::io::{DataCategory, DataChannel, Persistable};
use std::sync::mpsc::Receiver;
use std::thread;

impl Persistable for EntryResult {
    fn id(&self) -> usize {
        self.id
    }

    fn tag(&self) -> usize {
        self.tag
    }

    fn data_type(&self) -> DataCategory {
        DataCategory::Ensemble
    }
}

pub struct EntryResultReceiver;
impl DataChannel<EntryResult> for EntryResultReceiver {
    fn start_receiver_thread(rx: Receiver<EntryResult>) -> thread::JoinHandle<Result<(), String>> {
        thread::spawn(move || {
            // Process each ensemble result as it arrives
            while let Ok(entry_result) = rx.recv() {
                // Save to file using the tag
                BinIO::save(&entry_result).map_err(|e| e.to_string())?;
            }
            Ok(())
        })
    }
}
