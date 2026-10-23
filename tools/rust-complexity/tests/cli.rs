//! El binario tal como lo llama `just rust-complexity`: el codigo de salida es el
//! gate, asi que se prueba con el proceso de verdad.

#![allow(
    unused_crate_dependencies,
    reason = "the lint runs per target and this test only spawns the binary"
)]
#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is the assertion"
)]

use std::path::Path;
use std::process::{Command, Output};

fn run(roots: &[&Path]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_git-rudder-rust-complexity"))
        .args(roots)
        .output()
        .unwrap()
}

#[test]
fn a_function_over_the_cognitive_limit_fails_with_its_location() {
    let dir = tempfile::tempdir().unwrap();
    let source = "fn deep(v: &[Vec<Vec<i32>>]) -> i32 {
        let mut r = 0;
        for a in v { for b in a { for c in b {
            if *c > 0 { if *c > 5 { r += 1; } else { r -= 1; } }
        } } }
        r
    }";
    std::fs::write(dir.path().join("deep.rs"), source).unwrap();
    let output = run(&[dir.path()]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("deep.rs:1  deep: cognitive complexity 16 (limit 15)"),
        "{stderr}"
    );
}

#[test]
fn code_within_the_limits_passes() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("ok.rs"),
        "fn ok(a: bool) -> u8 { if a { 1 } else { 2 } }",
    )
    .unwrap();
    let output = run(&[dir.path()]);
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 functions in 1 files"));
}

#[test]
fn a_missing_root_is_an_error_not_a_pass() {
    let dir = tempfile::tempdir().unwrap();
    let output = run(&[&dir.path().join("nope")]);
    assert_eq!(output.status.code(), Some(2));
}
