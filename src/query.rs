use std::fmt::{self, Display};
use crate::{Description, Index, Tag, TodoItem};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Query {
    Add(Description, Vec<Tag>),
    Done(Index),
    Search(SearchParams),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SearchParams {
    pub words: Vec<SearchWord>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchWord(pub String);

impl SearchWord {
    #[must_use] pub fn new(s: &str) -> Self { Self(s.to_owned()) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryResult {
    Added(TodoItem),
    Done,
    Found(Vec<TodoItem>),
}

impl Display for QueryResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Added(ti) => write!(f, "{}", ti.index),
            Self::Done => write!(f, "done"),
            Self::Found(rs) => {
                writeln!(f, "{} item(s) found", rs.len())?;
                for (i, item) in rs.iter().enumerate() {
                    if i == rs.len() - 1 { write!(f, "{item}")?; }
                    else { writeln!(f, "{item}")?; }
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub struct QueryError(pub String);

impl Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.0)
    }
}
