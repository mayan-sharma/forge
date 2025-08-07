use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Rgb(u8, u8, u8),
}

impl Color {
    pub fn to_ansi_fg(&self) -> String {
        match self {
            Color::Black => "\x1B[30m".to_string(),
            Color::Red => "\x1B[31m".to_string(),
            Color::Green => "\x1B[32m".to_string(),
            Color::Yellow => "\x1B[33m".to_string(),
            Color::Blue => "\x1B[34m".to_string(),
            Color::Magenta => "\x1B[35m".to_string(),
            Color::Cyan => "\x1B[36m".to_string(),
            Color::White => "\x1B[37m".to_string(),
            Color::BrightBlack => "\x1B[90m".to_string(),
            Color::BrightRed => "\x1B[91m".to_string(),
            Color::BrightGreen => "\x1B[92m".to_string(),
            Color::BrightYellow => "\x1B[93m".to_string(),
            Color::BrightBlue => "\x1B[94m".to_string(),
            Color::BrightMagenta => "\x1B[95m".to_string(),
            Color::BrightCyan => "\x1B[96m".to_string(),
            Color::BrightWhite => "\x1B[97m".to_string(),
            Color::Rgb(r, g, b) => format!("\x1B[38;2;{};{};{}m", r, g, b),
        }
    }

    pub fn to_ansi_bg(&self) -> String {
        match self {
            Color::Black => "\x1B[40m".to_string(),
            Color::Red => "\x1B[41m".to_string(),
            Color::Green => "\x1B[42m".to_string(),
            Color::Yellow => "\x1B[43m".to_string(),
            Color::Blue => "\x1B[44m".to_string(),
            Color::Magenta => "\x1B[45m".to_string(),
            Color::Cyan => "\x1B[46m".to_string(),
            Color::White => "\x1B[47m".to_string(),
            Color::BrightBlack => "\x1B[100m".to_string(),
            Color::BrightRed => "\x1B[101m".to_string(),
            Color::BrightGreen => "\x1B[102m".to_string(),
            Color::BrightYellow => "\x1B[103m".to_string(),
            Color::BrightBlue => "\x1B[104m".to_string(),
            Color::BrightMagenta => "\x1B[105m".to_string(),
            Color::BrightCyan => "\x1B[106m".to_string(),
            Color::BrightWhite => "\x1B[107m".to_string(),
            Color::Rgb(r, g, b) => format!("\x1B[48;2;{};{};{}m", r, g, b),
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum Style {
    Reset,
    Bold,
    Dim,
    Italic,
    Underline,
    Blink,
    Reverse,
    Strikethrough,
}

impl Style {
    pub fn to_ansi(&self) -> &'static str {
        match self {
            Style::Reset => "\x1B[0m",
            Style::Bold => "\x1B[1m",
            Style::Dim => "\x1B[2m",
            Style::Italic => "\x1B[3m",
            Style::Underline => "\x1B[4m",
            Style::Blink => "\x1B[5m",
            Style::Reverse => "\x1B[7m",
            Style::Strikethrough => "\x1B[9m",
        }
    }
}

pub struct StyledText {
    text: String,
    fg_color: Option<Color>,
    bg_color: Option<Color>,
    styles: Vec<Style>,
}

impl StyledText {
    pub fn new(text: &str) -> Self {
        StyledText {
            text: text.to_string(),
            fg_color: None,
            bg_color: None,
            styles: Vec::new(),
        }
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.fg_color = Some(color);
        self
    }

    #[allow(dead_code)]
    pub fn bg(mut self, color: Color) -> Self {
        self.bg_color = Some(color);
        self
    }

    pub fn bold(mut self) -> Self {
        self.styles.push(Style::Bold);
        self
    }

    pub fn dim(mut self) -> Self {
        self.styles.push(Style::Dim);
        self
    }

    #[allow(dead_code)]
    pub fn italic(mut self) -> Self {
        self.styles.push(Style::Italic);
        self
    }

    #[allow(dead_code)]
    pub fn underline(mut self) -> Self {
        self.styles.push(Style::Underline);
        self
    }

    #[allow(dead_code)]
    pub fn blink(mut self) -> Self {
        self.styles.push(Style::Blink);
        self
    }

    #[allow(dead_code)]
    pub fn reverse(mut self) -> Self {
        self.styles.push(Style::Reverse);
        self
    }

    #[allow(dead_code)]
    pub fn strikethrough(mut self) -> Self {
        self.styles.push(Style::Strikethrough);
        self
    }
}

impl fmt::Display for StyledText {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Apply foreground color
        if let Some(fg) = &self.fg_color {
            write!(f, "{}", fg.to_ansi_fg())?;
        }

        // Apply background color
        if let Some(bg) = &self.bg_color {
            write!(f, "{}", bg.to_ansi_bg())?;
        }

        // Apply styles
        for style in &self.styles {
            write!(f, "{}", style.to_ansi())?;
        }

        // Write the text
        write!(f, "{}", self.text)?;

        // Reset formatting
        write!(f, "{}", Style::Reset.to_ansi())?;

        Ok(())
    }
}

// Convenience functions for common formatting
#[allow(dead_code)]
pub fn colored_text(text: &str, color: Color) -> StyledText {
    StyledText::new(text).fg(color)
}

// Interactive prompts
use std::io::{self, Write};

#[allow(dead_code)]
pub fn prompt_yes_no(question: &str, default: bool) -> Result<bool, Box<dyn std::error::Error>> {
    let default_text = if default { "Y/n" } else { "y/N" };
    print!("{} {} {} ", 
        StyledText::new("❓").fg(Color::BrightYellow),
        StyledText::new(question).fg(Color::White),
        StyledText::new(&format!("({})", default_text)).fg(Color::BrightBlack));
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();
    
    match input.as_str() {
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        "" => Ok(default),
        _ => {
            println!("{}", warning_text("⚠️  Please enter 'y' or 'n'"));
            prompt_yes_no(question, default)
        }
    }
}

#[allow(dead_code)]
pub fn prompt_string(question: &str, default: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(def) = default {
        print!("{} {} {} ", 
            StyledText::new("❓").fg(Color::BrightYellow),
            StyledText::new(question).fg(Color::White),
            StyledText::new(&format!("[{}]", def)).fg(Color::BrightBlack));
    } else {
        print!("{} {} ", 
            StyledText::new("❓").fg(Color::BrightYellow),
            StyledText::new(question).fg(Color::White));
    }
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();
    
    if input.is_empty() {
        if let Some(def) = default {
            Ok(def.to_string())
        } else {
            prompt_string(question, default)
        }
    } else {
        Ok(input.to_string())
    }
}

#[allow(dead_code)]
pub fn prompt_choice<T: Clone>(question: &str, choices: &[(T, &str)]) -> Result<T, Box<dyn std::error::Error>> {
    println!("{} {}", 
        StyledText::new("❓").fg(Color::BrightYellow),
        StyledText::new(question).fg(Color::White));
    
    for (i, (_, desc)) in choices.iter().enumerate() {
        println!("  {} {}", 
            StyledText::new(&format!("{})", i + 1)).fg(Color::BrightCyan),
            StyledText::new(desc).fg(Color::White));
    }
    
    print!("{} ", 
        StyledText::new(&format!("Enter choice (1-{}):", choices.len())).fg(Color::BrightBlue));
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();
    
    match input.parse::<usize>() {
        Ok(choice) if choice > 0 && choice <= choices.len() => {
            Ok(choices[choice - 1].0.clone())
        }
        _ => {
            println!("{}", warning_text(&format!("⚠️  Please enter a number between 1 and {}", choices.len())));
            prompt_choice(question, choices)
        }
    }
}

#[allow(dead_code)]
pub fn bold_text(text: &str) -> StyledText {
    StyledText::new(text).bold()
}

pub fn error_text(text: &str) -> StyledText {
    StyledText::new(text).fg(Color::Red).bold()
}

pub fn success_text(text: &str) -> StyledText {
    StyledText::new(text).fg(Color::Green).bold()
}

pub fn warning_text(text: &str) -> StyledText {
    StyledText::new(text).fg(Color::Yellow).bold()
}

pub fn info_text(text: &str) -> StyledText {
    StyledText::new(text).fg(Color::Blue).bold()
}

pub fn dim_text(text: &str) -> StyledText {
    StyledText::new(text).dim()
}

#[allow(dead_code)]
pub fn header_text(text: &str) -> StyledText {
    StyledText::new(text).fg(Color::BrightCyan).bold().underline()
}

#[allow(dead_code)]
pub fn subheader_text(text: &str) -> StyledText {
    StyledText::new(text).fg(Color::BrightYellow).bold()
}

#[allow(dead_code)]
pub fn highlight_text(text: &str) -> StyledText {
    StyledText::new(text).fg(Color::BrightMagenta).bold()
}

// Progress indicator
pub struct ProgressBar {
    width: usize,
    completed: usize,
    total: usize,
    title: String,
}

impl ProgressBar {
    pub fn new(total: usize, width: usize) -> Self {
        ProgressBar {
            width,
            completed: 0,
            total,
            title: String::new(),
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn set_progress(&mut self, completed: usize) {
        self.completed = completed.min(self.total);
    }

    #[allow(dead_code)]
    pub fn increment(&mut self) {
        if self.completed < self.total {
            self.completed += 1;
        }
    }

    pub fn render(&self) -> String {
        let percentage = if self.total > 0 {
            (self.completed * 100) / self.total
        } else {
            0
        };

        let filled_width = if self.total > 0 {
            (self.completed * self.width) / self.total
        } else {
            0
        };

        let filled = "█".repeat(filled_width);
        let empty = "░".repeat(self.width - filled_width);

        if self.title.is_empty() {
            format!("[{}{}] {}% ({}/{})", filled, empty, percentage, self.completed, self.total)
        } else {
            format!("{}: [{}{}] {}% ({}/{})", 
                    self.title, filled, empty, percentage, self.completed, self.total)
        }
    }
}

// Spinner for indeterminate progress
pub struct Spinner {
    frames: Vec<&'static str>,
    current_frame: usize,
    title: String,
    active: bool,
}

impl Spinner {
    pub fn new() -> Self {
        Spinner {
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            current_frame: 0,
            title: String::new(),
            active: true,
        }
    }

    #[allow(dead_code)]
    pub fn dots() -> Self {
        Spinner {
            frames: vec!["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"],
            current_frame: 0,
            title: String::new(),
            active: true,
        }
    }

    pub fn pulse() -> Self {
        Spinner {
            frames: vec!["◐", "◓", "◑", "◒"],
            current_frame: 0,
            title: String::new(),
            active: true,
        }
    }

    pub fn arrow() -> Self {
        Spinner {
            frames: vec!["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"],
            current_frame: 0,
            title: String::new(),
            active: true,
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn next_frame(&mut self) -> String {
        if !self.active {
            return self.render();
        }
        
        let frame = self.frames[self.current_frame];
        self.current_frame = (self.current_frame + 1) % self.frames.len();
        
        if self.title.is_empty() {
            StyledText::new(frame).fg(Color::BrightCyan).to_string()
        } else {
            format!("{} {}", 
                StyledText::new(frame).fg(Color::BrightCyan),
                StyledText::new(&self.title).fg(Color::White))
        }
    }

    pub fn stop(&mut self) {
        self.active = false;
    }

    pub fn start(&mut self) {
        self.active = true;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn render(&self) -> String {
        let frame = self.frames[self.current_frame];
        if self.title.is_empty() {
            if self.active {
                StyledText::new(frame).fg(Color::BrightCyan).to_string()
            } else {
                StyledText::new("✓").fg(Color::Green).to_string()
            }
        } else {
            if self.active {
                format!("{} {}", 
                    StyledText::new(frame).fg(Color::BrightCyan),
                    StyledText::new(&self.title).fg(Color::White))
            } else {
                format!("{} {}", 
                    StyledText::new("✓").fg(Color::Green),
                    StyledText::new(&self.title).fg(Color::Green))
            }
        }
    }
}

// Status indicators for better UX
pub struct StatusIndicator {
    pub status: StatusType,
    pub message: String,
    pub timestamp: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum StatusType {
    Loading,
    Success,
    Error,
    Warning,
    Info,
    Processing,
    Waiting,
}

impl StatusIndicator {
    pub fn new(status: StatusType, message: &str) -> Self {
        StatusIndicator {
            status,
            message: message.to_string(),
            timestamp: false,
        }
    }

    #[allow(dead_code)]
    pub fn with_timestamp(mut self) -> Self {
        self.timestamp = true;
        self
    }

    pub fn render(&self) -> String {
        let (icon, color) = match self.status {
            StatusType::Loading => ("⏳", Color::BrightYellow),
            StatusType::Success => ("✅", Color::Green),
            StatusType::Error => ("❌", Color::Red),
            StatusType::Warning => ("⚠️", Color::Yellow),
            StatusType::Info => ("ℹ️", Color::Blue),
            StatusType::Processing => ("⚙️", Color::BrightCyan),
            StatusType::Waiting => ("⏸️", Color::BrightBlack),
        };

        let status_text = StyledText::new(&format!("{} {}", icon, self.message))
            .fg(color);

        if self.timestamp {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let timestamp_text = StyledText::new(&format!("[{}]", 
                format_timestamp(now)))
                .fg(Color::BrightBlack);
            format!("{} {}", timestamp_text, status_text)
        } else {
            status_text.to_string()
        }
    }
}

fn format_timestamp(timestamp: u64) -> String {
    let hours = (timestamp % 86400) / 3600;
    let minutes = (timestamp % 3600) / 60;
    let seconds = timestamp % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

// Enhanced progress bar with better styling
pub struct EnhancedProgressBar {
    progress: ProgressBar,
    status: Option<String>,
    eta: Option<u64>,
    rate: Option<f64>,
}

impl EnhancedProgressBar {
    pub fn new(total: usize, width: usize) -> Self {
        EnhancedProgressBar {
            progress: ProgressBar::new(total, width),
            status: None,
            eta: None,
            rate: None,
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.progress = self.progress.with_title(title);
        self
    }

    pub fn set_status(&mut self, status: &str) {
        self.status = Some(status.to_string());
    }

    pub fn set_eta(&mut self, seconds: u64) {
        self.eta = Some(seconds);
    }

    pub fn set_rate(&mut self, items_per_second: f64) {
        self.rate = Some(items_per_second);
    }

    pub fn set_progress(&mut self, completed: usize) {
        self.progress.set_progress(completed);
    }

    #[allow(dead_code)]
    pub fn increment(&mut self) {
        self.progress.increment();
    }

    pub fn render(&self) -> String {
        let mut result = self.progress.render();
        
        // Add status if available
        if let Some(ref status) = self.status {
            result.push_str(&format!(" - {}", 
                StyledText::new(status).fg(Color::BrightBlue)));
        }

        // Add rate if available
        if let Some(rate) = self.rate {
            result.push_str(&format!(" ({:.1}/s)", rate));
        }

        // Add ETA if available
        if let Some(eta) = self.eta {
            let eta_str = if eta > 60 {
                format!("{}m{}s", eta / 60, eta % 60)
            } else {
                format!("{}s", eta)
            };
            result.push_str(&format!(" ETA: {}", 
                StyledText::new(&eta_str).fg(Color::BrightYellow)));
        }

        result
    }
}

// Box drawing utilities for better UI layout
pub struct BoxDrawing;

impl BoxDrawing {
    pub fn single_border(width: usize, height: usize, title: Option<&str>) -> Vec<String> {
        let mut lines = Vec::new();
        
        // Top border
        let top = if let Some(title) = title {
            let title_width = title.len();
            let padding = if width > title_width + 4 { (width - title_width - 4) / 2 } else { 0 };
            format!("┌{}┤ {} ├{}┐",
                "─".repeat(padding),
                StyledText::new(title).fg(Color::BrightCyan).bold(),
                "─".repeat(width - padding - title_width - 4))
        } else {
            format!("┌{}┐", "─".repeat(width - 2))
        };
        lines.push(top);
        
        // Side borders
        for _ in 0..height - 2 {
            lines.push(format!("│{}│", " ".repeat(width - 2)));
        }
        
        // Bottom border
        lines.push(format!("└{}┘", "─".repeat(width - 2)));
        
        lines
    }

    pub fn double_border(width: usize, height: usize, title: Option<&str>) -> Vec<String> {
        let mut lines = Vec::new();
        
        // Top border
        let top = if let Some(title) = title {
            let title_width = title.len();
            let padding = if width > title_width + 4 { (width - title_width - 4) / 2 } else { 0 };
            format!("╔{}╡ {} ╞{}╗",
                "═".repeat(padding),
                StyledText::new(title).fg(Color::BrightCyan).bold(),
                "═".repeat(width - padding - title_width - 4))
        } else {
            format!("╔{}╗", "═".repeat(width - 2))
        };
        lines.push(top);
        
        // Side borders
        for _ in 0..height - 2 {
            lines.push(format!("║{}║", " ".repeat(width - 2)));
        }
        
        // Bottom border
        lines.push(format!("╚{}╝", "═".repeat(width - 2)));
        
        lines
    }

    pub fn rounded_border(width: usize, height: usize, title: Option<&str>) -> Vec<String> {
        let mut lines = Vec::new();
        
        // Top border
        let top = if let Some(title) = title {
            let title_width = title.len();
            let padding = if width > title_width + 4 { (width - title_width - 4) / 2 } else { 0 };
            format!("╭{}┤ {} ├{}╮",
                "─".repeat(padding),
                StyledText::new(title).fg(Color::BrightCyan).bold(),
                "─".repeat(width - padding - title_width - 4))
        } else {
            format!("╭{}╮", "─".repeat(width - 2))
        };
        lines.push(top);
        
        // Side borders
        for _ in 0..height - 2 {
            lines.push(format!("│{}│", " ".repeat(width - 2)));
        }
        
        // Bottom border
        lines.push(format!("╰{}╯", "─".repeat(width - 2)));
        
        lines
    }
}

// Terminal clearing and cursor control
pub struct TerminalControl;

impl TerminalControl {
    pub fn clear_screen() -> &'static str {
        "\x1B[2J\x1B[H"
    }

    pub fn clear_line() -> &'static str {
        "\x1B[2K\r"
    }

    #[allow(dead_code)]
    pub fn move_cursor_up(lines: u16) -> String {
        format!("\x1B[{}A", lines)
    }

    #[allow(dead_code)]
    pub fn move_cursor_down(lines: u16) -> String {
        format!("\x1B[{}B", lines)
    }

    pub fn hide_cursor() -> &'static str {
        "\x1B[?25l"
    }

    pub fn show_cursor() -> &'static str {
        "\x1B[?25h"
    }

    #[allow(dead_code)]
    pub fn save_cursor() -> &'static str {
        "\x1B[s"
    }

    #[allow(dead_code)]
    pub fn restore_cursor() -> &'static str {
        "\x1B[u"
    }
}

// Enhanced table rendering
pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    column_widths: Vec<usize>,
    border_style: BorderStyle,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum BorderStyle {
    None,
    Single,
    Double,
    Rounded,
}

impl Table {
    pub fn new(headers: Vec<&str>) -> Self {
        let column_widths = headers.iter().map(|h| h.len()).collect();
        Table {
            headers: headers.into_iter().map(|s| s.to_string()).collect(),
            rows: Vec::new(),
            column_widths,
            border_style: BorderStyle::Single,
        }
    }

    pub fn border_style(mut self, style: BorderStyle) -> Self {
        self.border_style = style;
        self
    }

    pub fn add_row(&mut self, row: Vec<&str>) {
        for (i, cell) in row.iter().enumerate() {
            if i < self.column_widths.len() {
                self.column_widths[i] = self.column_widths[i].max(cell.len());
            }
        }
        self.rows.push(row.into_iter().map(|s| s.to_string()).collect());
    }

    pub fn render(&self) -> String {
        let mut result = String::new();
        let total_width: usize = self.column_widths.iter().sum::<usize>() + 
            (self.column_widths.len() - 1) * 3 + 4; // padding + borders

        match self.border_style {
            BorderStyle::None => {
                // Headers
                for (i, header) in self.headers.iter().enumerate() {
                    result.push_str(&format!("{:<width$}", 
                        StyledText::new(header).fg(Color::BrightCyan).bold(),
                        width = self.column_widths[i]));
                    if i < self.headers.len() - 1 {
                        result.push_str(" │ ");
                    }
                }
                result.push('\n');
                
                // Separator
                for (i, &width) in self.column_widths.iter().enumerate() {
                    result.push_str(&"─".repeat(width));
                    if i < self.column_widths.len() - 1 {
                        result.push_str("─┼─");
                    }
                }
                result.push('\n');
                
                // Rows
                for row in &self.rows {
                    for (i, cell) in row.iter().enumerate() {
                        result.push_str(&format!("{:<width$}", cell, width = self.column_widths[i]));
                        if i < row.len() - 1 {
                            result.push_str(" │ ");
                        }
                    }
                    result.push('\n');
                }
            }
            _ => {
                // Full bordered table (simplified for now)
                let top_border = format!("┌{}┐", "─".repeat(total_width - 2));
                result.push_str(&top_border);
                result.push('\n');
                
                // Headers with borders
                result.push('│');
                for (i, header) in self.headers.iter().enumerate() {
                    result.push_str(&format!(" {:<width$} ", 
                        StyledText::new(header).fg(Color::BrightCyan).bold(),
                        width = self.column_widths[i]));
                    if i < self.headers.len() - 1 {
                        result.push('│');
                    }
                }
                result.push('│');
                result.push('\n');
                
                // Header separator
                result.push('├');
                for (i, &width) in self.column_widths.iter().enumerate() {
                    result.push_str(&"─".repeat(width + 2));
                    if i < self.column_widths.len() - 1 {
                        result.push('┼');
                    }
                }
                result.push('┤');
                result.push('\n');
                
                // Rows with borders
                for row in &self.rows {
                    result.push('│');
                    for (i, cell) in row.iter().enumerate() {
                        result.push_str(&format!(" {:<width$} ", cell, width = self.column_widths[i]));
                        if i < row.len() - 1 {
                            result.push('│');
                        }
                    }
                    result.push('│');
                    result.push('\n');
                }
                
                // Bottom border
                let bottom_border = format!("└{}┘", "─".repeat(total_width - 2));
                result.push_str(&bottom_border);
            }
        }

        result
    }
}

// Multi-stage progress system
#[derive(Debug, Clone)]
pub struct MultiStageProgress {
    stages: Vec<ProgressStage>,
    current_stage: usize,
    total_progress: f64,
    pub start_time: Instant,
    title: String,
}

#[derive(Debug, Clone)]
pub struct ProgressStage {
    name: String,
    weight: f64,
    progress: f64,
    status: StageStatus,
    sub_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StageStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
}

impl MultiStageProgress {
    pub fn new(title: &str) -> Self {
        MultiStageProgress {
            stages: Vec::new(),
            current_stage: 0,
            total_progress: 0.0,
            start_time: Instant::now(),
            title: title.to_string(),
        }
    }

    pub fn add_stage(&mut self, name: &str, weight: f64) {
        self.stages.push(ProgressStage {
            name: name.to_string(),
            weight,
            progress: 0.0,
            status: StageStatus::Pending,
            sub_message: None,
        });
    }

    pub fn start_stage(&mut self, stage_index: usize) -> Result<(), String> {
        if stage_index >= self.stages.len() {
            return Err("Stage index out of bounds".to_string());
        }
        
        if self.current_stage < stage_index {
            // Mark previous stages as completed
            for i in self.current_stage..stage_index {
                if self.stages[i].status == StageStatus::Pending {
                    self.stages[i].status = StageStatus::Completed;
                    self.stages[i].progress = 1.0;
                }
            }
        }
        
        self.current_stage = stage_index;
        self.stages[stage_index].status = StageStatus::InProgress;
        self.update_total_progress();
        Ok(())
    }

    pub fn update_stage_progress(&mut self, stage_index: usize, progress: f64, sub_message: Option<&str>) -> Result<(), String> {
        if stage_index >= self.stages.len() {
            return Err("Stage index out of bounds".to_string());
        }
        
        self.stages[stage_index].progress = progress.min(1.0).max(0.0);
        if let Some(msg) = sub_message {
            self.stages[stage_index].sub_message = Some(msg.to_string());
        }
        self.update_total_progress();
        Ok(())
    }

    pub fn complete_stage(&mut self, stage_index: usize) -> Result<(), String> {
        if stage_index >= self.stages.len() {
            return Err("Stage index out of bounds".to_string());
        }
        
        self.stages[stage_index].status = StageStatus::Completed;
        self.stages[stage_index].progress = 1.0;
        self.stages[stage_index].sub_message = None;
        self.update_total_progress();
        Ok(())
    }

    pub fn fail_stage(&mut self, stage_index: usize, error: &str) -> Result<(), String> {
        if stage_index >= self.stages.len() {
            return Err("Stage index out of bounds".to_string());
        }
        
        self.stages[stage_index].status = StageStatus::Failed(error.to_string());
        self.update_total_progress();
        Ok(())
    }

    fn update_total_progress(&mut self) {
        let total_weight: f64 = self.stages.iter().map(|s| s.weight).sum();
        if total_weight == 0.0 {
            self.total_progress = 0.0;
            return;
        }

        self.total_progress = self.stages.iter()
            .map(|stage| {
                let stage_progress = match stage.status {
                    StageStatus::Completed => 1.0,
                    StageStatus::Failed(_) => 0.0,
                    _ => stage.progress,
                };
                (stage.weight / total_weight) * stage_progress
            })
            .sum();
    }

    pub fn render(&self, width: usize) -> String {
        let elapsed = self.start_time.elapsed();
        let mut output = String::new();

        // Title and overall progress
        output.push_str(&format!("{}\n", 
            StyledText::new(&self.title).fg(Color::BrightCyan).bold()));
        
        // Overall progress bar
        let filled_width = ((self.total_progress * width as f64) as usize).min(width);
        let filled = "█".repeat(filled_width);
        let empty = "░".repeat(width - filled_width);
        
        output.push_str(&format!("Overall: [{}{}] {:.1}% ({:.1}s)\n",
            StyledText::new(&filled).fg(Color::BrightGreen),
            StyledText::new(&empty).fg(Color::BrightBlack),
            self.total_progress * 100.0,
            elapsed.as_secs_f64()));

        // Individual stages
        for (_i, stage) in self.stages.iter().enumerate() {
            let status_icon = match &stage.status {
                StageStatus::Pending => StyledText::new("⏸").fg(Color::BrightBlack),
                StageStatus::InProgress => StyledText::new("⚙️").fg(Color::BrightYellow),
                StageStatus::Completed => StyledText::new("✅").fg(Color::Green),
                StageStatus::Failed(_) => StyledText::new("❌").fg(Color::Red),
            };

            let stage_filled = ((stage.progress * 20.0) as usize).min(20);
            let stage_bar_filled = "▰".repeat(stage_filled);
            let stage_bar_empty = "▱".repeat(20 - stage_filled);

            output.push_str(&format!("  {} {} [{}{}] {:.0}%",
                status_icon,
                StyledText::new(&stage.name).fg(Color::White),
                StyledText::new(&stage_bar_filled).fg(Color::BrightGreen),
                StyledText::new(&stage_bar_empty).fg(Color::BrightBlack),
                stage.progress * 100.0));

            if let Some(ref sub_msg) = stage.sub_message {
                output.push_str(&format!(" - {}", 
                    StyledText::new(sub_msg).fg(Color::BrightBlue)));
            }

            if let StageStatus::Failed(ref error) = stage.status {
                output.push_str(&format!(" ({})", 
                    StyledText::new(error).fg(Color::Red)));
            }

            output.push('\n');
        }

        output
    }

    #[allow(dead_code)]
    pub fn is_complete(&self) -> bool {
        self.stages.iter().all(|stage| matches!(stage.status, StageStatus::Completed))
    }

    #[allow(dead_code)]
    pub fn has_failed(&self) -> bool {
        self.stages.iter().any(|stage| matches!(stage.status, StageStatus::Failed(_)))
    }

    pub fn get_eta_seconds(&self) -> Option<u64> {
        if self.total_progress <= 0.0 {
            return None;
        }

        let elapsed = self.start_time.elapsed().as_secs_f64();
        let remaining_progress = 1.0 - self.total_progress;
        let estimated_total_time = elapsed / self.total_progress;
        let eta = estimated_total_time * remaining_progress;

        Some(eta as u64)
    }
}

// Background task monitoring system
pub struct TaskMonitor {
    tasks: Arc<Mutex<HashMap<String, BackgroundTask>>>,
    notifications: Arc<Mutex<Vec<Notification>>>,
}

#[derive(Debug, Clone)]
pub struct BackgroundTask {
    pub id: String,
    pub title: String,
    pub progress: MultiStageProgress,
    pub started_at: Instant,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Running,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub timestamp: Instant,
    pub read: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum NotificationType {
    Success,
    Error,
    Warning,
    Info,
}

impl TaskMonitor {
    pub fn new() -> Self {
        TaskMonitor {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            notifications: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn start_task(&self, id: &str, title: &str) -> String {
        let task_id = format!("{}-{}", id, Instant::now().elapsed().as_millis());
        let task = BackgroundTask {
            id: task_id.clone(),
            title: title.to_string(),
            progress: MultiStageProgress::new(title),
            started_at: Instant::now(),
            status: TaskStatus::Running,
        };

        if let Ok(mut tasks) = self.tasks.lock() {
            tasks.insert(task_id.clone(), task);
        }

        task_id
    }

    pub fn update_task<F>(&self, task_id: &str, updater: F) -> Result<(), String>
    where
        F: FnOnce(&mut BackgroundTask),
    {
        if let Ok(mut tasks) = self.tasks.lock() {
            if let Some(task) = tasks.get_mut(task_id) {
                updater(task);
                Ok(())
            } else {
                Err("Task not found".to_string())
            }
        } else {
            Err("Failed to acquire lock".to_string())
        }
    }

    pub fn complete_task(&self, task_id: &str, message: &str) {
        if let Ok(mut tasks) = self.tasks.lock() {
            if let Some(task) = tasks.get_mut(task_id) {
                task.status = TaskStatus::Completed;
                
                self.add_notification(&format!("task-complete-{}", task_id), 
                    &format!("✅ {} - {}", task.title, message), 
                    NotificationType::Success);
            }
        }
    }

    pub fn fail_task(&self, task_id: &str, error: &str) {
        if let Ok(mut tasks) = self.tasks.lock() {
            if let Some(task) = tasks.get_mut(task_id) {
                task.status = TaskStatus::Failed(error.to_string());
                
                self.add_notification(&format!("task-failed-{}", task_id),
                    &format!("❌ {} - {}", task.title, error),
                    NotificationType::Error);
            }
        }
    }

    pub fn add_notification(&self, id: &str, message: &str, notification_type: NotificationType) {
        if let Ok(mut notifications) = self.notifications.lock() {
            notifications.push(Notification {
                id: id.to_string(),
                message: message.to_string(),
                notification_type,
                timestamp: Instant::now(),
                read: false,
            });
        }
    }

    pub fn get_unread_notifications(&self) -> Vec<Notification> {
        if let Ok(notifications) = self.notifications.lock() {
            notifications.iter()
                .filter(|n| !n.read)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn mark_notification_read(&self, id: &str) {
        if let Ok(mut notifications) = self.notifications.lock() {
            if let Some(notification) = notifications.iter_mut().find(|n| n.id == id) {
                notification.read = true;
            }
        }
    }

    pub fn render_status_dashboard(&self, _max_width: usize) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("{}\n", 
            StyledText::new("📊 Background Tasks & Notifications").fg(Color::BrightCyan).bold()));

        // Render active tasks
        if let Ok(tasks) = self.tasks.lock() {
            let active_tasks: Vec<_> = tasks.values()
                .filter(|task| task.status == TaskStatus::Running)
                .collect();
            
            if !active_tasks.is_empty() {
                output.push_str(&format!("\n{}\n", 
                    StyledText::new("🔄 Active Tasks:").fg(Color::BrightYellow).bold()));
                
                for task in active_tasks {
                    let elapsed = task.started_at.elapsed().as_secs();
                    output.push_str(&format!("  {} ({}s)\n", 
                        StyledText::new(&task.title).fg(Color::White),
                        elapsed));
                    
                    // Show mini progress for each task
                    let mini_progress = task.progress.render(30);
                    for line in mini_progress.lines().skip(1) {
                        output.push_str(&format!("    {}\n", line));
                    }
                }
            }
        }

        // Render recent notifications
        let notifications = self.get_unread_notifications();
        if !notifications.is_empty() {
            output.push_str(&format!("\n{}\n", 
                StyledText::new("🔔 Recent Notifications:").fg(Color::BrightMagenta).bold()));
            
            for notification in notifications.iter().take(5) {
                let age = notification.timestamp.elapsed().as_secs();
                let age_str = if age < 60 {
                    format!("{}s ago", age)
                } else if age < 3600 {
                    format!("{}m ago", age / 60)
                } else {
                    format!("{}h ago", age / 3600)
                };

                let color = match notification.notification_type {
                    NotificationType::Success => Color::Green,
                    NotificationType::Error => Color::Red,
                    NotificationType::Warning => Color::Yellow,
                    NotificationType::Info => Color::Blue,
                };

                output.push_str(&format!("  {} ({})\n",
                    StyledText::new(&notification.message).fg(color),
                    StyledText::new(&age_str).fg(Color::BrightBlack)));
            }
        }

        if output.len() <= 100 { // Just the header
            output.push_str(&format!("{}\n", 
                StyledText::new("  No active tasks or notifications").fg(Color::BrightBlack)));
        }

        output
    }
}

// Global task monitor instance
lazy_static::lazy_static! {
    pub static ref GLOBAL_TASK_MONITOR: TaskMonitor = TaskMonitor::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_ansi_codes() {
        assert_eq!(Color::Red.to_ansi_fg(), "\x1B[31m");
        assert_eq!(Color::Blue.to_ansi_bg(), "\x1B[44m");
        assert_eq!(Color::Rgb(255, 128, 64).to_ansi_fg(), "\x1B[38;2;255;128;64m");
    }

    #[test]
    fn test_styled_text() {
        let styled = StyledText::new("Hello").fg(Color::Red).bold();
        let output = format!("{}", styled);
        assert!(output.contains("\x1B[31m")); // Red foreground
        assert!(output.contains("\x1B[1m"));  // Bold
        assert!(output.contains("Hello"));
        assert!(output.ends_with("\x1B[0m")); // Reset
    }

    #[test]
    fn test_progress_bar() {
        let mut progress = ProgressBar::new(100, 20).with_title("Test");
        progress.set_progress(50);
        let output = progress.render();
        assert!(output.contains("Test:"));
        assert!(output.contains("50%"));
        assert!(output.contains("(50/100)"));
    }

    #[test]
    fn test_spinner() {
        let mut spinner = Spinner::new().with_title("Loading");
        let frame1 = spinner.next_frame();
        let frame2 = spinner.next_frame();
        assert!(frame1.contains("Loading"));
        assert!(frame2.contains("Loading"));
        assert_ne!(frame1, frame2);
    }

    #[test]
    fn test_multi_stage_progress() {
        let mut progress = MultiStageProgress::new("Test Process");
        progress.add_stage("Stage 1", 0.3);
        progress.add_stage("Stage 2", 0.7);
        
        progress.start_stage(0).unwrap();
        progress.update_stage_progress(0, 0.5, Some("Half done")).unwrap();
        
        assert_eq!(progress.total_progress, 0.15); // 30% of 50%
        assert!(!progress.is_complete());
        
        progress.complete_stage(0).unwrap();
        progress.start_stage(1).unwrap();
        progress.complete_stage(1).unwrap();
        
        assert!(progress.is_complete());
    }

    #[test]
    fn test_task_monitor() {
        let monitor = TaskMonitor::new();
        let task_id = monitor.start_task("test", "Test Task");
        
        monitor.complete_task(&task_id, "All done!");
        let notifications = monitor.get_unread_notifications();
        
        assert!(!notifications.is_empty());
        assert!(notifications[0].message.contains("Test Task"));
    }
}