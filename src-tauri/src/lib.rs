pub mod emarker;
pub mod model;
pub mod probe;
pub mod verdict;

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::Duration;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
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

/// One-line status for the menu-bar tooltip.
fn tray_summary(snap: &Snapshot) -> String {
    let active = snap.ports.iter().filter(|p| p.occupied).count();
    let devices: usize = snap
        .ports
        .iter()
        .map(|p| {
            fn n(d: &crate::model::DeviceNode) -> usize {
                1 + d.children.iter().map(n).sum::<usize>()
            }
            p.devices.iter().map(n).sum::<usize>()
        })
        .sum();
    let watts = snap
        .ports
        .iter()
        .find_map(|p| p.charger.as_ref().and_then(|c| c.watts));
    let mut s = format!(
        "plugcheck — {active} port{} in use",
        if active == 1 { "" } else { "s" }
    );
    if devices > 0 {
        s.push_str(&format!(", {devices} device{}", if devices == 1 { "" } else { "s" }));
    }
    if let Some(w) = watts {
        s.push_str(&format!(", charging {w} W"));
    }
    s
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

/// `plugcheck --json`: probe once, print `{ snapshot, verdicts }`, exit.
pub fn print_json() {
    let probe = make_probe();
    match probe.snapshot() {
        Ok(snap) => {
            let out = serde_json::json!({
                "snapshot": snap,
                "verdicts": verdicts(&snap),
            });
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}

fn show_main(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
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
            // --- menu-bar tray ---
            let open_i = MenuItem::with_id(app, "open", "Open plugcheck", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_i, &quit_i])?;
            let tray = TrayIconBuilder::with_id("plugcheck")
                .icon(app.default_window_icon().cloned().unwrap())
                .icon_as_template(true)
                .tooltip("plugcheck")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show_main(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main(tray.app_handle());
                    }
                })
                .build(app)?;

            // --- poll thread ---
            let handle = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(POLL_INTERVAL);
                let state = handle.state::<AppState>();
                let Ok(snap) = state.probe.snapshot() else {
                    continue;
                };
                let _ = tray.set_tooltip(Some(tray_summary(&snap)));
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

    #[test]
    fn tray_summary_reads_well() {
        let s = empty(0);
        assert_eq!(tray_summary(&s), "plugcheck — 0 ports in use");
    }
}
