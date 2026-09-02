//! The probe boundary. macOS is the only implementation today; `linux` /
//! `windows` slot in here later without touching `verdict` or the UI.

use crate::model::{ProbeError, Snapshot};

#[cfg(target_os = "macos")]
pub mod macos;

pub trait UsbProbe: Send + Sync {
    fn snapshot(&self) -> Result<Snapshot, ProbeError>;
}
