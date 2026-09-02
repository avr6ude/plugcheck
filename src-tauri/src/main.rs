// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--json" || a == "-j") {
        plugcheck_lib::print_json();
        return;
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        eprintln!("plugcheck — USB-C / Thunderbolt inspector\n");
        eprintln!("  plugcheck            launch the app");
        eprintln!("  plugcheck --json     print a snapshot + verdicts as JSON and exit");
        return;
    }
    plugcheck_lib::run()
}
