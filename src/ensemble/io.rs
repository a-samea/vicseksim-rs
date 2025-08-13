use super::*;
use crate::io::{DataPersistence, DataType, bin};
use std::sync::mpsc::Receiver;
use std::thread;

/// Implementation of the [`DataPersistence`] trait for [`EntryResult`] structures.
///
/// This implementation enables automatic serialization and file management for ensemble
/// data through the unified I/O system. It defines how ensemble entries are categorized,
/// identified, and persisted to disk storage.
impl DataPersistence for EntryResult {
    /// Returns the data type identifier for ensemble entries.
    fn data_type() -> DataType {
        DataType::Ensemble
    }

    /// Returns the unique identifier for this ensemble entry.
    fn id(&self) -> usize {
        self.id
    }

    /// Returns the tag identifier for this ensemble entry.
    fn tag(&self) -> usize {
        self.tag
    }
}

/// Starts a dedicated I/O thread for concurrent ensemble data persistence.
///
/// This function creates a separate thread that continuously receives completed
/// [`EntryResult`] instances from parallel generation workers and saves them to
/// disk using the binary serialization system. This architecture prevents I/O
/// operations from blocking the CPU-intensive ensemble generation process.
///
/// # Architecture Benefits
///
/// - **Non-blocking Generation**: Ensemble generation threads can immediately
///   continue with new work after sending results
/// - **Concurrent I/O**: File writing happens in parallel with generation
/// - **Memory Efficiency**: Results are processed and released as soon as received
/// - **Error Isolation**: I/O failures don't crash generation workers
///
/// # Thread Lifecycle
///
/// 1. **Initialization**: Creates new thread with moved receiver ownership
/// 2. **Processing Loop**: Continuously receives and saves [`EntryResult`] instances
/// 3. **Termination**: Exits cleanly when all senders are dropped (channel closed)
/// 4. **Cleanup**: Thread join handle allows main thread to wait for completion
///
/// # Arguments
///
/// * `rx` - MPSC receiver for [`EntryResult`] instances from generation workers.
///          The receiver is moved into the thread for exclusive ownership.
///
/// # Returns
///
/// [`thread::JoinHandle<Result<(), String>>`] - Handle for waiting on thread completion.
/// The wrapped `Result` indicates whether all I/O operations succeeded:
/// - `Ok(())` - All ensemble entries saved successfully
/// - `Err(String)` - Descriptive error message for any I/O failures
///
/// # Error Handling
///
/// The I/O thread will continue processing as long as it can receive data, but will
/// terminate and return an error if any file save operation fails. This ensures
/// data integrity while maximizing successful saves.
pub(super) fn start_receiver_thread(
    rx: Receiver<EntryResult>,
) -> thread::JoinHandle<Result<(), String>> {
    thread::spawn(move || {
        // Continuously process ensemble results until channel closes
        while let Ok(entry_result) = rx.recv() {
            // Save each ensemble entry using binary serialization
            // Convert any I/O error to string for consistent error handling
            bin::save_file(&entry_result).map_err(|e| e.to_string())?;
        }
        Ok(())
    })
}
