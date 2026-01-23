# 🐊 Todo Swamp

یک اپلیکیشن مدیریت Todo ساده اما قدرتمند، نوشته شده با Rust.

این پروژه به عنوان یک **راهنمای یادگیری عملی Rust** طراحی شده. با مطالعه کد و این README، مفاهیم کلیدی Rust رو یاد می‌گیرید.

---

## 📖 فهرست مطالب

- [شروع سریع](#-شروع-سریع)
- [معماری پروژه](#-معماری-پروژه)
- [آموزش Rust با کد پروژه](#-آموزش-rust-با-کد-پروژه)
  - [۱. ساختار پروژه](#۱-ساختار-پروژه-rust)
  - [۲. Newtype Pattern](#۲-newtype-pattern)
  - [۳. Enums و Pattern Matching](#۳-enums-و-pattern-matching)
  - [۴. Traits و Display](#۴-traits-و-display)
  - [۵. Error Handling](#۵-error-handling)
  - [۶. Ownership و Borrowing](#۶-ownership-و-borrowing)
  - [۷. Iterators و Closures](#۷-iterators-و-closures)
  - [۸. Parser Combinators با nom](#۸-parser-combinators-با-nom)
  - [۹. HashMap و Indexing](#۹-hashmap-و-indexing)
  - [۱۰. Testing](#۱۰-testing)
- [دستورات CLI](#-دستورات-cli)
- [بهینه‌سازی‌ها](#-بهینه‌سازی‌ها)

---

## 🚀 شروع سریع

### نیازمندی‌ها
- Rust 1.40+ (نصب با [rustup](https://rustup.rs/))

### اجرا

```bash
# Build
cargo build --release

# اجرای مستقیم
echo "3
add \"Buy milk\" #shopping #urgent
add \"Read Rust book\" #learning
search #shopping" | ./target/release/application
```

**خروجی:**
```
0
1
1 item(s) found
0 "Buy milk" #shopping #urgent
```

### تست‌ها

```bash
cargo test
```

---

## 🏗 معماری پروژه

```
src/
├── lib.rs          # 🔌 Entry point کتابخانه - ماژول‌ها رو export می‌کنه
├── bin/
│   └── application.rs  # 🖥️ باینری اصلی - CLI
├── parser.rs       # 🔍 Parser با nom - تبدیل متن به Query
├── query.rs        # 📝 تایپ‌های Query, QueryResult, QueryError
├── runner.rs       # ⚙️ اجرای Query روی TodoList
└── todo_list.rs    # 📦 ساختار داده‌های اصلی: Index, Tag, TodoItem, TodoList

tests/
└── integration_test.rs  # 🧪 تست‌های end-to-end
```

### جریان داده (Data Flow)

```
Input String → Parser → Query → Runner → TodoList → QueryResult → Output
     ↑                                       ↑
"add \"task\" #tag"                    push/done/search
```

---

## 📚 آموزش Rust با کد پروژه

### ۱. ساختار پروژه Rust

#### Cargo.toml - قلب پروژه
```toml
[package]
name = "todo_swamp"      # نام crate
version = "0.1.0"
edition = "2018"         # نسخه Rust edition

[dependencies]
nom = "5"                # Parser combinator library

[profile.release]
lto = true              # Link Time Optimization
strip = true            # حذف debug symbols
panic = "abort"         # کاهش سایز باینری
```

#### lib.rs vs bin/

```rust
// src/lib.rs - Library crate
pub mod parser;      // ماژول‌های public
pub mod query;
pub mod runner;
pub mod todo_list;

pub use query::*;      // Re-export برای راحتی استفاده
pub use todo_list::*;
```

```rust
// src/bin/application.rs - Binary crate
use todo_swamp::{runner, TodoList};  // استفاده از library

pub fn main() { /* ... */ }
```

**چرا این ساختار؟**
- `lib.rs`: کد قابل استفاده مجدد
- `bin/`: اپلیکیشن‌های executable
- می‌تونید چندین باینری داشته باشید

---

### ۲. Newtype Pattern

**مشکل:** استفاده از `u64` برای index و `String` برای description گیج‌کننده‌ست.

**راه‌حل Rust:**

```rust
// ❌ بد - چی چیه؟
fn add_item(index: u64, desc: String, tag: String) { }

// ✅ خوب - تایپ‌های معنادار
pub struct Index(pub u64);           // Tuple struct
pub struct Description(pub String);
pub struct Tag(pub String);

fn add_item(index: Index, desc: Description, tag: Tag) { }
```

**مزایا:**
- کامپایلر جلوی اشتباه می‌گیره
- کد خواناتر می‌شه
- هزینه runtime صفر (Zero-cost abstraction)

#### پیاده‌سازی در پروژه:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Index(pub u64);

impl Index {
    #[must_use] 
    pub fn new(i: u64) -> Self { Self(i) }
}
```

**نکته‌های Derive:**
| Derive | کاربرد |
|--------|--------|
| `Debug` | چاپ با `{:?}` |
| `Clone` | امکان کپی کردن |
| `Copy` | کپی خودکار (فقط برای تایپ‌های کوچک) |
| `PartialEq, Eq` | مقایسه با `==` |
| `Default` | مقدار پیش‌فرض |

---

### ۳. Enums و Pattern Matching

Enum در Rust خیلی قوی‌تر از زبان‌های دیگه‌ست - می‌تونه **داده** نگه داره:

```rust
pub enum Query {
    Add(Description, Vec<Tag>),  // حمل داده
    Done(Index),
    Search(SearchParams),
}

pub enum QueryResult {
    Added(TodoItem),
    Done,                        // بدون داده
    Found(Vec<TodoItem>),
}
```

**Pattern Matching:**

```rust
fn run_query(q: Query, tl: &mut TodoList) -> Result<QueryResult, QueryError> {
    match q {
        Query::Add(desc, tags) => Ok(QueryResult::Added(tl.push(desc, tags))),
        Query::Done(idx) => tl.done_with_index(idx)
            .map(|_| QueryResult::Done)
            .ok_or_else(|| QueryError(format!("Index {idx} not found"))),
        Query::Search(params) => Ok(QueryResult::Found(
            tl.search(&params).into_iter().cloned().collect()
        )),
    }
}
```

**نکات:**
- `match` باید **exhaustive** باشه (همه حالات رو پوشش بده)
- کامپایلر خطا می‌ده اگه حالتی جا بمونه
- `_` برای "هر چیز دیگه"

---

### ۴. Traits و Display

Trait مثل interface در زبان‌های دیگه‌ست، ولی قوی‌تر:

```rust
use std::fmt::{self, Display};

impl Display for TodoItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} \"{}\"", self.index, self.description)?;
        for tag in &self.tags { 
            write!(f, " #{}", tag.0)?;  // .0 دسترسی به مقدار داخل tuple struct
        }
        Ok(())
    }
}
```

**استفاده:**
```rust
let item = TodoItem::new(Index::new(0), Description::new("task"), vec![]);
println!("{item}");  // خودکار Display صدا زده می‌شه
```

**نکته `?` Operator:**
```rust
write!(f, "{}", x)?;  // اگه خطا داد، برگردون - در غیر این صورت ادامه بده
```

---

### ۵. Error Handling

Rust از `Result<T, E>` برای error handling استفاده می‌کنه:

```rust
pub struct QueryError(pub String);

fn run_query(q: Query, tl: &mut TodoList) -> Result<QueryResult, QueryError> {
    // ...
}
```

**الگوهای رایج:**

```rust
// 1. ok_or_else - تبدیل Option به Result
tl.done_with_index(idx)
    .ok_or_else(|| QueryError(format!("Index {idx} not found")))

// 2. map - تغییر مقدار داخل Ok
.map(|_| QueryResult::Done)

// 3. ? - برگرداندن زودهنگام خطا
let item = get_item()?;  // اگه Err بود، return می‌کنه
```

---

### ۶. Ownership و Borrowing

**مفهوم کلیدی Rust!**

```rust
// Ownership - مالکیت
let s = String::from("hello");  // s مالک هست
let s2 = s;                      // مالکیت منتقل شد
// println!("{s}");              // ❌ خطا! s دیگه معتبر نیست

// Borrowing - قرض دادن
fn print_len(s: &String) {       // &: reference (قرض)
    println!("{}", s.len());
}
let s = String::from("hello");
print_len(&s);                   // s رو قرض می‌دیم
println!("{s}");                 // ✅ s هنوز معتبره

// Mutable borrow
fn add_suffix(s: &mut String) {  // &mut: قرض قابل تغییر
    s.push_str("!");
}
```

**در پروژه:**

```rust
pub fn search(&self, sp: &SearchParams) -> Vec<&TodoItem> {
//            ^^^^^ self رو قرض می‌گیریم (فقط خواندن)
//                      ^^^^^^^^^^^^^ sp هم reference هست
//                                        ^^^^^^^^^ reference به آیتم‌ها برمی‌گردونیم
}

pub fn done_with_index(&mut self, idx: Index) -> Option<Index> {
//                     ^^^^^^^^^ self رو به صورت mutable قرض می‌گیریم (تغییر می‌دیم)
}
```

---

### ۷. Iterators و Closures

**Iterator Chain - خوانا و کارآمد:**

```rust
fn is_subsequence(sub: &str, text: &str) -> bool {
    let mut sub_chars = sub.chars()           // Iterator روی کاراکترها
        .flat_map(char::to_lowercase)         // هر کاراکتر → چند کاراکتر (برای unicode)
        .peekable();                          // امکان peek بدون consume
    
    for ch in text.chars().flat_map(char::to_lowercase) {
        if sub_chars.peek() == Some(&ch) { 
            sub_chars.next(); 
        }
    }
    sub_chars.peek().is_none()
}
```

**Closures (توابع بی‌نام):**

```rust
// سینتکس کامل
let add = |a: i32, b: i32| -> i32 { a + b };

// سینتکس کوتاه (تایپ‌ها استنباط می‌شن)
let add = |a, b| a + b;

// استفاده با iterator
self.items.iter()
    .filter(|item| !item.done)           // فیلتر
    .filter(|item| {                     // closure چند خطی
        sp.tags.iter().all(|tag| /* ... */)
    })
    .collect()
```

**متدهای پرکاربرد Iterator:**
| متد | کاربرد | مثال |
|-----|--------|------|
| `filter` | فیلتر کردن | `.filter(\|x\| x > 5)` |
| `map` | تبدیل | `.map(\|x\| x * 2)` |
| `find` | پیدا کردن اولی | `.find(\|x\| x == 5)` |
| `any/all` | شرط روی همه | `.all(\|x\| x > 0)` |
| `collect` | تبدیل به collection | `.collect::<Vec<_>>()` |
| `flat_map` | map + flatten | `.flat_map(\|x\| x.chars())` |

---

### ۸. Parser Combinators با nom

`nom` یک parser combinator library قدرتمنده. ایده: **parser های کوچک رو ترکیب کن!**

```rust
use nom::{
    branch::alt,                    // این یا اون
    bytes::complete::{tag, take_while1},
    sequence::{pair, preceded, delimited},
    multi::separated_list,
    combinator::opt,
    IResult,
};

// Parser ساده - یک کلمه
fn word(input: &str) -> IResult<&str, &str> { 
    take_while1(|c: char| c.is_ascii_alphabetic())(input) 
}

// ترکیب - description داخل ""
fn description(input: &str) -> IResult<&str, String> {
    delimited(tag("\""), sentence, tag("\""))(input)
        .map(|(rest, desc)| (rest, desc.to_string()))
}

// شاخه‌بندی - add یا done یا search
pub fn query(input: &str) -> IResult<&str, Query> {
    alt((add, done, search))(input.trim())
}
```

**Combinator های کلیدی:**
| Combinator | کاربرد | مثال |
|------------|--------|------|
| `tag` | رشته ثابت | `tag("add")` |
| `take_while1` | حداقل یک کاراکتر با شرط | `take_while1(\|c\| c.is_alphabetic())` |
| `alt` | اولین موفق | `alt((a, b, c))` |
| `pair` | دو parser پشت هم | `pair(tag("a"), tag("b"))` |
| `preceded` | اول رو نادیده بگیر | `preceded(tag("#"), word)` |
| `delimited` | محصور شده | `delimited(tag("\""), text, tag("\""))` |
| `opt` | اختیاری | `opt(space)` |
| `separated_list` | لیست با جداکننده | `separated_list(space, item)` |

---

### ۹. HashMap و Indexing

**بهینه‌سازی جستجو با ایندکس:**

```rust
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TodoList {
    items: Vec<TodoItem>,
    tag_index: HashMap<String, Vec<u64>>,   // tag → [item indices]
    word_index: HashMap<String, Vec<u64>>,  // word → [item indices]
}
```

**ساخت ایندکس هنگام اضافه کردن:**

```rust
pub fn push(&mut self, description: Description, tags: Vec<Tag>) -> TodoItem {
    let idx = self.top_index.0;
    
    // ایندکس کلمات
    for word in description.0.split_whitespace() {
        self.word_index
            .entry(word.to_lowercase())  // اگه key نبود، بساز
            .or_default()                 // Vec خالی پیش‌فرض
            .push(idx);
    }
    
    // ایندکس تگ‌ها
    for tag in &tags {
        self.tag_index
            .entry(tag.0.to_lowercase())
            .or_default()
            .push(idx);
    }
    // ...
}
```

**Entry API:**
```rust
// ❌ کد تکراری
if !map.contains_key(&key) {
    map.insert(key, Vec::new());
}
map.get_mut(&key).unwrap().push(value);

// ✅ با Entry API
map.entry(key).or_default().push(value);
```

---

### ۱۰. Testing

#### Integration Tests

```rust
// tests/integration_test.rs
use assert_cmd::Command;
use predicates::prelude::*;

fn run(input: &str) -> String {
    String::from_utf8(
        Command::cargo_bin("application").unwrap()
            .write_stdin(input)
            .output().unwrap()
            .stdout
    ).unwrap()
}

#[test]
fn test_add_single_item() {
    let output = run("1\nadd \"Buy milk\" #shopping");
    assert_eq!(output.trim(), "0");
}

#[test]
fn test_search() {
    let output = run("2\nadd \"task\" #work\nsearch #work");
    assert!(output.contains("1 item(s) found"));
}
```

**اجرای تست‌ها:**
```bash
cargo test              # همه تست‌ها
cargo test test_add     # فقط تست‌هایی که اسمشون شامل test_add هست
cargo test -- --nocapture  # نمایش println! ها
```

---

## 💻 دستورات CLI

| دستور | شکل | مثال |
|-------|-----|------|
| **Add** | `add "description" #tag1 #tag2` | `add "Buy milk" #shopping #urgent` |
| **Done** | `done <index>` | `done 0` |
| **Search** | `search [words] [#tags]` | `search milk #shopping` |

### ورودی

```
<count>
<command 1>
<command 2>
...
```

### مثال کامل

```
5
add "Buy milk" #shopping #urgent
add "Read Rust book" #learning
add "Call mom"
done 0
search #shopping
```

**خروجی:**
```
0
1
2
done
0 item(s) found
```

---

## ⚡ بهینه‌سازی‌ها

### کد
- **Newtype Pattern**: تایپ‌های معنادار بدون overhead
- **HashMap Indexing**: جستجوی سریع O(1) برای تگ‌ها
- **Iterator Chains**: بدون allocation اضافی
- **Zero-cost abstractions**: انتزاع بدون هزینه runtime

### باینری
```toml
[profile.release]
lto = true      # Link Time Optimization
strip = true    # حذف debug symbols
panic = "abort" # کاهش سایز
```

**نتیجه:** باینری فقط **369KB**!

### کیفیت کد
- ✅ صفر هشدار clippy (حتی pedantic)
- ✅ تمام توابع `#[must_use]` دارن
- ✅ ایمپورت‌های صریح (بدون wildcard)
- ✅ مستندات برای public API

---

## 📝 یادداشت‌های پایانی

این پروژه نشون می‌ده چطور:
1. **ساختار داد** به یه پروژه Rust
2. **تایپ‌های قوی** ساخت
3. **Parser** نوشت با nom
4. **Error handling** انجام داد
5. **تست** نوشت
6. **بهینه‌سازی** کرد

**منابع بیشتر:**
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [nom Documentation](https://docs.rs/nom/)

---

*ساخته شده با ❤️ و Rust*
