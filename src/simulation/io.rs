use super::*;
use crate::io::{DataPersistence, DataType, bin};
use std::sync::mpsc::Receiver;
use std::thread;

impl DataPersistence for SimulationResult {
    fn data_type() -> DataType {
        DataType::Simulation
    }

    fn id(&self) -> usize {
        self.id
    }

    fn tag(&self) -> usize {
        self.tag
    }
}

pub fn start_receiver_thread(
    rx: Receiver<SimulationSnapshot>,
    params: SimulationParams,
    id: usize,
    tag: usize,
    ensemble_entry_id: usize,
) -> thread::JoinHandle<Result<(), String>> {
    thread::spawn(move || {
        let expected_snapshots =
            (params.total_iterations + params.frame_interval - 1) / params.frame_interval;
        let mut snapshots = Vec::with_capacity(expected_snapshots);

        while let Ok(snapshot) = rx.recv() {
            snapshots.push(snapshot);
        }

        let result = SimulationResult {
            id,
            tag,
            ensemble_entry_id,
            params,
            snapshots,
        };

        bin::save_file(&result).map_err(|e| e.to_string())?;
        Ok(())
    })
}
