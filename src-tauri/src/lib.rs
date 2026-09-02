pub mod emarker;
pub mod history;
pub mod model;
pub mod probe;
pub mod settings;
pub mod verdict;

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::Duration;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{ActivationPolicy, Emitter, Manager, State};
use tauri_plugin_autostart::{ManagerExt, MacosLauncher};
use tauri_plugin_notification::NotificationExt;

use crate::model::Snapshot;
use crate::probe::UsbProbe;
use crate::settings::Settings;
use crate::verdict::{verdicts, Blame, PortVerdict};

struct AppState {
    last: Mutex<Option<Snapshot>>,
    settings: Mutex<Settings>,
    probe: Box<dyn UsbProbe>,
}

/// True when the port set changed (ignores `captured_ms`).
fn changed(old: &Option<Snapshot>, new: &Snapshot) -> bool {
    match old {
        None => true,
        Some(o) => o.ports != new.ports,
    }
}

fn device_total(p: &crate::model::Port) -> usize {
    fn n(d: &crate::model::DeviceNode) -> usize {
        1 + d.children.iter().map(n).sum::<usize>()
    }
    p.devices.iter().map(n).sum()
}

/// One-line status for the menu-bar tooltip.
fn tray_summary(snap: &Snapshot) -> String {
    let active = snap.ports.iter().filter(|p| p.occupied).count();
    let devices: usize = snap.ports.iter().map(device_total).sum();
    let watts = snap
        .ports
        .iter()
        .find_map(|p| p.charger.as_ref().and_then(|c| c.watts));
    let mut s = format!(
        "plugcheck — {active} port{} in use",
        if active == 1 { "" } else { "s" }
    );
    if devices > 0 {
        s.push_str(&format!(
            ", {devices} device{}",
            if devices == 1 { "" } else { "s" }
        ));
    }
    if let Some(w) = watts {
        s.push_str(&format!(", charging {w} W"));
    }
    s
}

/// Fire a notification for each meaningful change between two snapshots.
fn notify_changes(app: &tauri::AppHandle, old: &Snapshot, new: &Snapshot) {
    let vnew = verdicts(new);
    for np in &new.ports {
        let op = old.ports.iter().find(|p| p.id == np.id);
        let was_occupied = op.map(|p| p.occupied).unwrap_or(false);
        let label = np.id.replace("Port-", "").replace('@', " port ");

        if np.occupied && !was_occupied {
            let head = vnew
                .iter()
                .find(|v| v.port_id == np.id)
                .map(|v| v.headline.clone())
                .unwrap_or_else(|| "Device".into());
            let _ = app
                .notification()
                .builder()
                .title(format!("{label} — {head} connected"))
                .show();
        } else if !np.occupied && was_occupied {
            let _ = app
                .notification()
                .builder()
                .title(format!("{label} — disconnected"))
                .show();
        } else if np.occupied {
            // a data bottleneck that wasn't there before
            let now_bad = vnew
                .iter()
                .find(|v| v.port_id == np.id)
                .map(|v| v.data_blame != Blame::None)
                .unwrap_or(false);
            let was_bad = op
                .map(|p| verdicts(&one_port(p)))
                .and_then(|v| v.into_iter().next())
                .map(|v| v.data_blame != Blame::None)
                .unwrap_or(false);
            if now_bad && !was_bad {
                let line = vnew
                    .iter()
                    .find(|v| v.port_id == np.id)
                    .map(|v| v.data_line.clone())
                    .unwrap_or_default();
                let _ = app
                    .notification()
                    .builder()
                    .title(format!("{label} — data speed limited"))
                    .body(line)
                    .show();
            }
        }
    }
}

fn one_port(p: &crate::model::Port) -> Snapshot {
    Snapshot {
        ports: vec![p.clone()],
        captured_ms: 0,
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

/// `plugcheck --json`: probe once, print `{ snapshot, verdicts }`, exit.
pub fn print_json() {
    match make_probe().snapshot() {
        Ok(snap) => {
            let out = serde_json::json!({ "snapshot": snap, "verdicts": verdicts(&snap) });
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}

/// `plugcheck --text` / `--watch`: readable per-port summary.
pub fn print_text() {
    let snap = match make_probe().snapshot() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    let v = verdicts(&snap);
    for p in &snap.ports {
        if !p.occupied {
            println!("{}  —  empty", p.id);
            continue;
        }
        let pv = v.iter().find(|x| x.port_id == p.id);
        println!(
            "\n{}  {}",
            p.id,
            pv.map(|x| x.headline.as_str()).unwrap_or("connected")
        );
        if let Some(s) = pv.map(|x| x.subline.as_str()).filter(|s| !s.is_empty()) {
            println!("  {s}");
        }
        for c in pv.map(|x| x.cards.as_slice()).unwrap_or(&[]) {
            let mark = match c.status {
                crate::verdict::CardStatus::Ok => "\u{2713}",
                crate::verdict::CardStatus::Warn => "!",
                crate::verdict::CardStatus::Bad => "\u{2717}",
                crate::verdict::CardStatus::Idle => "\u{00b7}",
            };
            println!("  {mark} {}", c.head);
            println!("     {}", c.text);
        }
        for t in pv.map(|x| x.trust_flags.as_slice()).unwrap_or(&[]) {
            println!("  ! {t}");
        }
        for b in pv.map(|x| x.cable_details.as_slice()).unwrap_or(&[]) {
            println!("  \u{2022} {b}");
        }
        for d in &p.devices {
            print_dev(d, 2);
        }
    }
}

fn print_dev(d: &crate::model::DeviceNode, indent: usize) {
    let pad = " ".repeat(indent);
    let mut meta = Vec::new();
    if let Some(c) = &d.class {
        meta.push(c.clone());
    }
    if let Some(v) = &d.vendor {
        meta.push(v.clone());
    }
    if let Some(vp) = &d.vid_pid {
        meta.push(vp.clone());
    }
    println!("{pad}- {} ({})", d.name, meta.join(" · "));
    for c in &d.children {
        print_dev(c, indent + 2);
    }
}

fn show_main(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

fn apply_settings(app: &tauri::AppHandle, s: &Settings) {
    let _ = app.set_activation_policy(if s.menu_bar_only {
        ActivationPolicy::Accessory
    } else {
        ActivationPolicy::Regular
    });
    let mgr = app.autolaunch();
    let _ = if s.launch_at_login {
        mgr.enable()
    } else {
        mgr.disable()
    };
}

#[tauri::command]
fn get_snapshot(app: tauri::AppHandle, state: State<AppState>) -> Result<Snapshot, String> {
    let mut snap = state.probe.snapshot().map_err(|e| e.to_string())?;
    history::touch(&app, &mut snap);
    *state.last.lock().unwrap() = Some(snap.clone());
    Ok(snap)
}

#[tauri::command]
fn get_verdicts(state: State<AppState>) -> Result<Vec<PortVerdict>, String> {
    let snap = state.probe.snapshot().map_err(|e| e.to_string())?;
    Ok(verdicts(&snap))
}

#[tauri::command]
fn rename_cable(app: tauri::AppHandle, sig: String, name: Option<String>) {
    history::rename(&app, &sig, name);
}

#[tauri::command]
fn saved_cables(app: tauri::AppHandle) -> Vec<history::SavedCable> {
    history::list(&app)
}

#[tauri::command]
fn forget_cable(app: tauri::AppHandle, sig: String) {
    history::forget(&app, &sig);
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

#[tauri::command]
fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[tauri::command]
fn get_settings(state: State<AppState>) -> Settings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
fn set_settings(app: tauri::AppHandle, state: State<AppState>, next: Settings) -> Result<(), String> {
    settings::save(&app, &next)?;
    apply_settings(&app, &next);
    *state.settings.lock().unwrap() = next;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = env_logger::try_init();

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let handle = app.handle().clone();
            let cfg = settings::load(&handle);
            apply_settings(&handle, &cfg);

            app.manage(AppState {
                last: Mutex::new(None),
                settings: Mutex::new(cfg),
                probe: make_probe(),
            });

            // --- menu-bar tray ---
            let open_i = MenuItem::with_id(app, "open", "Open plugcheck", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_i, &quit_i])?;
            let tray_icon = tauri::image::Image::from_bytes(include_bytes!(
                "../icons/tray@2x.png"
            ))?;
            let tray = TrayIconBuilder::with_id("plugcheck")
                .icon(tray_icon)
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
            let poll_handle = handle.clone();
            std::thread::spawn(move || loop {
                let secs = poll_handle
                    .state::<AppState>()
                    .settings
                    .lock()
                    .unwrap()
                    .poll_secs();
                std::thread::sleep(Duration::from_secs(secs));

                let state = poll_handle.state::<AppState>();
                let Ok(mut snap) = state.probe.snapshot() else {
                    continue;
                };
                history::touch(&poll_handle, &mut snap);
                let _ = tray.set_tooltip(Some(tray_summary(&snap)));

                let mut last = state.last.lock().unwrap();
                if changed(&last, &snap) {
                    let prev = last.replace(snap.clone());
                    let notify = state.settings.lock().unwrap().notifications;
                    drop(last);
                    if notify {
                        if let Some(prev) = prev {
                            notify_changes(&poll_handle, &prev, &snap);
                        }
                    }
                    let _ = poll_handle.emit("snapshot-changed", snap);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            get_verdicts,
            engineer_dump,
            get_settings,
            set_settings,
            rename_cable,
            saved_cables,
            forget_cable,
            app_version
        ])
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
        assert_eq!(tray_summary(&empty(0)), "plugcheck — 0 ports in use");
    }

    #[test]
    fn settings_poll_clamped() {
        let mut s = Settings::default();
        s.poll_secs = 0;
        assert_eq!(s.poll_secs(), 1);
        s.poll_secs = 999;
        assert_eq!(s.poll_secs(), 60);
    }
}
