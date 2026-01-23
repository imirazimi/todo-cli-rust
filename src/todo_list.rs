use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display};

use crate::SearchParams;

fn is_subsequence(sub: &str, text: &str) -> bool {
    let mut sub_chars = sub.chars().flat_map(char::to_lowercase).peekable();
    for ch in text.chars().flat_map(char::to_lowercase) {
        if sub_chars.peek() == Some(&ch) { sub_chars.next(); }
    }
    sub_chars.peek().is_none()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Index(pub u64);

impl Index {
    #[must_use] pub fn new(i: u64) -> Self { Self(i) }
}

impl Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Description(pub String);

impl Description {
    #[must_use] pub fn new(s: &str) -> Self { Self(s.to_owned()) }
}

impl Display for Description {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag(pub String);

impl Tag {
    #[must_use] pub fn new(s: &str) -> Self { Self(s.to_owned()) }
}

impl Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoItem {
    pub index: Index,
    pub description: Description,
    pub tags: Vec<Tag>,
    pub done: bool,
}

impl TodoItem {
    #[must_use] pub fn new(index: Index, description: Description, tags: Vec<Tag>) -> Self {
        Self { index, description, tags, done: false }
    }
}

impl Display for TodoItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} \"{}\"", self.index, self.description)?;
        for tag in &self.tags { write!(f, " #{}", tag.0)?; }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TodoList {
    top_index: Index,
    items: Vec<TodoItem>,
    tag_index: HashMap<String, Vec<u64>>,
    word_index: HashMap<String, Vec<u64>>,
}

impl TodoList {
    #[must_use] pub fn new() -> Self { Self::default() }

    pub fn push(&mut self, description: Description, tags: Vec<Tag>) -> TodoItem {
        let idx = self.top_index.0;
        
        for word in description.0.split_whitespace() {
            self.word_index.entry(word.to_lowercase()).or_default().push(idx);
        }
        for tag in &tags {
            self.tag_index.entry(tag.0.to_lowercase()).or_default().push(idx);
        }
        
        let item = TodoItem::new(self.top_index, description, tags);
        self.items.push(item.clone());
        self.top_index = Index(idx + 1);
        item
    }

    pub fn done_with_index(&mut self, idx: Index) -> Option<Index> {
        self.items.iter_mut().find(|i| i.index == idx).map(|i| { i.done = true; idx })
    }

    #[must_use] pub fn search(&self, sp: &SearchParams) -> Vec<&TodoItem> {
        // Tag-only search uses index for faster lookup
        if sp.words.is_empty() {
            return if sp.tags.is_empty() {
                self.items.iter().filter(|item| !item.done).collect()
            } else {
                self.search_by_tags_only(&sp.tags)
            };
        }
        
        // For word search, first try to find candidates using the word index
        let mut candidate_indices: Option<HashSet<u64>> = None;
        
        for search_word in &sp.words {
            let search_lower = search_word.0.to_lowercase();
            
            // Find all words in the index that the search_word is a subsequence of
            let mut matching_indices = HashSet::new();
            for (indexed_word, indices) in &self.word_index {
                if is_subsequence(&search_lower, indexed_word) {
                    for &idx in indices {
                        matching_indices.insert(idx);
                    }
                }
            }
            
            match candidate_indices {
                None => candidate_indices = Some(matching_indices),
                Some(ref mut existing) => {
                    existing.retain(|idx| matching_indices.contains(idx));
                }
            }
        }
        
        // Now filter by tags if needed
        let Some(indices) = candidate_indices else { return Vec::new() };
        
        indices.iter()
            .filter_map(|&idx| {
                self.items.iter().find(|item| item.index.0 == idx && !item.done)
            })
            .filter(|item| {
                // Check if all tags match
                sp.tags.iter().all(|search_tag| {
                    item.tags.iter().any(|item_tag| {
                        is_subsequence(&search_tag.0, &item_tag.0)
                    })
                })
            })
            .collect()
    }
    
    fn search_by_tags_only(&self, search_tags: &[Tag]) -> Vec<&TodoItem> {
        // For exact tag matching, use the index
        // First find all items that have all the required tags
        let mut candidate_indices: Option<Vec<u64>> = None;
        
        for search_tag in search_tags {
            let tag_lower = search_tag.0.to_lowercase();
            
            // Try exact match first
            if let Some(indices) = self.tag_index.get(&tag_lower) {
                match candidate_indices {
                    None => candidate_indices = Some(indices.clone()),
                    Some(ref mut existing) => {
                        existing.retain(|idx| indices.contains(idx));
                    }
                }
            } else {
                // No exact match, fall back to subsequence matching on all items
                return self.search_tags_subsequence(search_tags);
            }
        }
        
        match candidate_indices {
            Some(indices) => {
                indices.iter()
                    .filter_map(|&idx| {
                        self.items.iter()
                            .find(|item| item.index.0 == idx && !item.done)
                    })
                    .collect()
            }
            None => Vec::new(),
        }
    }
    
    fn search_tags_subsequence(&self, search_tags: &[Tag]) -> Vec<&TodoItem> {
        self.items.iter()
            .filter(|item| !item.done)
            .filter(|item| {
                search_tags.iter().all(|search_tag| {
                    item.tags.iter().any(|item_tag| {
                        is_subsequence(&search_tag.0, &item_tag.0)
                    })
                })
            })
            .collect()
    }
}
