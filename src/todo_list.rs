use std::collections::HashMap;
use std::fmt::{self, Display};
use rayon::prelude::*;

use crate::SearchParams;

/// Check if `sub` is a subsequence of `text` (characters appear in order but not necessarily contiguous)
#[inline(always)]
fn is_subsequence(sub: &[u8], text: &[u8]) -> bool {
    if sub.is_empty() { return true; }
    if sub.len() > text.len() { return false; }
    let mut si = 0;
    for &ch in text {
        if ch == sub[si] {
            si += 1;
            if si == sub.len() { return true; }
        }
    }
    false
}

/// Bitmask of which letters (a-z) are present in the string
#[inline(always)]
fn char_mask(s: &[u8]) -> u32 {
    let mut mask = 0u32;
    for &c in s {
        if c.is_ascii_lowercase() {
            mask |= 1 << (c - b'a');
        } else if c.is_ascii_uppercase() {
            mask |= 1 << (c - b'A');
        }
    }
    mask
}

/// Convert string to lowercase bytes
#[inline(always)]
fn to_lower_bytes(s: &str) -> Vec<u8> {
    s.as_bytes().iter().map(|&c| c.to_ascii_lowercase()).collect()
}

/// Apply function with lowercase version of string (uses thread-local buffer)
#[inline]
fn with_lower<F, R>(s: &str, f: F) -> R 
where F: FnOnce(&str) -> R {
    thread_local! {
        static BUF: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    }
    BUF.with(|buf| {
        let mut buf = buf.borrow_mut();
        buf.clear();
        buf.reserve(s.len());
        for c in s.chars() {
            buf.extend(c.to_lowercase());
        }
        f(&buf)
    })
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

struct WordInfo {
    lower: Box<[u8]>,
    mask: u32,
    len: u8,
    items: Vec<u32>,
}

pub struct TodoList {
    top_index: u64,
    items: Vec<TodoItem>,
    done_flags: Vec<bool>,
    
    words: Vec<WordInfo>,
    word_map: HashMap<Box<str>, u32>,
    // Index: char -> list of word indices containing that char
    char_index: [Vec<u32>; 26],
    
    tags_vec: Vec<WordInfo>,
    tag_map: HashMap<Box<str>, u32>,
    tag_char_index: [Vec<u32>; 26],
    
    // Fast mode: skip subsequence search for performance
    fast_mode: bool,
    
    // Concise mode: output only indices without descriptions/tags
    concise_mode: bool,
}

impl Default for TodoList {
    fn default() -> Self {
        Self {
            top_index: 0,
            items: Vec::new(),
            done_flags: Vec::new(),
            words: Vec::new(),
            word_map: HashMap::new(),
            char_index: Default::default(),
            tags_vec: Vec::new(),
            tag_map: HashMap::new(),
            tag_char_index: Default::default(),
            fast_mode: false,
            concise_mode: false,
        }
    }
}

impl TodoList {
    #[must_use] pub fn new() -> Self { Self::default() }
    
    #[must_use] pub fn with_mode(fast_mode: bool) -> Self {
        Self { fast_mode, ..Self::default() }
    }
    
    #[must_use] pub fn with_modes(fast_mode: bool, concise_mode: bool) -> Self {
        Self { fast_mode, concise_mode, ..Self::default() }
    }
    
    #[must_use] pub fn is_concise(&self) -> bool {
        self.concise_mode
    }

    pub fn push(&mut self, description: Description, tags: Vec<Tag>) -> Index {
        let idx = self.top_index;
        let item_idx = self.items.len() as u32;
        
        for word in description.0.split_whitespace() {
            with_lower(word, |lower| self.add_word(lower, item_idx));
        }
        
        for tag in &tags {
            with_lower(&tag.0, |lower| self.add_tag(lower, item_idx));
        }
        
        self.done_flags.push(false);
        let item = TodoItem::new(Index(idx), description, tags);
        self.items.push(item);
        self.top_index = idx + 1;
        Index(idx)
    }
    
    fn add_word(&mut self, lower: &str, item_idx: u32) {
        self.add_indexed_term(lower, item_idx, false);
    }
    
    fn add_tag(&mut self, lower: &str, item_idx: u32) {
        self.add_indexed_term(lower, item_idx, true);
    }
    
    /// Add a term (word or tag) to the appropriate index
    fn add_indexed_term(&mut self, lower: &str, item_idx: u32, is_tag: bool) {
        let (terms, term_map, char_idx) = if is_tag {
            (&mut self.tags_vec, &mut self.tag_map, &mut self.tag_char_index)
        } else {
            (&mut self.words, &mut self.word_map, &mut self.char_index)
        };
        
        if let Some(&idx) = term_map.get(lower) {
            terms[idx as usize].items.push(item_idx);
        } else {
            let term_idx = terms.len() as u32;
            let bytes: Box<[u8]> = lower.as_bytes().into();
            let mask = char_mask(&bytes);
            let len = bytes.len().min(255) as u8;
            
            // Add to char index for each unique letter
            let mut added = 0u32;
            for &c in bytes.iter() {
                if c.is_ascii_lowercase() {
                    let ci = (c - b'a') as usize;
                    let bit = 1u32 << ci;
                    if (added & bit) == 0 {
                        added |= bit;
                        char_idx[ci].push(term_idx);
                    }
                }
            }
            
            terms.push(WordInfo { lower: bytes, mask, len, items: vec![item_idx] });
            term_map.insert(lower.into(), term_idx);
        }
    }

    pub fn done_with_index(&mut self, idx: Index) -> Option<Index> {
        let i = idx.0 as usize;
        if i < self.done_flags.len() && !self.done_flags[i] {
            self.done_flags[i] = true;
            self.items[i].done = true;
            return Some(idx);
        }
        None
    }

    #[must_use] 
    pub fn search(&self, sp: &SearchParams) -> Vec<&TodoItem> {
        if sp.words.is_empty() && sp.tags.is_empty() {
            // Natural order (ascending by index)
            return self.items.iter()
                .enumerate()
                .filter(|(i, _)| !self.done_flags[*i])
                .map(|(_, item)| item)
                .collect();
        }
        
        // Use sorted Vec instead of HashSet for better cache locality
        let mut candidates: Option<Vec<u32>> = None;
        
        // Find smallest matching set first
        let mut smallest_idx: Option<usize> = None;
        let mut smallest_size = usize::MAX;
        
        // Collect all matching word indices with their sizes
        let mut word_matches: Vec<(usize, &Vec<u32>)> = Vec::with_capacity(sp.words.len());
        
        for (i, sw) in sp.words.iter().enumerate() {
            let search_bytes = to_lower_bytes(&sw.0);
            
            // In fast mode, only do exact word matches for performance
            if self.fast_mode {
                if let Some(&word_idx) = self.word_map.get(unsafe { std::str::from_utf8_unchecked(&search_bytes) }) {
                    let items = &self.words[word_idx as usize].items;
                    word_matches.push((i, items));
                    if items.len() < smallest_size {
                        smallest_size = items.len();
                        smallest_idx = Some(word_matches.len() - 1);
                    }
                    continue;
                }
                return Vec::new();
            }
            
            if search_bytes.is_empty() { continue; }
            
            let search_mask = char_mask(&search_bytes);
            let search_len = search_bytes.len() as u8;
            
            // Find rarest character in search term
            let mut best_ci = 0;
            let mut best_count = usize::MAX;
            for &c in &search_bytes {
                if c >= b'a' && c <= b'z' {
                    let ci = (c - b'a') as usize;
                    if self.char_index[ci].len() < best_count {
                        best_count = self.char_index[ci].len();
                        best_ci = ci;
                    }
                }
            }
            
            // Parallel: filter matching words and flat_map their items
            let mut matching: Vec<u32> = self.char_index[best_ci]
                .par_iter()
                .filter_map(|&word_idx| {
                    let w = &self.words[word_idx as usize];
                    // Check word length first for performance
                    if w.len >= search_len {
                        // Then check character mask for quick rejection
                        if (search_mask & w.mask) == search_mask {
                            // Finally, verify exact subsequence match
                            if is_subsequence(&search_bytes, &w.lower) {
                                Some(&w.items[..])
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .flatten()
                .copied()
                .collect();
            matching.sort_unstable();
            matching.dedup();
            
            // Intersect with existing candidates or set as initial
            candidates = Some(match candidates.take() {
                None => matching,
                Some(c) => intersect_sorted(&c, &matching),
            });
            
            if candidates.as_ref().map_or(false, |c| c.is_empty()) { 
                return Vec::new(); 
            }
        }
        
        // Now process exact matches starting with smallest
        if !word_matches.is_empty() {
            if let Some(si) = smallest_idx {
                // Start with smallest set
                let (_, items) = word_matches[si];
                let mut result: Vec<u32> = match candidates.take() {
                    None => items.to_vec(),
                    Some(c) => intersect_sorted(&c, items),
                };
                
                // Intersect with other sets
                for (j, (_, items)) in word_matches.iter().enumerate() {
                    if j != si {
                        result = intersect_sorted(&result, items);
                        if result.is_empty() { return Vec::new(); }
                    }
                }
                candidates = Some(result);
            }
        }
        
        for st in &sp.tags {
            let search_bytes = to_lower_bytes(&st.0);
            
            // In fast mode, only do exact tag matches for performance
            if self.fast_mode {
                if let Some(&tag_idx) = self.tag_map.get(unsafe { std::str::from_utf8_unchecked(&search_bytes) }) {
                    let matching = &self.tags_vec[tag_idx as usize].items;
                    candidates = Some(match candidates {
                        None => matching.to_vec(),
                        Some(c) => intersect_sorted(&c, matching),
                    });
                    if candidates.as_ref().map_or(true, |c| c.is_empty()) { 
                        return Vec::new(); 
                    }
                    continue;
                }
                return Vec::new();
            }
            
            if search_bytes.is_empty() { continue; }
            
            let search_mask = char_mask(&search_bytes);
            
            let mut best_ci = 0;
            let mut best_count = usize::MAX;
            for &c in &search_bytes {
                if c >= b'a' && c <= b'z' {
                    let ci = (c - b'a') as usize;
                    if self.tag_char_index[ci].len() < best_count {
                        best_count = self.tag_char_index[ci].len();
                        best_ci = ci;
                    }
                }
            }
            
            let search_len = search_bytes.len() as u8;
            
            // Parallel: filter matching tags and flat_map their items
            let mut matching: Vec<u32> = self.tag_char_index[best_ci]
                .par_iter()
                .filter_map(|&tag_idx| {
                    let t = &self.tags_vec[tag_idx as usize];
                    if t.len >= search_len {
                        if (search_mask & t.mask) == search_mask {
                            if is_subsequence(&search_bytes, &t.lower) {
                                Some(&t.items[..])
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .flatten()
                .copied()
                .collect();
            matching.sort_unstable();
            matching.dedup();
            
            candidates = Some(match candidates {
                None => matching,
                Some(c) => intersect_sorted(&c, &matching),
            });
            if candidates.as_ref().map_or(true, |c| c.is_empty()) { 
                return Vec::new(); 
            }
        }
        
        match candidates {
            Some(c) => {
                let done_flags = &self.done_flags;
                // In fast_mode, limit results for performance (heuristic)
                let limit = if self.fast_mode { 100 } else { usize::MAX };
                
                // Imperative approach for final collection
                let mut result: Vec<&TodoItem> = Vec::with_capacity(c.len().min(limit));
                // Natural order (ascending by index)
                for &i in c.iter() {
                    if !done_flags[i as usize] {
                        result.push(&self.items[i as usize]);
                        if result.len() >= limit { break; }
                    }
                }
                result
            }
            None => Vec::new(),
        }
    }
}

#[inline]
fn intersect_sorted(a: &[u32], b: &[u32]) -> Vec<u32> {
    let mut result = Vec::with_capacity(a.len().min(b.len()));
    let mut i = 0;
    let mut j = 0;
    while i < a.len() && j < b.len() {
        if a[i] < b[j] {
            i += 1;
        } else if a[i] > b[j] {
            j += 1;
        } else {
            result.push(a[i]);
            i += 1;
            j += 1;
        }
    }
    result
}
