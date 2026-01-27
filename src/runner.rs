use std::io::Write;
use crate::{parser, Query, QueryError, TodoList, TodoItem, Index};

pub fn run_line_buffered<W: Write>(line: &str, tl: &mut TodoList, out: &mut W, line_ending: &[u8]) {
    let trimmed = line.trim();
    if trimmed.is_empty() { return; }
    
    if let Ok((_, q)) = parser::query(trimmed) {
        match run_query_ref(q, tl) {
            Ok(r) => { let _ = write_result(out, &r, line_ending); }
            Err(e) => { 
                let _ = write!(out, "Error: {}", e.0);
                let _ = out.write_all(line_ending);
            }
        }
    }
}

enum QueryResultRef<'a> {
    Added(Index),
    Done,
    Found(Vec<&'a TodoItem>, bool), // bool indicates concise mode
}

fn run_query_ref(q: Query, tl: &mut TodoList) -> Result<QueryResultRef<'_>, QueryError> {
    let concise = tl.is_concise();
    match q {
        Query::Add(desc, tags) => Ok(QueryResultRef::Added(tl.push(desc, tags))),
        Query::Done(idx) => tl.done_with_index(idx)
            .map(|_| QueryResultRef::Done)
            .ok_or_else(|| QueryError(format!("Index {idx} not found"))),
        Query::Search(params) => Ok(QueryResultRef::Found(tl.search(&params), concise)),
    }
}

fn write_result<W: Write>(out: &mut W, r: &QueryResultRef, line_ending: &[u8]) -> std::io::Result<()> {
    match r {
        QueryResultRef::Added(idx) => {
            let mut buffer = itoa::Buffer::new();
            out.write_all(buffer.format(idx.0).as_bytes())?;
            out.write_all(line_ending)
        }
        QueryResultRef::Done => {
            out.write_all(b"done")?;
            out.write_all(line_ending)
        }
        QueryResultRef::Found(items, concise) => {
            let mut buffer = itoa::Buffer::new();
            out.write_all(buffer.format(items.len()).as_bytes())?;
            out.write_all(b" item(s) found")?;
            out.write_all(line_ending)?;
            for item in items.iter() {
                out.write_all(buffer.format(item.index.0).as_bytes())?;
                if !concise {
                    out.write_all(b" \"")?;
                    out.write_all(item.description.0.as_bytes())?;
                    out.write_all(b"\"")?;
                    for tag in &item.tags {
                        out.write_all(b" #")?;
                        out.write_all(tag.0.as_bytes())?;
                    }
                }
                out.write_all(line_ending)?;
            }
            Ok(())
        }
    }
}
