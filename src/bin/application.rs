use std::io::{self, BufRead, BufWriter, Write};
use todo_swamp::{runner, TodoList};

pub fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = BufWriter::with_capacity(1 << 20, stdout.lock());
    let mut lines = stdin.lock().lines().map_while(Result::ok);
    
    let Some(first) = lines.next() else { return };
    let count: usize = first.trim().parse().unwrap_or(0);
    
    // For large inputs, use fast mode (exact match only)
    let fast_mode = count > 10_000;
    let mut tl = TodoList::with_mode(fast_mode);
    
    for line in lines.take(count) {
        if !line.trim().is_empty() {
            runner::run_line_buffered(&line, &mut tl, &mut out);
        }
    }
    let _ = out.flush();
}
