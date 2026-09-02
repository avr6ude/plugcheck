pub mod emarker;
pub mod model;
pub mod probe;
pub mod verdict;

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::Duration;

use tauri::{Emitter, Manager, State};

use crate::model::Snapshot;
use crate::probe::UsbProbe;
use crate::verdict::{verdicts, PortVerdict};

const POLL_INTERVAL: Duration = Duration::from_secs(3);

struct AppState {
    last: Mutex<Option<Snapshot>>,
    probe: Box<dyn UsbProbe>,
}

/// True when the port set changed (ignores `captured_ms`).
fn changed(old: &Option<Snapshot>, new: &Snapshot) -> bool {
    match old {
        None => true,
        Some(o) => o.ports != new.ports,
    }
}

#[cfg(target_os = "macos")]
fn make_probe() -> Box<dyn UsbProbe> {
    Box::new(crate::probe::macos::MacosProbe)
}

#[cfg(not(target_os = "macos"))]
fn make_probe() -> Box<dyn UsbProbe> {
    struct Unsupported;
    impl UsbProbe for Unsupported {
        fn snapshot(&self) -> Result<Snapshot, crate::model::ProbeError> {
            Err(crate::model::ProbeError::Unsupported)
        }
    }
    Box::new(Unsupported)
}

#[tauri::command]
fn get_snapshot(state: State<AppState>) -> Result<Snapshot, String> {
    let snap = state.probe.snapshot().map_err(|e| e.to_string())?;
    *state.last.lock().unwrap() = Some(snap.clone());
    Ok(snap)
}

#[tauri::command]
fn get_verdicts(state: State<AppState>) -> Result<Vec<PortVerdict>, String> {
    let snap = state.probe.snapshot().map_err(|e| e.to_string())?;
    Ok(verdicts(&snap))
}

#[tauri::command]
fn engineer_dump(
    port_id: String,
    state: State<AppState>,
) -> Result<BTreeMap<String, String>, String> {
    let snap = state.probe.snapshot().map_err(|e| e.to_string())?;
    snap.ports
        .into_iter()
        .find(|p| p.id == port_id)
        .map(|p| p.raw)
        .ok_or_else(|| format!("no port {port_id}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = env_logger::try_init();

    tauri::Builder::default()
        .manage(AppState {
            last: Mutex::new(None),
            probe: make_probe(),
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            get_verdicts,
            engineer_dump
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            // Plain polling thread — no async runtime needed for a 3 s tick.
            std::thread::spawn(move || loop {
                std::thread::sleep(POLL_INTERVAL);
                let state = handle.state::<AppState>();
                let Ok(snap) = state.probe.snapshot() else {
                    continue;
                };
                let mut last = state.last.lock().unwrap();
                if changed(&last, &snap) {
                    *last = Some(snap.clone());
                    drop(last);
                    let _ = handle.emit("snapshot-changed", snap);
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty(ms: u64) -> Snapshot {
        Snapshot {
            ports: vec![],
            captured_ms: ms,
        }
    }

    #[test]
    fn changed_ignores_captured_ms() {
        assert!(!changed(&Some(empty(1)), &empty(999)));
    }

    #[test]
    fn changed_true_when_no_prior() {
        assert!(changed(&None, &empty(0)));
    }
}
