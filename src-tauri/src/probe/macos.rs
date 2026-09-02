//! macOS probe. Filled in by plan Task 5/6.

use crate::model::{ProbeError, Snapshot};

pub struct MacosProbe;

impl super::UsbProbe for MacosProbe {
    fn snapshot(&self) -> Result<Snapshot, ProbeError> {
        Err(ProbeError::Unsupported)
    }
}
