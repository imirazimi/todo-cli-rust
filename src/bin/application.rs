use std::io::{self, BufRead};
use todo_swamp::{runner, TodoList};

pub fn main() {
    let mut tl = TodoList::new();
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines().map_while(Result::ok);
    
    let Some(first) = lines.next() else { return };
    let count: usize = first.trim().parse().unwrap_or(0);
    
    for line in lines.take(count) {
        if !line.trim().is_empty() {
            runner::run_line(&line, &mut tl);
        }
    }
}
