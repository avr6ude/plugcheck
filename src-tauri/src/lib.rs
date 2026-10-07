pub mod emarker;
pub mod faults;
pub mod history;
pub mod model;
pub mod probe;
pub mod settings;
pub mod update;
pub mod verdict;

use std::sync::Mutex;
use std::time::Duration;

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{ActivationPolicy, Emitter, Manager, State};
use tauri_plugin_autostart::{ManagerExt, MacosLauncher};
use tauri_plugin_notification::NotificationExt;

use crate::faults::port_label;
use crate::model::{Port, Snapshot};
use crate::probe::UsbProbe;
use crate::settings::Settings;
use crate::verdict::{verdicts, Blame, PortVerdict};

struct AppState {
    last: Mutex<Option<Snapshot>>,
    update: Mutex<Option<update::Update>>,
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
        let label = port_label(np);

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

/// One menu line per port, e.g. "USB-C 3 — LG ULTRAGEAR".
fn port_line(p: &Port, v: Option<&PortVerdict>) -> String {
    let label = port_label(p);
    if !p.occupied {
        return format!("{label} — Not connected");
    }
    let what = p
        .display
        .as_ref()
        .map(|d| d.name.clone())
        .or_else(|| p.devices.first().map(|d| d.name.clone()))
        .or_else(|| v.map(|v| v.headline.clone()))
        .unwrap_or_else(|| "Connected".into());
    match p.charger.as_ref().and_then(|c| c.live_watts.or(c.watts.map(f32::from))) {
        Some(w) => format!("{label} — {what}, {w:.0} W"),
        None => format!("{label} — {what}"),
    }
}

fn charging_watts(snap: &Snapshot) -> Option<f32> {
    snap.ports
        .iter()
        .find_map(|p| p.charger.as_ref().filter(|c| c.is_charging).and_then(|c| c.live_watts.or(c.watts.map(f32::from))))
}

fn tray_menu(app: &tauri::AppHandle, snap: Option<&Snapshot>) -> tauri::Result<Menu<tauri::Wry>> {
    let menu = Menu::new(app)?;
    if let Some(snap) = snap {
        let v = verdicts(snap);
        for p in &snap.ports {
            let line = port_line(p, v.iter().find(|x| x.port_id == p.id));
            menu.append(&MenuItem::with_id(app, format!("port:{}", p.id), line, false, None::<&str>)?)?;
        }
        menu.append(&PredefinedMenuItem::separator(app)?)?;
    }
    menu.append(&MenuItem::with_id(app, "refresh", "Refresh", true, Some("CmdOrCtrl+R"))?)?;
    menu.append(&MenuItem::with_id(app, "settings", "Settings…", true, Some("CmdOrCtrl+,"))?)?;
    menu.append(&MenuItem::with_id(app, "updates", "Check for Updates…", true, None::<&str>)?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(app, "open", "Open plugcheck", true, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(app, "quit", "Quit plugcheck", true, Some("CmdOrCtrl+Q"))?)?;
    Ok(menu)
}

/// Refresh the tray's menu, tooltip and (optionally) the live charging wattage beside the icon.
fn update_tray(app: &tauri::AppHandle, snap: &Snapshot) {
    let Some(tray) = app.tray_by_id("plugcheck") else { return };
    let _ = tray.set_tooltip(Some(tray_summary(snap)));
    if let Ok(menu) = tray_menu(app, Some(snap)) {
        let _ = tray.set_menu(Some(menu));
    }
    let show = app.state::<AppState>().settings.lock().unwrap().menu_bar_watts;
    let title = show.then(|| charging_watts(snap)).flatten().map(|w| format!("{w:.0} W"));
    let _ = tray.set_title(title);
}

fn open_url(url: &str) {
    let _ = std::process::Command::new("/usr/bin/open").arg(url).spawn();
}

/// Background check; `manual` also reports "up to date" and opens the release page.
fn check_updates(app: &tauri::AppHandle, manual: bool) {
    let found = update::check();
    let state = app.state::<AppState>();
    let known = state.update.lock().unwrap().clone();
    *state.update.lock().unwrap() = found.clone();
    match found {
        Some(u) if manual => open_url(&u.url),
        Some(u) => {
            let _ = app.emit("update-available", &u);
            if known.as_ref() != Some(&u) {
                let _ = app
                    .notification()
                    .builder()
                    .title(format!("plugcheck {} is available", u.version))
                    .body("Open plugcheck to download it.")
                    .show();
            }
        }
        None if manual => {
            let _ = app
                .notification()
                .builder()
                .title("plugcheck is up to date")
                .body(format!("Version {}", env!("CARGO_PKG_VERSION")))
                .show();
        }
        None => {}
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

/// `plugcheck --text` / `--watch`: readable per-port summary; `raw` adds IOKit properties.
pub fn print_text(raw: bool) {
    let snap = match make_probe().snapshot() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    let v = verdicts(&snap);
    for p in &snap.ports {
        let label = port_label(p);
        if !p.occupied {
            println!("{label}  —  not connected");
            continue;
        }
        let pv = v.iter().find(|x| x.port_id == p.id);
        println!(
            "\n{label}  {}",
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
        if raw {
            for (k, val) in &p.raw {
                println!("    {k} = {val}");
            }
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
fn get_update(state: State<AppState>) -> Option<update::Update> {
    state.update.lock().unwrap().clone()
}

#[tauri::command]
fn open_release(url: String) -> Result<(), String> {
    // Only our own release pages; never an arbitrary URL from the webview.
    if !url.starts_with("https://github.com/avr6ude/plugcheck/") {
        return Err("not a plugcheck release URL".into());
    }
    open_url(&url);
    Ok(())
}

const CLI_LINK: &str = "/usr/local/bin/plugcheck";

/// Whether `plugcheck` on the PATH already points at this app.
#[tauri::command]
fn cli_installed() -> bool {
    let exe = std::env::current_exe().ok();
    std::fs::read_link(CLI_LINK).ok().is_some_and(|t| Some(t) == exe)
}

/// Symlink the app binary to /usr/local/bin/plugcheck. macOS shows its own
/// administrator prompt; plugcheck never sees the password.
#[tauri::command]
fn install_cli() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe = exe.to_string_lossy();
    if exe.contains(['\'', '"', '\\']) {
        return Err("app path contains quotes; move plugcheck to /Applications".into());
    }
    let script = format!(
        "do shell script \"mkdir -p /usr/local/bin && ln -sf '{exe}' {CLI_LINK}\" with administrator privileges with prompt \"plugcheck wants to install its command-line tool.\""
    );
    let out = std::process::Command::new("/usr/bin/osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
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
    if let Some(snap) = state.last.lock().unwrap().clone() {
        update_tray(&app, &snap); // menu-bar watts toggle applies at once
    }
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
                update: Mutex::new(None),
                settings: Mutex::new(cfg),
                probe: make_probe(),
            });

            // --- menu-bar tray ---
            let menu = tray_menu(&handle, None)?;
            let tray_icon = tauri::image::Image::from_bytes(include_bytes!(
                "../icons/tray@2x.png"
            ))?;
            TrayIconBuilder::with_id("plugcheck")
                .icon(tray_icon)
                .icon_as_template(true)
                .tooltip("plugcheck")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show_main(app),
                    "quit" => app.exit(0),
                    "refresh" | "settings" => {
                        show_main(app);
                        let _ = app.emit("menu", event.id.as_ref());
                    }
                    "updates" => {
                        let app = app.clone();
                        std::thread::spawn(move || check_updates(&app, true));
                    }
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
                update_tray(&poll_handle, &snap);

                let mut last = state.last.lock().unwrap();
                if changed(&last, &snap) {
                    let prev = last.replace(snap.clone());
                    let notify = state.settings.lock().unwrap().notifications;
                    drop(last);
                    if let Some(prev) = prev {
                        let found = faults::faults(&prev, &snap);
                        if !found.is_empty() {
                            // Faults always reach the window; notifications follow the setting.
                            let _ = poll_handle.emit("faults", &found);
                            for f in found.iter().filter(|_| notify) {
                                let _ = poll_handle.notification().builder().title(&f.title).body(&f.text).show();
                            }
                        }
                        if notify {
                            notify_changes(&poll_handle, &prev, &snap);
                        }
                    }
                    let _ = poll_handle.emit("snapshot-changed", snap);
                }
            });

            // --- update check: at launch, then every 6 hours ---
            let update_handle = handle.clone();
            std::thread::spawn(move || loop {
                if update_handle.state::<AppState>().settings.lock().unwrap().update_checks {
                    check_updates(&update_handle, false);
                }
                std::thread::sleep(Duration::from_secs(6 * 60 * 60));
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            get_verdicts,
            get_settings,
            set_settings,
            rename_cable,
            saved_cables,
            forget_cable,
            get_update,
            open_release,
            cli_installed,
            install_cli,
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
