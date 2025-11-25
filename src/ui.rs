use unicode_width::UnicodeWidthStr;

const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

pub struct ProgramInfo {
    pub station_name: String,
    pub area_name: String,
    pub program_title: String,
    pub description: String,
}

fn truncate_str(s: &str, max_width: usize) -> String {
    let mut width = 0;
    let mut result = String::new();

    for c in s.chars() {
        let char_width = UnicodeWidthStr::width(c.to_string().as_str());
        if width + char_width > max_width - 3 {
            result.push_str("...");
            break;
        }
        width += char_width;
        result.push(c);
    }

    if width + 3 <= max_width && result.ends_with("...") {
        result
    } else if UnicodeWidthStr::width(s) <= max_width {
        s.to_string()
    } else {
        result
    }
}

fn pad_to_width(s: &str, width: usize) -> String {
    let current_width = UnicodeWidthStr::width(s);
    if current_width >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - current_width))
    }
}

pub fn print_now_playing(info: &ProgramInfo) {
    let inner_width = 46;

    let station_line = format!("📻 NHK {} - {}", info.station_name, info.area_name);
    let title_line = format!("♪ {}", truncate_str(&info.program_title, inner_width - 4));
    let desc_line = truncate_str(&info.description, inner_width - 2);

    let border = "─".repeat(inner_width);

    println!();
    println!("  {}┌{}┐{}", CYAN, border, RESET);
    println!(
        "  {}│ {}{}{}{}│{}",
        CYAN,
        BOLD,
        pad_to_width(&station_line, inner_width - 1),
        RESET,
        CYAN,
        RESET
    );
    println!("  {}├{}┤{}", CYAN, border, RESET);
    println!(
        "  {}│ {}{}{}{}│{}",
        CYAN,
        YELLOW,
        pad_to_width(&title_line, inner_width - 1),
        RESET,
        CYAN,
        RESET
    );
    println!(
        "  {}│{}│{}",
        CYAN,
        " ".repeat(inner_width),
        RESET
    );
    println!(
        "  {}│ {}{}{}{}│{}",
        CYAN,
        DIM,
        pad_to_width(&desc_line, inner_width - 1),
        RESET,
        CYAN,
        RESET
    );
    println!("  {}└{}┘{}", CYAN, border, RESET);
    println!();
    println!(
        "    {}▶ Playing...{}  {}[Ctrl+C to stop]{}",
        GREEN, RESET, DIM, RESET
    );
    println!();
}
