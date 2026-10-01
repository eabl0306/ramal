#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests fail loudly on purpose; a panic here is the assertion"
)]

use std::path::{Path, PathBuf};

use super::{COGNITIVE_LIMIT, CYCLOMATIC_LIMIT, CheckError, analyze_source, check};

fn repo_file(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    std::fs::read_to_string(path).expect("the repo file should exist")
}

// Los limites se escriben una vez por herramienta; este test vigila que los de
// Rust no se separen de clippy.toml ni de los de ESLint.
#[test]
fn the_limits_match_clippy_and_eslint() {
    let clippy = repo_file("clippy.toml");
    assert!(clippy.contains(&format!(
        "cognitive-complexity-threshold = {COGNITIVE_LIMIT}\n"
    )));
    let eslint = repo_file("packages/eslint-config/quality.js");
    assert!(eslint.contains(&format!("cognitiveComplexity: {COGNITIVE_LIMIT},")));
    assert!(eslint.contains(&format!("  complexity: {CYCLOMATIC_LIMIT},")));
}

#[test]
fn functions_are_named_by_their_impl_or_trait() {
    let source = "
struct Repo;
impl Repo { fn open(&self) {} }
trait Reader { fn read(&self) -> i32 { if true { 1 } else { 2 } } fn name(&self); }
fn free() { let _ = |x: i32| x; }
";
    let names: Vec<(String, usize)> = analyze_source(source)
        .unwrap()
        .into_iter()
        .map(|score| (score.name, score.line))
        .collect();
    let expected = [
        ("Repo::open", 3),
        ("Reader::read", 4),
        ("free", 5),
        ("free::{closure}", 5),
    ]
    .map(|(name, line)| (name.to_owned(), line));
    assert_eq!(names, expected);
}

fn write(dir: &Path, name: &str, source: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, source).unwrap();
    path
}

#[test]
fn check_reports_only_the_functions_over_a_limit() {
    let dir = tempfile::tempdir().unwrap();
    let ifs = (0..11).fold(String::new(), |body, value| {
        body + &format!("if x == {value} {{ r += 1; }}")
    });
    write(
        dir.path(),
        "a.rs",
        &format!("fn branchy(x: u8) -> u8 {{ let mut r = 0; {ifs} r }}"),
    );
    write(dir.path(), "b.rs", "fn calm() -> u8 { 1 }");
    let report = check(&[dir.path().to_path_buf()]).unwrap();
    assert_eq!((report.files, report.functions), (2, 2));
    let names: Vec<&str> = report
        .violations
        .iter()
        .map(|v| v.score.name.as_str())
        .collect();
    assert_eq!(names, ["branchy"]);
    assert_eq!(
        report.violations.first().map(|v| v.problem.as_str()),
        Some("cyclomatic complexity 12 (limit 10)")
    );
}

#[test]
fn check_fails_instead_of_skipping_what_it_cannot_measure() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing");
    assert!(matches!(check(&[missing]), Err(CheckError::MissingRoot(_))));
    assert!(matches!(
        check(&[dir.path().to_path_buf()]),
        Err(CheckError::NoFiles)
    ));
    assert!(matches!(check(&[]), Err(CheckError::NoFiles)));
    write(dir.path(), "broken.rs", "fn broken( {");
    let error = check(&[dir.path().to_path_buf()]).unwrap_err();
    assert!(
        matches!(error, CheckError::Parse { line: 1, .. }),
        "{error}"
    );
}

#[test]
fn ignored_folders_are_not_measured() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("target/debug")).unwrap();
    write(&dir.path().join("target/debug"), "built.rs", "fn broken( {");
    write(dir.path(), "lib.rs", "fn ok() {}");
    assert_eq!(check(&[dir.path().to_path_buf()]).unwrap().files, 1);
}
