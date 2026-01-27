use assert_cmd::Command;
use std::fs;
use std::time::{Duration, Instant};

fn get_binary() -> Command {
    Command::cargo_bin("application").unwrap()
}

fn get_binary_with_timeout(secs: u64) -> Command {
    let mut cmd = get_binary();
    cmd.timeout(Duration::from_secs(secs));
    cmd
}

fn normalize_output(output: &str) -> Vec<String> {
    output.lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn run_and_get_output(input: &str) -> String {
    let output = get_binary()
        .write_stdin(input)
        .output()
        .expect("Failed to execute command");
    assert!(output.status.success(), "Command failed: {:?}", output);
    String::from_utf8(output.stdout).unwrap()
}

/// Run a performance test with given parameters
fn run_perf_test(name: &str, input: String, timeout_secs: u64, max_secs: f64) {
    let start = Instant::now();
    let output = get_binary_with_timeout(timeout_secs)
        .write_stdin(input)
        .output()
        .expect(&format!("{} timed out after {} seconds", name, timeout_secs));
    assert!(output.status.success(), "{} failed or timed out", name);
    let elapsed = start.elapsed().as_secs_f64();
    println!("⏱️  {}: {:.2}s", name, elapsed);
    assert!(elapsed < max_secs, "{} took {:.2}s, expected < {:.0}s", name, elapsed, max_secs);
}

#[test]
fn test_sample_input() {
    let input = fs::read_to_string("tests/fixtures/sample.in").unwrap();
    let expected = fs::read_to_string("tests/fixtures/sample.out").unwrap();
    let actual = run_and_get_output(&input);
    
    let mut expected_lines = normalize_output(&expected);
    let mut actual_lines = normalize_output(&actual);
    expected_lines.sort();
    actual_lines.sort();
    
    assert_eq!(expected_lines, actual_lines, "Output mismatch!\nExpected:\n{}\nActual:\n{}", expected, actual);
}

#[test]
fn test_bishibosh_input() {
    let input = fs::read_to_string("tests/fixtures/Bishibosh.in").unwrap();
    let expected = fs::read_to_string("tests/fixtures/Bishibosh.out").unwrap();
    let actual = run_and_get_output(&input);
    
    assert_eq!(expected, actual, "Bishibosh output mismatch!");
}

#[test]
fn test_flamespike_the_crawler_input() {
    let input = fs::read_to_string("tests/fixtures/Flamespike-The-Crawler.in").unwrap();
    let expected = fs::read_to_string("tests/fixtures/Flamespike-The-Crawler.out").unwrap();
    let actual = run_and_get_output(&input);
    
    assert_eq!(expected, actual, "Flamespike-The-Crawler output mismatch");
}

#[test]
fn test_add_index_output() {
    let actual = run_and_get_output("2\nadd \"itemone\" #tagone\nadd \"itemtwo\" #tagtwo\n");
    assert_eq!(normalize_output("0\n1"), normalize_output(&actual));
}

#[test]
fn test_done_command() {
    let actual = run_and_get_output("2\nadd \"task\" #work\ndone 0\n");
    assert_eq!(normalize_output("0\ndone"), normalize_output(&actual));
}

#[test]
fn test_search_with_tag() {
    let actual = run_and_get_output("3\nadd \"buy bread\" #groceries\nadd \"buy milk\" #groceries\nsearch #groceries\n");
    // Per PDF spec: text representation is optional, checker ignores it
    // Just verify count and that we got results
    assert!(actual.contains("2 item(s) found"));
}

#[test]
fn test_done_filtering() {
    let actual = run_and_get_output("4\nadd \"taskone\" #work\nadd \"tasktwo\" #work\ndone 0\nsearch #work\n");
    // After done 0, search should return only index 1
    assert!(actual.contains("1 item(s) found"));
    let lines: Vec<&str> = actual.lines().collect();
    // Find the line after "1 item(s) found" - should start with "1"
    let found_idx = lines.iter().position(|l| l.contains("1 item(s) found")).unwrap();
    assert!(lines[found_idx + 1].trim().starts_with("1"));
}

#[test]
fn test_subsequence_matching() {
    let actual = run_and_get_output("2\nadd \"bread\" #food\nsearch a\n");
    // "a" is subsequence of "bread", should find 1 item (index 0)
    assert!(actual.contains("1 item(s) found"));
}

#[test]
fn test_empty_search() {
    let actual = run_and_get_output("3\nadd \"itemone\" #tagone\nadd \"itemtwo\" #tagtwo\nsearch\n");
    // Empty search returns all items
    assert!(actual.contains("2 item(s) found"));
}

#[test]
fn test_case_insensitive_matching() {
    let actual = run_and_get_output("2\nadd \"BREAD\" #FOOD\nsearch bread\n");
    // Case insensitive - should find 1 item
    assert!(actual.contains("1 item(s) found"));
}

#[test]
fn test_multiple_words_search() {
    let actual = run_and_get_output("3\nadd \"buy bread\" #groceries\nadd \"buy milk\" #groceries\nsearch buy bread\n");
    // Only "buy bread" has both words
    assert!(actual.contains("1 item(s) found"));
}

#[test]
fn test_mixed_query() {
    let actual = run_and_get_output("3\nadd \"buy bread\" #groceries\nadd \"buy milk\" #groceries\nsearch buy #groceries\n");
    // Both items have "buy" and #groceries
    assert!(actual.contains("2 item(s) found"));
}

#[test]
fn test_output_format() {
    let actual = run_and_get_output("2\nadd \"test\" #tag\nsearch test\n");
    assert!(actual.contains("item(s) found"));
    // Check no trailing whitespace
    for line in actual.lines() {
        assert!(!line.ends_with(' ') && !line.ends_with('\t'));
    }
}

#[test]
fn test_performance_medium() {
    const WORDS: [&str; 10] = ["alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "iota", "kappa"];
    const TAGS: [&str; 10] = ["tagone", "tagtwo", "tagthree", "tagfour", "tagfive", "tagsix", "tagseven", "tageight", "tagnine", "tagten"];
    
    let mut input = String::from("1000\n");
    for i in 0..500 {
        input.push_str(&format!("add \"{} {} description\" #{}\n", WORDS[i % 10], WORDS[(i / 10) % 10], TAGS[i % 10]));
    }
    for i in 0..500 {
        input.push_str(&format!("search #{}\n", TAGS[i % 10]));
    }
    run_perf_test("test_performance_medium", input, 10, 10.0);
}

#[test]
fn test_performance_large() {
    const WORDS: [&str; 50] = [
        "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "iota", "kappa",
        "lambda", "mu", "nu", "xi", "omicron", "pi", "rho", "sigma", "tau", "upsilon",
        "phi", "chi", "psi", "omega", "one", "two", "three", "four", "five", "six",
        "seven", "eight", "nine", "ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen",
        "seventeen", "eighteen", "nineteen", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty"
    ];
    
    let mut input = String::from("10000\n");
    for i in 0..5000 {
        input.push_str(&format!("add \"{} {} with description\" #{}\n", WORDS[i % 50], WORDS[(i / 50) % 50], WORDS[i % 50]));
    }
    for i in 0..5000 {
        input.push_str(&format!("search #{}\n", WORDS[i % 50]));
    }
    run_perf_test("test_performance_large", input, 10, 10.0);
}

// Helper to generate random-ish words based on index
fn generate_word(i: usize) -> String {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    let mut word = String::with_capacity(6);
    let mut n = i;
    for _ in 0..6 {
        word.push(CHARS[n % 26] as char);
        n /= 26;
    }
    word
}

#[test]
fn test_performance_100k() {
    let n = 100_000;
    let adds = n / 2;
    let searches = n / 2;
    
    let mut input = format!("{}\n", n);
    for i in 0..adds {
        let w1 = generate_word(i);
        let w2 = generate_word(i * 7);
        let tag = generate_word(i % 1000);
        input.push_str(&format!("add {} {} #{}\n", w1, w2, tag));
    }
    for i in 0..searches {
        let term = &generate_word(i % 1000)[..3];
        input.push_str(&format!("search {}\n", term));
    }
    
    let start = Instant::now();
    let output = get_binary_with_timeout(10)
        .write_stdin(input)
        .output()
        .expect("Command timed out after 10 seconds");
    assert!(output.status.success(), "Performance test 100K failed or timed out");
    let elapsed = start.elapsed().as_secs_f64();
    println!("⏱️  test_performance_100k: {:.2}s", elapsed);
    assert!(elapsed < 10.0, "100K commands took {:.2}s, expected < 10s", elapsed);
}

#[test]
fn test_performance_1m() {
    let n = 1_000_000;
    let adds = n / 2;
    let searches = n / 2;
    
    let mut input = format!("{}\n", n);
    for i in 0..adds {
        let w1 = generate_word(i);
        let w2 = generate_word(i * 7);
        let tag = generate_word(i % 5000);
        input.push_str(&format!("add {} {} #{}\n", w1, w2, tag));
    }
    for i in 0..searches {
        let term = &generate_word(i % 5000)[..3];
        input.push_str(&format!("search {}\n", term));
    }
    
    let start = Instant::now();
    let output = get_binary_with_timeout(10)
        .write_stdin(input)
        .output()
        .expect("Command timed out after 10 seconds");
    assert!(output.status.success(), "Performance test 1M failed or timed out");
    let elapsed = start.elapsed().as_secs_f64();
    println!("⏱️  test_performance_1m: {:.2}s", elapsed);
    assert!(elapsed < 10.0, "1M commands took {:.2}s, expected < 10s", elapsed);
}

#[test]
fn test_performance_5m() {
    let n = 5_000_000;
    let adds = n / 2;
    let searches = n / 2;
    
    let mut input = format!("{}\n", n);
    for i in 0..adds {
        let w1 = generate_word(i);
        let w2 = generate_word(i * 7);
        let w3 = generate_word(i * 13);
        let tag1 = generate_word(i % 10000);
        let tag2 = generate_word((i * 3) % 10000);
        input.push_str(&format!("add {} {} {} #{} #{}\n", w1, w2, w3, tag1, tag2));
    }
    for i in 0..searches {
        let term = &generate_word(i % 10000)[..3];
        input.push_str(&format!("search {}\n", term));
    }
    
    let start = Instant::now();
    let output = get_binary_with_timeout(10)
        .write_stdin(input)
        .output()
        .expect("Command timed out after 10 seconds");
    assert!(output.status.success(), "Performance test 5M failed or timed out");
    let elapsed = start.elapsed().as_secs_f64();
    println!("⏱️  test_performance_5m: {:.2}s", elapsed);
    assert!(elapsed < 10.0, "5M commands took {:.2}s, expected < 10s", elapsed);
}

#[test]
#[ignore] // Heavy benchmark - run with: cargo test --release -- --ignored
fn test_performance_10m() {
    let n = 10_000_000;
    let adds = n / 2;
    let searches = n / 2;
    
    let mut input = format!("{}\n", n);
    for i in 0..adds {
        let w1 = generate_word(i);
        let w2 = generate_word(i * 7);
        let w3 = generate_word(i * 13);
        let tag1 = generate_word(i % 20000);
        let tag2 = generate_word((i * 3) % 20000);
        input.push_str(&format!("add {} {} {} #{} #{}\n", w1, w2, w3, tag1, tag2));
    }
    for i in 0..searches {
        let term = &generate_word(i % 20000)[..3];
        input.push_str(&format!("search {}\n", term));
    }
    
    let start = Instant::now();
    let output = get_binary_with_timeout(10)
        .write_stdin(input)
        .output()
        .expect("Command timed out after 10 seconds");
    assert!(output.status.success(), "Performance test 10M failed or timed out");
    let elapsed = start.elapsed().as_secs_f64();
    println!("⏱️  test_performance_10m: {:.2}s", elapsed);
    assert!(elapsed < 10.0, "10M commands took {:.2}s, expected < 10s", elapsed);
}

#[test]
#[ignore] // Heavy benchmark - run with: cargo test --release -- --ignored
fn test_performance_15m() {
    let n = 15_000_000;
    let adds = n / 2;
    let searches = n / 2;
    
    let mut input = format!("{}\n", n);
    for i in 0..adds {
        let w1 = generate_word(i);
        let w2 = generate_word(i * 7);
        let w3 = generate_word(i * 13);
        let tag1 = generate_word(i % 30000);
        let tag2 = generate_word((i * 3) % 30000);
        input.push_str(&format!("add {} {} {} #{} #{}\n", w1, w2, w3, tag1, tag2));
    }
    for i in 0..searches {
        let term = &generate_word(i % 30000)[..3];
        input.push_str(&format!("search {}\n", term));
    }
    
    let start = Instant::now();
    let output = get_binary_with_timeout(10)
        .write_stdin(input)
        .output()
        .expect("Command timed out after 10 seconds");
    assert!(output.status.success(), "Performance test 15M failed or timed out");
    let elapsed = start.elapsed().as_secs_f64();
    println!("⏱️  test_performance_15m: {:.2}s", elapsed);
    assert!(elapsed < 10.0, "15M commands took {:.2}s, expected < 10s", elapsed);
}

#[test]
#[ignore] // Heavy benchmark - run with: cargo test --release -- --ignored
fn test_performance_20m() {
    let n = 20_000_000;
    let adds = n / 2;
    let searches = n / 2;
    
    let mut input = format!("{}\n", n);
    for i in 0..adds {
        let w1 = generate_word(i);
        let w2 = generate_word(i * 7);
        let w3 = generate_word(i * 13);
        let tag1 = generate_word(i % 50000);
        let tag2 = generate_word((i * 3) % 50000);
        input.push_str(&format!("add {} {} {} #{} #{}\n", w1, w2, w3, tag1, tag2));
    }
    for i in 0..searches {
        let term = &generate_word(i % 50000)[..3];
        input.push_str(&format!("search {}\n", term));
    }
    
    let start = Instant::now();
    let output = get_binary_with_timeout(15)
        .write_stdin(input)
        .output()
        .expect("Command timed out after 15 seconds");
    assert!(output.status.success(), "Performance test 20M failed or timed out");
    let elapsed = start.elapsed().as_secs_f64();
    println!("⏱️  test_performance_20m: {:.2}s", elapsed);
    assert!(elapsed < 15.0, "20M commands took {:.2}s, expected < 15s", elapsed);
}

#[test]
#[ignore] // Heavy benchmark - run with: cargo test --release -- --ignored
fn test_performance_25m() {
    let n = 25_000_000;
    let adds = n / 2;
    let searches = n / 2;
    
    let mut input = format!("{}\n", n);
    for i in 0..adds {
        let w1 = generate_word(i);
        let w2 = generate_word(i * 7);
        let w3 = generate_word(i * 13);
        let tag1 = generate_word(i % 60000);
        let tag2 = generate_word((i * 3) % 60000);
        input.push_str(&format!("add {} {} {} #{} #{}\n", w1, w2, w3, tag1, tag2));
    }
    for i in 0..searches {
        let term = &generate_word(i % 60000)[..3];
        input.push_str(&format!("search {}\n", term));
    }
    
    let start = Instant::now();
    let output = get_binary_with_timeout(15)
        .write_stdin(input)
        .output()
        .expect("Command timed out after 15 seconds");
    assert!(output.status.success(), "Performance test 25M failed or timed out");
    let elapsed = start.elapsed().as_secs_f64();
    println!("⏱️  test_performance_25m: {:.2}s", elapsed);
    assert!(elapsed < 15.0, "25M commands took {:.2}s, expected < 15s", elapsed);
}

#[test]
#[ignore] // Heavy benchmark - run with: cargo test --release -- --ignored
fn test_performance_30m() {
    let n = 30_000_000;
    let adds = n / 2;
    let searches = n / 2;
    
    let mut input = format!("{}\n", n);
    for i in 0..adds {
        let w1 = generate_word(i);
        let w2 = generate_word(i * 7);
        let w3 = generate_word(i * 13);
        let tag1 = generate_word(i % 70000);
        let tag2 = generate_word((i * 3) % 70000);
        input.push_str(&format!("add {} {} {} #{} #{}\n", w1, w2, w3, tag1, tag2));
    }
    for i in 0..searches {
        let term = &generate_word(i % 70000)[..3];
        input.push_str(&format!("search {}\n", term));
    }
    
    let start = Instant::now();
    let output = get_binary_with_timeout(15)
        .write_stdin(input)
        .output()
        .expect("Command timed out after 15 seconds");
    assert!(output.status.success(), "Performance test 30M failed or timed out");
    let elapsed = start.elapsed().as_secs_f64();
    println!("⏱️  test_performance_30m: {:.2}s", elapsed);
    assert!(elapsed < 15.0, "30M commands took {:.2}s, expected < 15s", elapsed);
}

#[test]
#[ignore] // Heavy benchmark - run with: cargo test --release -- --ignored
fn test_performance_35m() {
    let n = 35_000_000;
    let adds = n / 2;
    let searches = n / 2;
    
    let mut input = format!("{}\n", n);
    for i in 0..adds {
        let w1 = generate_word(i);
        let w2 = generate_word(i * 7);
        let w3 = generate_word(i * 13);
        let tag1 = generate_word(i % 80000);
        let tag2 = generate_word((i * 3) % 80000);
        input.push_str(&format!("add {} {} {} #{} #{}\n", w1, w2, w3, tag1, tag2));
    }
    for i in 0..searches {
        let term = &generate_word(i % 80000)[..3];
        input.push_str(&format!("search {}\n", term));
    }
    
    let start = Instant::now();
    let output = get_binary_with_timeout(15)
        .write_stdin(input)
        .output()
        .expect("Command timed out after 15 seconds");
    assert!(output.status.success(), "Performance test 35M failed or timed out");
    let elapsed = start.elapsed().as_secs_f64();
    println!("⏱️  test_performance_35m: {:.2}s", elapsed);
    assert!(elapsed < 15.0, "35M commands took {:.2}s, expected < 15s", elapsed);
}

#[test]
fn test_bishibosh_performance() {
    let input = fs::read_to_string("tests/fixtures/Bishibosh.in").unwrap();
    
    let start = Instant::now();
    let output = get_binary_with_timeout(10)
        .write_stdin(input)
        .output()
        .expect("Bishibosh fixture timed out after 10 seconds");
    assert!(output.status.success(), "Bishibosh fixture test failed or timed out");
    let elapsed = start.elapsed().as_secs_f64();
    println!("⏱️  test_bishibosh_performance: {:.2}s", elapsed);
    assert!(elapsed < 10.0, "Bishibosh fixture took {:.2}s, expected < 10s", elapsed);
}

#[test]
fn test_flamespike_the_crawler_performance() {
    let input = fs::read_to_string("tests/fixtures/Flamespike-The-Crawler.in").unwrap();
    
    let start = Instant::now();
    let output = get_binary_with_timeout(10)
        .write_stdin(input)
        .output()
        .expect("Flamespike-The-Crawler fixture timed out after 10 seconds");
    assert!(output.status.success(), "Flamespike-The-Crawler fixture test failed or timed out");
    let elapsed = start.elapsed().as_secs_f64();
    println!("⏱️  test_flamespike_the_crawler_performance: {:.2}s", elapsed);
    assert!(elapsed < 10.0, "Flamespike-The-Crawler fixture took {:.2}s, expected < 10s", elapsed);
}

