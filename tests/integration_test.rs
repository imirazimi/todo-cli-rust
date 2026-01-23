use assert_cmd::Command;
use std::fs;
use std::time::Instant;

fn get_binary() -> Command {
    Command::cargo_bin("application").unwrap()
}

fn normalize_output(output: &str) -> Vec<String> {
    output
        .lines()
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
    assert!(actual.contains("buy milk") && actual.contains("#groceries"));
}

#[test]
fn test_done_filtering() {
    let actual = run_and_get_output("4\nadd \"taskone\" #work\nadd \"tasktwo\" #work\ndone 0\nsearch #work\n");
    assert!(actual.contains("tasktwo") && !actual.contains("\"taskone\""));
}

#[test]
fn test_subsequence_matching() {
    let actual = run_and_get_output("2\nadd \"bread\" #food\nsearch a\n");
    assert!(actual.contains("bread"));
}

#[test]
fn test_empty_search() {
    let actual = run_and_get_output("3\nadd \"itemone\" #tagone\nadd \"itemtwo\" #tagtwo\nsearch\n");
    assert!(actual.contains("itemone") && actual.contains("itemtwo"));
}

#[test]
fn test_case_insensitive_matching() {
    let actual = run_and_get_output("2\nadd \"BREAD\" #FOOD\nsearch bread\n");
    assert!(actual.to_lowercase().contains("bread"));
}

#[test]
fn test_multiple_words_search() {
    let actual = run_and_get_output("3\nadd \"buy bread\" #groceries\nadd \"buy milk\" #groceries\nsearch buy bread\n");
    assert!(actual.contains("\"buy bread\"") && !actual.contains("\"buy milk\""));
}

#[test]
fn test_mixed_query() {
    let actual = run_and_get_output("3\nadd \"buy bread\" #groceries\nadd \"buy milk\" #groceries\nsearch buy #groceries\n");
    assert!(actual.contains("buy bread") && actual.contains("buy milk"));
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
    
    let start = Instant::now();
    let _ = run_and_get_output(&input);
    assert!(start.elapsed().as_secs_f64() < 10.0, "Performance test exceeded 10s limit");
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
    
    let start = Instant::now();
    let _ = run_and_get_output(&input);
    assert!(start.elapsed().as_secs_f64() < 10.0, "Performance test exceeded 10s limit");
}
