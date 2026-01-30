use owo_colors::OwoColorize;

/// Print a step indicator (blue arrow)
pub fn step(message: &str) {
    println!("{} {}", "==>".blue().bold(), message.bold());
}

/// Print a success message (green checkmark)
pub fn success(message: &str) {
    println!("{} {}", "✓".green().bold(), message);
}

/// Print an error message (red X)
pub fn error(message: &str) {
    eprintln!("{} {}", "✗".red().bold(), message.red());
}

/// Print a warning message (yellow)
pub fn warning(message: &str) {
    println!("{} {}", "!".yellow().bold(), message.yellow());
}

/// Print an info message (dim)
pub fn info(message: &str) {
    println!("  {}", message.dimmed());
}

/// Print a header box
pub fn header(title: &str) {
    let line = "─".repeat(title.len() + 4);
    println!("{}", format!("┌{}┐", line).cyan());
    println!("{}", format!("│  {}  │", title).cyan().bold());
    println!("{}", format!("└{}┘", line).cyan());
}

/// Print a command that will be/was executed
pub fn command(cmd: &str) {
    println!("  {} {}", "$".dimmed(), cmd.dimmed());
}

/// Print a list item
pub fn list_item(label: &str, value: &str) {
    println!("  {} {}", format!("{}:", label).dimmed(), value);
}

/// Print tool availability status
pub fn tool_status(name: &str, available: bool, path: Option<&str>) {
    if available {
        let path_str = path.map(|p| format!(" ({})", p)).unwrap_or_default();
        println!("  {} {}{}", "✓".green(), name, path_str.dimmed());
    } else {
        println!("  {} {} {}", "✗".red(), name, "(not found)".red().dimmed());
    }
}
