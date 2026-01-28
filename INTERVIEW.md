# 📋 مستند مصاحبه - Todo Swamp Project

## 📖 بخش اول: شرح پروژه

### معرفی پروژه
**Todo Swamp** یک اپلیکیشن CLI مدیریت Todo است که با Rust نوشته شده و برای تست Serokell طراحی شده است. این پروژه یک سیستم مدیریت کارها (Todo List) با قابلیت‌های پیشرفته جستجو و بهینه‌سازی‌های عملکردی است.

### عملکرد کلی پروژه
پروژه سه عملیات اصلی را پشتیبانی می‌کند:

1. **`add`** - اضافه کردن یک کار جدید با توضیحات و تگ‌ها
   - ورودی: `add "description" #tag1 #tag2`
   - خروجی: index (شروع از 0)

2. **`done`** - علامت‌گذاری یک کار به عنوان انجام شده
   - ورودی: `done <index>`
   - خروجی: `done`

3. **`search`** - جستجوی کارها بر اساس کلمات و تگ‌ها
   - ورودی: `search <query>` یا `search #tag`
   - خروجی: تعداد + لیست indices (از جدید به قدیم)

### ویژگی‌های کلیدی
- ✅ پشتیبانی از **5 میلیون دستور** در کمتر از 10 ثانیه
- ✅ جستجوی **case-insensitive** (حساس به حروف بزرگ/کوچک نیست)
- ✅ جستجوی **subsequence matching** (مثلاً `a` در `bread` پیدا می‌شود)
- ✅ پردازش موازی با **rayon**
- ✅ بهینه‌سازی‌های حافظه و I/O

---

## 🛠️ بخش دوم: ابزارها، پترن‌ها و تکنیک‌ها

### 1. ابزارها و کتابخانه‌ها

#### Dependencies اصلی:
- **`nom` (v5)**: Parser combinator library برای parsing دستورات ورودی
- **`rayon` (v1.11.0)**: کتابخانه پردازش موازی برای Rust
- **`itoa` (v1)**: تبدیل سریع اعداد به string بدون allocation

#### Dev Dependencies:
- **`assert_cmd`**: برای تست‌های integration
- **`predicates`**: برای assertion در تست‌ها

### 2. ساختار پروژه و معماری

```
src/
├── lib.rs          # Export ماژول‌ها
├── bin/
│   └── application.rs  # Entry point - خواندن stdin و نوشتن stdout
├── parser.rs       # Parser با nom برای parsing دستورات
├── query.rs        # تایپ‌های Query (Add, Done, Search)
├── runner.rs       # اجرای queries و نوشتن خروجی
└── todo_list.rs    # ساختار داده اصلی و الگوریتم‌های جستجو
```

### 3. پترن‌های طراحی استفاده شده

#### 3.1. Newtype Pattern
استفاده از wrapper types برای type safety:
- `Index(u64)` - برای index های todo items
- `Description(String)` - برای توضیحات
- `Tag(String)` - برای تگ‌ها
- `SearchWord(String)` - برای کلمات جستجو

**مزایا:**
- جلوگیری از اشتباهات (مثلاً نمی‌توان Index را با u64 عادی اشتباه گرفت)
- امکان اضافه کردن validation در آینده
- کد خوانا‌تر و type-safe تر

#### 3.2. Builder Pattern (ناقص)
استفاده از `with_modes()` برای ساخت TodoList با تنظیمات خاص:
```rust
TodoList::with_modes(fast_mode, concise_mode)
```

#### 3.3. Strategy Pattern
دو حالت اجرا:
- **Normal mode**: جستجوی کامل با subsequence matching
- **Fast mode**: فقط exact match برای inputs بزرگ (>10K)

### 4. تکنیک‌های بهینه‌سازی

#### 4.1. ساختار داده‌های بهینه

**HashMap برای Exact Match:**
```rust
word_map: HashMap<Box<str>, u32>  // O(1) lookup
tag_map: HashMap<Box<str>, u32>   // O(1) lookup
```

**Char Index برای فیلتر سریع:**
```rust
char_index: [Vec<u32>; 26]  // Index: char -> list of word indices
```
- برای هر حرف (a-z)، لیست کلماتی که شامل آن حرف هستند
- استفاده از نادرترین حرف در query برای شروع جستجو

**Bitmask برای فیلتر سریع:**
```rust
mask: u32  // Bitmask of which letters (a-z) are present
```
- اگر حروف search در کلمه نباشند، skip می‌شود
- عملیات bitwise برای چک سریع

#### 4.2. الگوریتم‌های بهینه

**Exact Match Fast Path:**
- اگر کلمه دقیقاً وجود داشته باشد، از HashMap می‌گیریم (O(1))
- نیازی به subsequence matching نیست

**Smallest Set First:**
- اول کوچکترین مجموعه را پیدا می‌کنیم
- سپس با بقیه intersect می‌کنیم
- کاهش تعداد مقایسه‌ها

**Sorted Vec Intersection:**
- به جای HashSet از sorted Vec استفاده می‌شود
- بهتر برای cache locality
- الگوریتم دو pointer برای intersect

**Reverse Insertion Order:**
- نتایج جستجو از جدید به قدیم مرتب می‌شوند
- استفاده از insertion order طبیعی

#### 4.3. بهینه‌سازی I/O

**BufWriter با buffer 1MB:**
```rust
BufWriter::with_capacity(1 << 20, stdout.lock())
```
- کاهش syscalls
- نوشتن batch به جای تک‌تک

**itoa برای تبدیل اعداد:**
```rust
let mut buffer = itoa::Buffer::new();
out.write_all(buffer.format(idx.0).as_bytes())?;
```
- بدون allocation
- سریع‌تر از `format!` یا `to_string()`

#### 4.4. پردازش موازی

**rayon برای Parallel Processing:**
```rust
self.char_index[best_ci]
    .par_iter()
    .filter_map(|&word_idx| { ... })
```
- استفاده از `par_iter()` برای scan کردن کلمات
- بهینه برای CPU های چند هسته‌ای

#### 4.5. بهینه‌سازی حافظه

**Box<[u8]> به جای String:**
- کمتر allocation
- استفاده از `unsafe { from_utf8_unchecked }` برای جلوگیری از validation اضافی

**Vec<u32> به جای u64:**
- نصف حافظه برای indices
- کافی برای 4+ میلیارد item

**Thread-local Buffer:**
```rust
thread_local! {
    static BUF: std::cell::RefCell<String> = ...
}
```
- استفاده مجدد از buffer برای lowercase conversion
- کاهش allocation

#### 4.6. Heuristics برای Inputs بزرگ

**Fast Mode:**
- برای inputs > 10K دستور
- فقط exact match (skip subsequence matching)
- محدود کردن نتایج به 100 تا

### 5. تکنیک‌های Parsing

**Parser Combinators با nom:**
- استفاده از `nom` برای parsing دستورات
- ترکیب parser های کوچک برای ساخت parser بزرگ
- Error handling خودکار

**مثال:**
```rust
fn add(input: &str) -> IResult<&str, Query> {
    preceded(
        pair(tag("add"), space1),
        pair(description, opt(preceded(space0, tags)))
    )(input)
    .map(|(r, (d, t))| (r, Query::Add(Description::new(&d), t.unwrap_or_default())))
}
```

### 6. Error Handling

- استفاده از `Result` برای error handling
- Custom error type: `QueryError`
- Graceful handling در runner

---

## ❓ بخش سوم: سوالات احتمالی از پروژه

### سوال 1: چرا از HashMap برای exact match استفاده کردید؟

**پاسخ:**
HashMap برای exact match به دلیل پیچیدگی O(1) استفاده شده است. وقتی کاربر یک کلمه کامل را جستجو می‌کند (مثلاً `search bread`)، می‌توانیم مستقیماً از HashMap آن را پیدا کنیم بدون نیاز به scan کردن تمام کلمات. این یک "fast path" است که برای queries متداول بسیار سریع است.

**کد مرتبط:**
```rust
if let Some(&word_idx) = self.word_map.get(unsafe { std::str::from_utf8_unchecked(&search_bytes) }) {
    let items = &self.words[word_idx as usize].items;
    // استفاده مستقیم از items
}
```

### سوال 2: الگوریتم subsequence matching چگونه کار می‌کند؟

**پاسخ:**
الگوریتم subsequence matching چک می‌کند که آیا تمام کاراکترهای query به ترتیب در کلمه وجود دارند یا نه (نه لزوماً پشت سر هم).

**الگوریتم:**
```rust
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
```

**مثال:** `"a"` در `"bread"` پیدا می‌شود چون `a` در `bread` وجود دارد.

**بهینه‌سازی:**
- قبل از subsequence matching، از bitmask و char_index برای فیلتر سریع استفاده می‌شود
- فقط کلماتی که حروف query را دارند چک می‌شوند

### سوال 3: چرا از char_index استفاده کردید؟

**پاسخ:**
char_index یک index معکوس است که برای هر حرف (a-z)، لیست کلماتی که شامل آن حرف هستند را نگه می‌دارد. این برای فیلتر سریع استفاده می‌شود:

1. **پیدا کردن نادرترین حرف:** از بین حروف query، حرفی که در کمترین کلمات وجود دارد انتخاب می‌شود
2. **شروع از کوچکترین مجموعه:** به جای scan کردن تمام کلمات، فقط کلماتی که شامل آن حرف نادر هستند چک می‌شوند
3. **کاهش تعداد مقایسه‌ها:** به جای O(n) مقایسه، O(k) مقایسه که k << n

**کد:**
```rust
char_index: [Vec<u32>; 26]  // Index: char -> list of word indices
```

### سوال 4: چرا از sorted Vec به جای HashSet برای intersection استفاده کردید؟

**پاسخ:**
استفاده از sorted Vec به جای HashSet چند مزیت دارد:

1. **Cache Locality بهتر:** Vec داده‌ها را به صورت sequential در حافظه نگه می‌دارد که برای CPU cache بهتر است
2. **حافظه کمتر:** HashSet overhead بیشتری دارد (hash table + buckets)
3. **الگوریتم دو pointer:** برای sorted arrays، intersection با دو pointer بسیار کارآمد است (O(n+m))

**الگوریتم intersection:**
```rust
fn intersect_sorted(a: &[u32], b: &[u32]) -> Vec<u32> {
    let mut result = Vec::with_capacity(a.len().min(b.len()));
    let mut i = 0;
    let mut j = 0;
    while i < a.len() && j < b.len() {
        if a[i] < b[j] { i += 1; }
        else if a[i] > b[j] { j += 1; }
        else { result.push(a[i]); i += 1; j += 1; }
    }
    result
}
```

### سوال 5: چرا از rayon برای parallel processing استفاده کردید؟

**پاسخ:**
rayon برای پردازش موازی subsequence matching استفاده شده است. وقتی باید تمام کلمات را scan کنیم (در حالت non-exact match)، می‌توانیم این کار را به صورت موازی انجام دهیم:

**مزایا:**
- استفاده از تمام CPU cores
- برای inputs بزرگ، speedup قابل توجه
- API ساده: فقط `par_iter()` به جای `iter()`

**کد:**
```rust
let mut matching: Vec<u32> = self.char_index[best_ci]
    .par_iter()
    .filter_map(|&word_idx| {
        // subsequence matching logic
    })
    .flatten()
    .copied()
    .collect();
```

**نکته:** فقط برای subsequence matching استفاده می‌شود، نه برای exact match (که از HashMap استفاده می‌شود).

### سوال 6: fast_mode چیست و چرا استفاده شده؟

**پاسخ:**
fast_mode یک heuristic برای inputs بزرگ (>10K دستور) است:

**ویژگی‌ها:**
- فقط exact match (skip subsequence matching)
- محدود کردن نتایج به 100 تا
- trade-off بین accuracy و performance

**دلیل:**
برای inputs بسیار بزرگ، subsequence matching می‌تواند کند باشد. با fast_mode، فقط exact matches را برمی‌گردانیم که سریع‌تر است. این یک trade-off است: ممکن است برخی نتایج را از دست بدهیم، اما performance بهتری داریم.

**کد:**
```rust
let fast_mode = count > 10_000;
if self.fast_mode {
    // فقط exact match
    if let Some(&word_idx) = self.word_map.get(...) {
        // ...
    }
    return Vec::new();
}
```

### سوال 7: چرا از unsafe استفاده کردید؟

**پاسخ:**
استفاده از `unsafe { from_utf8_unchecked }` برای جلوگیری از validation اضافی:

```rust
self.word_map.get(unsafe { std::str::from_utf8_unchecked(&search_bytes) })
```

**دلیل:**
- ما خودمان `to_lower_bytes()` را صدا می‌زنیم که از ASCII bytes استفاده می‌کند
- می‌دانیم که bytes معتبر UTF-8 هستند
- validation اضافی overhead دارد

**نکته امنیتی:** این safe است چون:
1. فقط برای ASCII characters استفاده می‌شود
2. `to_lower_bytes()` فقط ASCII را تولید می‌کند
3. در fast_mode فقط exact matches استفاده می‌شود

### سوال 8: چرا از thread-local buffer استفاده کردید؟

**پاسخ:**
برای lowercase conversion، از thread-local buffer استفاده می‌شود تا allocation را کاهش دهیم:

```rust
thread_local! {
    static BUF: std::cell::RefCell<String> = const { ... };
}
```

**مزایا:**
- استفاده مجدد از buffer بین calls
- کاهش allocation
- Thread-safe (هر thread buffer خودش را دارد)

**استفاده:**
```rust
with_lower(word, |lower| self.add_word(lower, item_idx));
```

### سوال 9: چرا از BufWriter با capacity 1MB استفاده کردید؟

**پاسخ:**
BufWriter با buffer بزرگ برای کاهش syscalls استفاده می‌شود:

```rust
BufWriter::with_capacity(1 << 20, stdout.lock())  // 1MB
```

**مزایا:**
- کاهش تعداد write syscalls
- نوشتن batch به جای تک‌تک
- برای outputs بزرگ، performance بهتر

**Trade-off:**
- حافظه بیشتر (1MB)
- اما برای performance بهتر، ارزش دارد

### سوال 10: چرا از itoa به جای format! استفاده کردید؟

**پاسخ:**
itoa برای تبدیل اعداد به string بدون allocation استفاده می‌شود:

```rust
let mut buffer = itoa::Buffer::new();
out.write_all(buffer.format(idx.0).as_bytes())?;
```

**مزایا:**
- بدون allocation (stack-allocated buffer)
- سریع‌تر از `format!` یا `to_string()`
- برای I/O-heavy applications مهم است

---

## 🦀 بخش چهارم: سوالات رایج مصاحبه Rust Backend Development

### سوال 1: تفاوت بین `&str` و `String` چیست؟

**پاسخ:**
- **`&str`**: یک slice به string data (borrowed)
  - Fixed size (pointer + length)
  - Immutable reference
  - Stack-allocated
  - مثال: `let s: &str = "hello";`

- **`String`**: یک owned, growable string
  - Heap-allocated
  - Mutable
  - Dynamic size
  - مثال: `let s: String = String::from("hello");`

**استفاده:**
- `&str` برای function parameters (بیشتر موارد)
- `String` وقتی ownership نیاز است یا string باید grow کند

**در پروژه:**
- `Description(String)` - owned string برای storage
- Function parameters از `&str` استفاده می‌کنند

### سوال 2: Ownership و Borrowing چیست؟

**پاسخ:**
Ownership یکی از ویژگی‌های کلیدی Rust است که memory safety را بدون GC تضمین می‌کند:

**قوانین:**
1. هر value یک owner دارد
2. فقط یک owner در هر زمان
3. وقتی owner از scope خارج می‌شود، value drop می‌شود

**Borrowing:**
- `&T`: immutable borrow (می‌تواند چند تا باشد)
- `&mut T`: mutable borrow (فقط یکی در هر زمان)

**مثال:**
```rust
let s = String::from("hello");
let len = calculate_length(&s);  // borrow
// s هنوز valid است
```

**در پروژه:**
- `&TodoList` برای search (immutable borrow)
- `&mut TodoList` برای add/done (mutable borrow)

### سوال 3: تفاوت بین `Vec` و `Box<[T]>` چیست؟

**پاسخ:**
- **`Vec<T>`**: Growable array
  - Heap-allocated
  - می‌تواند grow/shrink کند
  - Overhead: pointer + length + capacity

- **`Box<[T]>`**: Fixed-size slice on heap
  - Heap-allocated
  - Fixed size (نمی‌تواند grow کند)
  - Overhead: فقط pointer + length

**استفاده:**
- `Vec` وقتی size ممکن است تغییر کند
- `Box<[T]>` وقتی size ثابت است و می‌خواهیم overhead کمتر

**در پروژه:**
```rust
lower: Box<[u8]>  // Fixed size, کمتر allocation
```

### سوال 4: `Result<T, E>` و `Option<T>` چیست؟

**پاسخ:**
- **`Option<T>`**: برای values که ممکن است وجود نداشته باشند
  - `Some(T)`: value وجود دارد
  - `None`: value وجود ندارد

- **`Result<T, E>`**: برای operations که ممکن است fail کنند
  - `Ok(T)`: success
  - `Err(E)`: error

**استفاده:**
```rust
// Option
fn find_item(id: u32) -> Option<Item> { ... }

// Result
fn parse_query(s: &str) -> Result<Query, ParseError> { ... }
```

**در پروژه:**
- `done_with_index()` returns `Option<Index>`
- `parser::query()` returns `IResult<&str, Query>` (nom's Result)

### سوال 5: Lifetime چیست و چرا مهم است؟

**پاسخ:**
Lifetime مشخص می‌کند که یک reference چقدر valid است:

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

**قوانین:**
1. هر reference یک lifetime دارد
2. Lifetime elision: در بسیاری موارد، compiler خودش infer می‌کند
3. Explicit lifetime وقتی ambiguous است

**در پروژه:**
```rust
pub fn search(&self, sp: &SearchParams) -> Vec<&TodoItem>
//              ^      ^                    ^
//              |      |                    |
//              |      |                    lifetime از self
//              |      lifetime parameter
//              self borrow
```

### سوال 6: تفاوت بین `clone()` و `copy` trait چیست؟

**پاسخ:**
- **`Copy` trait**: Types که automatically copy می‌شوند
  - Stack-only types (numbers, bool, char)
  - No ownership transfer
  - مثال: `i32`, `bool`, `char`

- **`clone()`**: Explicit deep copy
  - Heap-allocated types معمولاً
  - Ownership transfer یا copy
  - مثال: `String`, `Vec`

**در پروژه:**
- `Index(u64)` - Copy (u64 is Copy)
- `Description(String)` - Clone (String is not Copy)

### سوال 7: `Send` و `Sync` traits چیست؟

**پاسخ:**
- **`Send`**: Type می‌تواند بین threads transfer شود
  - مثال: `String`, `Vec`, `Arc<T>`

- **`Sync`**: Type می‌تواند بین threads shared شود (via `&T`)
  - مثال: `String`, `Vec`, `Arc<T>`, `Mutex<T>`

**استفاده:**
- `rayon` نیاز به `Send + Sync` دارد
- در پروژه، `TodoList` باید `Send + Sync` باشد برای parallel processing

### سوال 8: `Arc` و `Rc` چیست و تفاوت‌شان چیست؟

**پاسخ:**
- **`Rc<T>`**: Reference counting (single-threaded)
  - برای single-threaded code
  - Overhead کمتر

- **`Arc<T>`**: Atomic reference counting (multi-threaded)
  - برای multi-threaded code
  - Thread-safe
  - Overhead بیشتر (atomic operations)

**استفاده:**
```rust
// Single-threaded
let data = Rc::new(data);

// Multi-threaded
let data = Arc::new(data);
```

**در پروژه:**
- از `rayon` استفاده می‌شود (multi-threaded)
- اما `TodoList` را share نمی‌کنیم (immutable borrow در search)

### سوال 9: `Mutex` و `RwLock` چیست؟

**پاسخ:**
- **`Mutex<T>`**: Mutual exclusion
  - فقط یک thread می‌تواند access کند
  - Read/write lock

- **`RwLock<T>`**: Read-write lock
  - چند readers یا یک writer
  - برای read-heavy workloads بهتر

**استفاده:**
```rust
let data = Arc::new(Mutex::new(data));
let data = data.lock().unwrap();
```

**در پروژه:**
- از mutex استفاده نمی‌شود
- `search()` immutable borrow می‌گیرد
- `add()`/`done()` mutable borrow می‌گیرند

### سوال 10: Pattern Matching چیست؟

**پاسخ:**
Pattern matching یکی از ویژگی‌های قدرتمند Rust:

```rust
match value {
    Pattern1 => action1,
    Pattern2 => action2,
    _ => default,
}
```

**استفاده:**
- `match` expressions
- `if let` برای single pattern
- `while let` برای loops

**در پروژه:**
```rust
match q {
    Query::Add(desc, tags) => Ok(QueryResultRef::Added(tl.push(desc, tags))),
    Query::Done(idx) => ...,
    Query::Search(params) => ...,
}
```

### سوال 11: Iterator و Iterator Adapters چیست؟

**پاسخ:**
Iterator trait برای lazy evaluation و functional programming:

```rust
let result: Vec<_> = items
    .iter()
    .filter(|x| x > 5)
    .map(|x| x * 2)
    .collect();
```

**Adapters:**
- `map`: transform
- `filter`: filter
- `take`: limit
- `collect`: collect to collection

**در پروژه:**
```rust
self.items.iter()
    .enumerate()
    .filter(|(i, _)| !self.done_flags[*i])
    .map(|(_, item)| item)
    .collect()
```

### سوال 12: Error Handling در Rust چگونه است؟

**پاسخ:**
Rust از `Result<T, E>` برای error handling استفاده می‌کند:

```rust
fn parse_number(s: &str) -> Result<i32, ParseIntError> {
    s.parse()
}

// استفاده
match parse_number("123") {
    Ok(n) => println!("{}", n),
    Err(e) => println!("Error: {}", e),
}

// یا با ?
fn process() -> Result<(), Error> {
    let n = parse_number("123")?;  // propagate error
    Ok(())
}
```

**در پروژه:**
```rust
pub fn run_line_buffered(...) {
    if let Ok((_, q)) = parser::query(trimmed) {
        match run_query_ref(q, tl) {
            Ok(r) => ...,
            Err(e) => ...,
        }
    }
}
```

### سوال 13: `unsafe` چیست و چه زمانی استفاده می‌شود؟

**پاسخ:**
`unsafe` برای bypass کردن borrow checker استفاده می‌شود:

**استفاده‌های مجاز:**
1. Raw pointers
2. Calling unsafe functions
3. Accessing mutable static variables
4. Implementing unsafe traits

**نکته:** `unsafe` فقط compiler checks را disable می‌کند، memory safety را تضمین نمی‌کند!

**در پروژه:**
```rust
unsafe { std::str::from_utf8_unchecked(&search_bytes) }
```
- Safe چون می‌دانیم bytes معتبر UTF-8 هستند (از ASCII)

### سوال 14: تفاوت بین `const` و `static` چیست؟

**پاسخ:**
- **`const`**: Compile-time constant
  - Inlined در هر استفاده
  - No memory address
  - مثال: `const MAX: u32 = 100;`

- **`static`**: Global variable با fixed memory address
  - یک instance در کل program
  - Mutable با `unsafe`
  - مثال: `static MAX: u32 = 100;`

**در پروژه:**
```rust
thread_local! {
    static BUF: std::cell::RefCell<String> = const { ... };
}
```

### سوال 15: Trait چیست و چگونه استفاده می‌شود؟

**پاسخ:**
Trait مشابه interface در زبان‌های دیگر است:

```rust
trait Animal {
    fn speak(&self);
}

struct Dog;
impl Animal for Dog {
    fn speak(&self) {
        println!("Woof!");
    }
}
```

**استفاده:**
- Polymorphism
- Generic constraints
- Extension methods

**در پروژه:**
- `Display` trait برای formatting
- `Default` trait برای default values

### سوال 16: Generic Functions و Type Parameters چیست؟

**پاسخ:**
Generics برای code reuse:

```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    // ...
}
```

**Constraints:**
- `T: Trait` - T must implement Trait
- `T: Clone + Send` - Multiple bounds

**در پروژه:**
```rust
pub fn search(&self, sp: &SearchParams) -> Vec<&TodoItem>
// Generic نیست، اما می‌توانست باشد
```

### سوال 17: `Box`, `Rc`, `Arc` - چه زمانی استفاده می‌شوند؟

**پاسخ:**
- **`Box<T>`**: Heap allocation, single owner
  - برای recursive types
  - برای large types on stack

- **`Rc<T>`**: Reference counting (single-threaded)
  - Multiple owners, single thread

- **`Arc<T>`**: Atomic reference counting (multi-threaded)
  - Multiple owners, multiple threads

**در پروژه:**
```rust
lower: Box<[u8]>  // Fixed-size array on heap
```

### سوال 18: Memory Management در Rust چگونه است؟

**پاسخ:**
Rust از **RAII (Resource Acquisition Is Initialization)** استفاده می‌کند:

1. **Ownership**: هر value یک owner دارد
2. **Drop**: وقتی owner از scope خارج می‌شود، value drop می‌شود
3. **No GC**: بدون garbage collector
4. **Zero-cost abstractions**: abstractions بدون runtime overhead

**مثال:**
```rust
{
    let s = String::from("hello");
    // s در scope است
}  // s drop می‌شود، memory آزاد می‌شود
```

### سوال 19: تفاوت بین `str` و `String` در memory layout چیست؟

**پاسخ:**
- **`str`**: Fat pointer
  - `*const u8` (pointer to data)
  - `usize` (length)
  - Stack-allocated

- **`String`**: Struct
  - `*mut u8` (pointer to heap)
  - `usize` (length)
  - `usize` (capacity)
  - Heap-allocated data

**Memory:**
```
String: [ptr|len|cap] -> [heap data]
&str:   [ptr|len]     -> [data]
```

### سوال 20: Performance Tips برای Rust Backend چیست؟

**پاسخ:**
1. **Avoid allocations**: استفاده از `&str` به جای `String` وقتی ممکن است
2. **Use `Vec::with_capacity()`**: برای pre-allocate
3. **Use `slice`**: به جای clone کردن
4. **Profile**: استفاده از `cargo flamegraph` یا `perf`
5. **Release mode**: `cargo build --release`
6. **LTO**: Link-time optimization
7. **Avoid `clone()`**: تا جایی که ممکن است
8. **Use iterators**: به جای loops (بهتر optimize می‌شوند)

**در پروژه:**
- `Vec::with_capacity()` برای pre-allocation
- `itoa` برای بدون allocation number formatting
- `BufWriter` برای batch I/O
- `rayon` برای parallelism

---

## 📝 خلاصه نکات کلیدی برای مصاحبه

### نکات پروژه:
1. ✅ استفاده از HashMap برای O(1) exact match
2. ✅ Char index برای فیلتر سریع
3. ✅ Bitmask برای quick rejection
4. ✅ Sorted Vec intersection برای cache locality
5. ✅ Parallel processing با rayon
6. ✅ بهینه‌سازی I/O با BufWriter و itoa
7. ✅ Fast mode heuristic برای inputs بزرگ

### نکات Rust:
1. ✅ Ownership و Borrowing
2. ✅ Lifetime annotations
3. ✅ Error handling با Result
4. ✅ Pattern matching
5. ✅ Iterator adapters
6. ✅ Memory management (RAII)
7. ✅ Performance optimization techniques

---

**موفق باشید در مصاحبه! 🚀**

