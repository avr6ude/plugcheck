// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let has = |f: &str| args.iter().any(|a| a == f);

    if has("--help") || has("-h") {
        eprintln!("PlugCheck — USB-C / Thunderbolt inspector\n");
        eprintln!("  plugcheck            launch the app");
        eprintln!("  plugcheck --text     print a readable snapshot and exit");
        eprintln!("  plugcheck --json     print snapshot + verdicts as JSON and exit");
        eprintln!("  plugcheck --watch    refresh the readable snapshot every 2s");
        eprintln!("  plugcheck --raw      with --text/--watch: include raw IOKit properties");
        eprintln!("  plugcheck --version  print the version");
        return;
    }
    if has("--version") || has("-V") {
        println!("PlugCheck {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let raw = has("--raw");
    if has("--json") || has("-j") {
        plugcheck_lib::print_json();
        return;
    }
    if has("--watch") || has("--dashboard") {
        let period = if has("--dashboard") { 1 } else { 2 };
        loop {
            print!("\x1b[2J\x1b[H"); // clear
            println!("PlugCheck — live  (Ctrl-C to quit)\n");
            plugcheck_lib::print_text(raw);
            std::thread::sleep(std::time::Duration::from_secs(period));
        }
    }
    if has("--text") || has("-t") || raw {
        plugcheck_lib::print_text(raw);
        return;
    }
    plugcheck_lib::run()
}
