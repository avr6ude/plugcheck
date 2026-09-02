// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let has = |f: &str| args.iter().any(|a| a == f);

    if has("--help") || has("-h") {
        eprintln!("plugcheck — USB-C / Thunderbolt inspector\n");
        eprintln!("  plugcheck            launch the app");
        eprintln!("  plugcheck --text     print a readable snapshot and exit");
        eprintln!("  plugcheck --json     print snapshot + verdicts as JSON and exit");
        eprintln!("  plugcheck --watch    refresh the readable snapshot every 2s");
        return;
    }
    if has("--json") || has("-j") {
        plugcheck_lib::print_json();
        return;
    }
    if has("--watch") {
        loop {
            print!("\x1b[2J\x1b[H"); // clear
            plugcheck_lib::print_text();
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }
    if has("--text") || has("-t") {
        plugcheck_lib::print_text();
        return;
    }
    plugcheck_lib::run()
}
