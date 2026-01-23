use crate::{parser, Query, QueryError, QueryResult, TodoList};

pub fn run_line(line: &str, tl: &mut TodoList) {
    let trimmed = line.trim();
    if trimmed.is_empty() { return; }
    
    if let Ok((_, q)) = parser::query(trimmed) {
        match run_query(q, tl) {
            Ok(r) => println!("{r}"),
            Err(e) => eprintln!("{e}"),
        }
    }
}

fn run_query(q: Query, tl: &mut TodoList) -> Result<QueryResult, QueryError> {
    match q {
        Query::Add(desc, tags) => Ok(QueryResult::Added(tl.push(desc, tags))),
        Query::Done(idx) => tl.done_with_index(idx)
            .map(|_| QueryResult::Done)
            .ok_or_else(|| QueryError(format!("Index {idx} not found"))),
        Query::Search(params) => Ok(QueryResult::Found(tl.search(&params).into_iter().cloned().collect())),
    }
}
