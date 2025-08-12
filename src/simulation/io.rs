use super::*;
use crate::io::bin::BinIO;
use crate::io::{DataCategory, Persistable};
use std::sync::mpsc::Receiver;
use std::thread;

impl Persistable for SimulationResult {
    fn id(&self) -> usize {
        self.id
    }

    fn tag(&self) -> usize {
        self.tag
    }

    fn data_type(&self) -> DataCategory {
        DataCategory::Simulation
    }
}

pub struct SimulationResultReceiver {
    pub id: usize,
    pub tag: usize,
    pub ensemble_entry_id: usize,
    pub params: SimulationParams,
}

impl SimulationResultReceiver {
    pub fn start_receiver_thread(
        self,
        rx: Receiver<SimulationSnapshot>
    ) -> thread::JoinHandle<Result<(), String>> {
        thread::spawn(move || {
            let mut snapshots = Vec::new();
            
            while let Ok(snapshot) = rx.recv() {
                snapshots.push(snapshot);
            }
            
            let total_steps = snapshots.last().map(|s| s.step).unwrap_or(0);
            
            let result = SimulationResult {
                id: self.id,
                tag: self.tag,
                ensemble_entry_id: self.ensemble_entry_id,
                params: self.params,
                snapshots,
                total_steps,
            };
            
            BinIO::save(&result).map_err(|e| e.to_string())?;
            Ok(())
        })
    }
}