use crate::{Description, Index, Query, SearchParams, SearchWord, Tag};
use nom::{
    branch::alt, bytes::complete::{tag, take_while, take_while1},
    character::complete::{digit1, space0, space1}, combinator::opt,
    multi::{many1, separated_list}, sequence::{delimited, pair, preceded}, IResult,
};

/// Parses a query string into a Query enum.
/// # Errors
/// Returns a nom error if the input doesn't match any valid query format.
pub fn query(input: &str) -> IResult<&str, Query> {
    alt((add, done, search))(input.trim())
}

fn add(input: &str) -> IResult<&str, Query> {
    preceded(pair(tag("add"), space1), pair(description, opt(preceded(space0, tags))))(input)
        .map(|(r, (d, t))| (r, Query::Add(Description::new(&d), t.unwrap_or_default())))
}

fn is_word_char(c: char) -> bool { c.is_ascii_alphabetic() || c == '-' }
fn is_sentence_char(c: char) -> bool { is_word_char(c) || c.is_whitespace() }

fn word(input: &str) -> IResult<&str, &str> { take_while1(is_word_char)(input) }
fn sentence(input: &str) -> IResult<&str, &str> { take_while(is_sentence_char)(input) }
fn todo_tag(input: &str) -> IResult<&str, &str> { preceded(tag("#"), word)(input) }

fn description(input: &str) -> IResult<&str, String> {
    delimited(tag("\""), sentence, tag("\""))(input).map(|(r, d)| (r, d.to_string()))
}

fn tags(input: &str) -> IResult<&str, Vec<Tag>> {
    separated_list(space1, todo_tag)(input).map(|(r, t)| (r, t.into_iter().map(Tag::new).collect()))
}

fn done(input: &str) -> IResult<&str, Query> {
    preceded(pair(tag("done"), space1), many1(digit1))(input)
        .map(|(r, d)| (r, Query::Done(Index::new(d.concat().parse().unwrap()))))
}

enum WordOrTag { Word(String), Tag(String) }

fn search(input: &str) -> IResult<&str, Query> {
    preceded(tag("search"), opt(preceded(space1, separated_list(space1, word_or_tag))))(input)
        .map(|(r, m)| (r, to_query(m.unwrap_or_default())))
}

fn word_or_tag(input: &str) -> IResult<&str, WordOrTag> {
    alt((
        |i| pair(tag("#"), word)(i).map(|(r, (_, w))| (r, WordOrTag::Tag(w.to_string()))),
        |i| word(i).map(|(r, w)| (r, WordOrTag::Word(w.to_string())))
    ))(input)
}

fn to_query(items: Vec<WordOrTag>) -> Query {
    let (mut words, mut tags) = (Vec::new(), Vec::new());
    for item in items {
        match item {
            WordOrTag::Word(w) => words.push(SearchWord::new(&w)),
            WordOrTag::Tag(t) => tags.push(Tag::new(&t)),
        }
    }
    Query::Search(SearchParams { words, tags })
}
