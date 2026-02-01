use owo_colors::OwoColorize;
use std::sync::atomic::{AtomicBool, Ordering};

/// Global quiet mode flag
static QUIET_MODE: AtomicBool = AtomicBool::new(false);

/// Enable quiet mode (suppress logo, colors, use minimal output)
pub fn set_quiet(quiet: bool) {
    QUIET_MODE.store(quiet, Ordering::SeqCst);
}

/// Check if quiet mode is enabled
pub fn is_quiet() -> bool {
    QUIET_MODE.load(Ordering::SeqCst)
}

const LOGO: &str = r#"
     _
  __| |_ __ ___   __ _ _ __
 / _` | '_ ` _ \ / _` | '__|
| (_| | | | | | | (_| | |
 \__,_|_| |_| |_|\__, |_|
                 |___/
"#;

/// Print the ASCII logo
pub fn logo() {
    if is_quiet() {
        return;
    }
    println!("{}", LOGO.cyan().bold());
}

/// Print the menu header with description and URL
pub fn menu_header() {
    if is_quiet() {
        return;
    }
    logo();
    println!("{}", "macOS app distribution manager".dimmed());
    println!("{}\n", "https://github.com/albertogalca/dmgr".blue());
}

/// Print a step indicator (blue arrow)
pub fn step(message: &str) {
    if is_quiet() {
        println!("==> {}", message);
        return;
    }
    println!("{} {}", "==>".blue().bold(), message.bold());
}

/// Print a success message (green checkmark)
pub fn success(message: &str) {
    if is_quiet() {
        println!("[OK] {}", message);
        return;
    }
    println!("{} {}", "✓".green().bold(), message);
}

/// Print an error message (red X)
pub fn error(message: &str) {
    if is_quiet() {
        eprintln!("[ERROR] {}", message);
        return;
    }
    eprintln!("{} {}", "✗".red().bold(), message.red());
}

/// Print a warning message (yellow)
pub fn warning(message: &str) {
    if is_quiet() {
        println!("[WARN] {}", message);
        return;
    }
    println!("{} {}", "!".yellow().bold(), message.yellow());
}

/// Print an info message (dim)
pub fn info(message: &str) {
    if is_quiet() {
        println!("  {}", message);
        return;
    }
    println!("  {}", message.dimmed());
}

/// Print a header box
pub fn header(title: &str) {
    if is_quiet() {
        // In quiet mode, just print a simple header
        println!("=== {} ===", title);
        return;
    }
    let line = "─".repeat(title.len() + 4);
    println!("{}", format!("┌{}┐", line).cyan());
    println!("{}", format!("│  {}  │", title).cyan().bold());
    println!("{}", format!("└{}┘", line).cyan());
}

/// Print a command that will be/was executed
pub fn command(cmd: &str) {
    if is_quiet() {
        println!("  $ {}", cmd);
        return;
    }
    println!("  {} {}", "$".dimmed(), cmd.dimmed());
}

/// Print a list item
pub fn list_item(label: &str, value: &str) {
    if is_quiet() {
        println!("  {}: {}", label, value);
        return;
    }
    println!("  {} {}", format!("{}:", label).dimmed(), value);
}

/// Print tool availability status
pub fn tool_status(name: &str, available: bool, path: Option<&str>) {
    if is_quiet() {
        if available {
            let path_str = path.map(|p| format!(" ({})", p)).unwrap_or_default();
            println!("  [OK] {}{}", name, path_str);
        } else {
            println!("  [MISSING] {}", name);
        }
        return;
    }
    if available {
        let path_str = path.map(|p| format!(" ({})", p)).unwrap_or_default();
        println!("  {} {}{}", "✓".green(), name, path_str.dimmed());
    } else {
        println!("  {} {} {}", "✗".red(), name, "(not found)".red().dimmed());
    }
}
