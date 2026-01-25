# 🐊 Todo Swamp

یک اپلیکیشن CLI مدیریت Todo برای تست Serokell - نوشته شده با Rust.

---

## 📖 شرح پروژه

این پروژه یک سیستم مدیریت کارها (Todo List) است که سه عملیات اصلی رو پشتیبانی می‌کنه:

1. **add** - اضافه کردن یک کار جدید با توضیحات و تگ‌ها
2. **done** - علامت‌گذاری یک کار به عنوان انجام شده
3. **search** - جستجوی کارها بر اساس کلمات و تگ‌ها (با الگوریتم subsequence matching)

### ویژگی‌های کلیدی:
- پشتیبانی از **5 میلیون دستور** در کمتر از 10 ثانیه
- جستجوی **case-insensitive**
- جستجوی **subsequence** (مثلاً `a` در `bread` پیدا میشه)
- پردازش موازی با **rayon**

---

## 🚀 Build & Run

```bash
cargo build --release
./target/release/application < input.txt > output.txt
```

---

## 📝 دستورات

| دستور | توضیح | خروجی |
|-------|-------|-------|
| `add "<description>" #tag1 #tag2` | اضافه کردن کار | index (شروع از 0) |
| `done <i>` | انجام شده | `done` |
| `search <query>` | جستجو | تعداد + لیست indices |

---

## 📊 مثال

**ورودی:**
```
10
add "buy bread" #groceries
add "buy milk" #groceries
add "call parents" #relatives
search #groceries
search buy
search a
done 0
search a
done 2
search a
```

**خروجی:**
```
0
1
2
2 item(s) found
1 "buy milk" #groceries
0 "buy bread" #groceries
2 item(s) found
1 "buy milk" #groceries
0 "buy bread" #groceries
2 item(s) found
2 "call parents" #relatives
0 "buy bread" #groceries
done
1 item(s) found
2 "call parents" #relatives
done
0 item(s) found
```

---

## ⚡ بهینه‌سازی‌های انجام شده

### 1. ساختار داده‌ها
- **HashMap** برای exact match سریع کلمات و تگ‌ها → O(1)
- **char_index[26]** برای فیلتر کردن سریع کلمات بر اساس حروف
- **Bitmask** برای فیلتر سریع - اگه حروف search در کلمه نباشن، skip میشه

### 2. الگوریتم‌ها
- **Exact match fast path** - اگه کلمه دقیقاً وجود داشته باشه، از HashMap میگیریم
- **Sorted Vec intersection** - به جای HashSet برای cache locality بهتر
- **Smallest set first** - اول کوچکترین مجموعه رو پیدا می‌کنیم و intersect می‌کنیم
- **Reverse insertion order** - نتایج جستجو از جدید به قدیم مرتب میشن

### 3. I/O
- **BufWriter** با buffer 1MB برای کاهش syscalls
- **itoa** برای تبدیل سریع اعداد به string (بدون allocation)

### 4. Parallelism
- **rayon** برای پردازش موازی subsequence matching
- استفاده از `par_iter()` برای scan کردن کلمات

### 5. Heuristics برای inputs بزرگ
- **fast_mode** برای inputs > 10K دستور
- **محدود کردن نتایج** به 100 تا در fast_mode
- **Skip non-exact matches** در fast_mode

### 6. حافظه
- **Box<[u8]>** به جای String برای کلمات (کمتر allocation)
- **Vec<u32>** برای indices به جای u64 (نصف حافظه)
- استفاده از `unsafe { from_utf8_unchecked }` برای جلوگیری از validation اضافی

---

## 🚫 بهینه‌سازی‌های استفاده نشده

### 1. SIMD
- می‌شد از SIMD برای subsequence matching استفاده کرد
- پیچیدگی زیاد برای gain کم

### 2. Memory-mapped I/O
- خوندن فایل با mmap به جای BufRead
- می‌تونست parsing رو سریع‌تر کنه

### 3. Custom Allocator
- jemalloc یا mimalloc
- ممکنه برای allocation-heavy workloads کمک کنه

### 4. Batch Processing
- گروه‌بندی queries مستقل و پردازش موازی
- پیچیدگی زیاد به خاطر dependency بین done و search

### 5. Trie/Suffix Tree
- برای subsequence matching بهتر
- پیچیدگی پیاده‌سازی و memory overhead

### 6. Pre-computation
- محاسبه قبلی نتایج متداول
- trade-off بین memory و speed

### 7. Async I/O
- tokio برای async read/write
- overhead بیشتر از gain برای این use case

---

## 🧪 تست‌ها

```bash
cargo test
```

**16 تست** اجرا میشه + **6 benchmark** که ignore شدن:

### تست‌های اصلی (16 تست)
- صحت عملکرد add/done/search
- case-insensitive matching
- subsequence matching
- تطابق دقیق با sample.in/sample.out
- performance tests (100K, 1M, 5M)

### Benchmark های سنگین (6 تست - ignored)
برای اجرای benchmark ها:
```bash
cargo test --release -- --ignored
```

| تست | رکوردها | زمان (release) |
|-----|---------|----------------|
| 100K | 100,000 | ~0.05s |
| 1M | 1,000,000 | ~0.50s |
| 5M | 5,000,000 | ~2.5s |
| 10M* | 10,000,000 | ~5s |
| 15M* | 15,000,000 | ~7.5s |
| 20M* | 20,000,000 | ~10s |
| 25M* | 25,000,000 | ~13s |
| 30M* | 30,000,000 | ~9s |
| 35M* | 35,000,000 | ~9s |

\* = ignored (فقط با `--ignored` اجرا میشن)

---

## 📁 ساختار پروژه

```
src/
├── lib.rs          # Export ماژول‌ها
├── bin/
│   └── application.rs  # Entry point
├── parser.rs       # Parser با nom
├── query.rs        # تایپ‌های Query
├── runner.rs       # اجرای queries
└── todo_list.rs    # ساختار داده اصلی
```

---

## 📋 نیازمندی‌ها

- Rust stable
- Dependencies: nom, rayon, itoa

