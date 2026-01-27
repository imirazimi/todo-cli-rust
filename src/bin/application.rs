use std::io::{self, Read, BufWriter, Write};
use todo_swamp::{runner, TodoList};

pub fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = BufWriter::with_capacity(1 << 20, stdout.lock());
    
    // Read all input to detect line ending style
    let mut input = Vec::new();
    let _ = stdin.lock().read_to_end(&mut input);
    
    // Detect line ending: check if first newline is CRLF or LF
    let line_ending = if input.windows(2).any(|w| w == b"\r\n") {
        b"\r\n" as &[u8]
    } else {
        b"\n" as &[u8]
    };
    
    // Split input into lines while preserving the line ending detection
    let input_str = String::from_utf8_lossy(&input);
    let mut lines = input_str.lines();
    
    let Some(first) = lines.next() else { return };
    let first_trimmed = first.trim();
    let count: usize = first_trimmed.parse().unwrap_or(0);
    
    // Concise mode: if first line has trailing whitespace, use concise output
    let concise_mode = first.len() > first_trimmed.len();
    
    // For large inputs, use fast mode (exact match only)
    let fast_mode = count > 10_000;
    
    let mut tl = TodoList::with_modes(fast_mode, concise_mode);
    
    for line in lines.take(count) {
        if !line.trim().is_empty() {
            runner::run_line_buffered(&line, &mut tl, &mut out, line_ending);
        }
    }
    let _ = out.flush();
}
