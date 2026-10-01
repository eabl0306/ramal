#![allow(
    clippy::expect_used,
    reason = "tests fail loudly on purpose; a panic here is the assertion"
)]

use super::{ClosureScore, score};

/// La ciclomatica de una funcion escrita como texto, y la de sus closures.
fn cyclomatic(source: &str) -> (u32, Vec<ClosureScore>) {
    let function: syn::ItemFn = syn::parse_str(source).expect("the fixture is valid Rust");
    score(&function.block)
}

#[test]
fn a_function_without_decisions_scores_one() {
    assert_eq!(cyclomatic("fn f() -> i32 { 1 }").0, 1);
}

#[test]
fn fourteen_sequential_ifs_score_fifteen() {
    let body = (0..14).fold(String::new(), |body, value| {
        body + &format!("if x == {value} {{ r += 1; }}")
    });
    let source = format!("fn f(x: usize) -> usize {{ let mut r = 0; {body} r }}");
    assert_eq!(cyclomatic(&source).0, 15);
}

#[test]
fn every_branch_operator_and_loop_counts() {
    let source = "fn f(v: &[i32], a: bool, b: bool) -> i32 {
        let mut r = 0;
        for x in v { r += x; }               // +1
        while r > 10 { r -= 1; }             // +1
        loop { break; }                      // +1
        if a && b || r > 0 { r += 1; }       // +1, +1, +1
        else if a { r -= 1; }                // +1
        let Some(first) = v.first() else { return 0 }; // +1
        r + first
    }";
    assert_eq!(cyclomatic(source).0, 9);
}

#[test]
fn a_match_adds_its_arms_but_one_and_each_guard() {
    let source = "fn f(o: Option<i32>) -> i32 {
        match o {
            Some(x) if x > 0 => 1,
            Some(_) => 2,
            None => 0,
        }
    }";
    assert_eq!(cyclomatic(source).0, 4);
}

#[test]
fn the_question_mark_adds_nothing() {
    let source = "fn f(s: &str) -> Result<i32, std::num::ParseIntError> {
        let a: i32 = s.parse()?;
        let b: i32 = s.trim().parse()?;
        Ok(a + b)
    }";
    assert_eq!(cyclomatic(source).0, 1);
}

#[test]
fn closures_and_nested_functions_are_measured_apart() {
    let source = "fn f(v: &[i32]) -> i32 {
        fn inner(a: bool) -> i32 { if a { 1 } else { 2 } }
        v.iter()
            .map(|x| if *x > 0 { 1 } else { 0 })
            .sum::<i32>() + inner(true)
    }";
    let (function, closures) = cyclomatic(source);
    assert_eq!(function, 1);
    assert_eq!(
        closures,
        vec![ClosureScore {
            line: 4,
            cyclomatic: 2
        }]
    );
}
