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

// Los tests siguientes cubren lo que syn 3 representa distinto que syn 2 (las
// guardas viven en el patron del brazo, el primer `|` de un closure cambio de
// nombre) y las construcciones que mas facil se rompen al subir de version. Los
// valores esperados son los que daba la version sobre syn 2.

#[test]
fn guards_count_once_per_arm_with_or_patterns_and_leading_vert() {
    let source = "fn f(o: Option<i32>, a: bool, b: bool) -> i32 {
        match o {
            Some(x) if x > 0 && a => 1,      // +1 guarda, +1 &&
            Some(0) | None if b || a => 2,   // +1 guarda, +1 ||
            | Some(1) if a => 3,             // +1 guarda
            Some(_) => 4,
            _ => 5,
        }                                    // +4 brazos
    }";
    assert_eq!(cyclomatic(source).0, 10);
}

#[test]
fn a_guard_is_scored_with_its_match_and_its_closures_apart() {
    let source = "fn f(o: Option<i32>, v: &[i32]) -> i32 {
        match o {
            Some(x) if v.iter().any(|y| if *y > x { true } else { false }) => 1,
            Some(x) if match x { 1 => true, _ => false } => 2,
            _ => 0,
        }
    }";
    let (function, closures) = cyclomatic(source);
    assert_eq!(function, 6);
    assert_eq!(
        closures,
        vec![ClosureScore {
            line: 3,
            cyclomatic: 2
        }]
    );
}

#[test]
fn closures_are_measured_apart_whatever_their_modifiers() {
    let source = "fn f() {
        let _ = || 1;
        let _ = move |x: &i32| if *x > 0 { 1 } else { 0 };
        let _ = for<'a> |x: &'a i32| -> &'a i32 { if *x > 0 { x } else { x } };
        let _ = async |x: i32| { if x > 0 { 1 } else { 0 } };
        let _ = async move |x: i32| x;
        let _ = const |x: i32| x;
        let _ = |x: i32| -> i32 { match x { 0 => 1, 1 => 2, _ => 3 } };
        let _ = |x: i32| move |y: i32| if x > y { x } else { y };
        let _ = move
            |x: i32|
            x;
    }";
    let (function, closures) = cyclomatic(source);
    assert_eq!(function, 1);
    let scores: Vec<(usize, u32)> = closures
        .iter()
        .map(|closure| (closure.line, closure.cyclomatic))
        .collect();
    let expected = [
        (2, 1),
        (3, 2),
        (4, 2),
        (5, 2),
        (6, 1),
        (7, 1),
        (8, 3),
        (9, 1),
        (9, 2),
        (11, 1),
    ];
    assert_eq!(scores, expected);
}

#[test]
fn let_chains_count_each_condition_and_operator() {
    let if_chain = "fn f(a: Option<i32>, b: Option<i32>) -> i32 {
        if let Some(x) = a && let Some(y) = b && x > y { x } // +1, +2 &&
        else if let Some(x) = a || b.is_some() { 0 }         // +1, +1 ||
        else { 1 }
    }";
    assert_eq!(cyclomatic(if_chain).0, 6);
    let while_chain = "fn f(mut v: Vec<i32>, a: bool) -> i32 {
        let mut r = 0;
        while let Some(x) = v.pop() && a && x > 0 { r += x; } // +1, +2 &&
        r
    }";
    assert_eq!(cyclomatic(while_chain).0, 4);
}

#[test]
fn every_let_else_counts_and_so_does_what_its_else_contains() {
    let source = "fn f(v: &[Option<i32>]) -> i32 {
        let Some(first) = v.first() else { return 0 };     // +1
        let Some(x) = first else {                         // +1
            if v.is_empty() { return 1 } else { return 2 } // +1
        };
        for item in v {                                    // +1
            let Some(y) = item else { continue };          // +1
            let _ = y;
        }
        *x
    }";
    assert_eq!(cyclomatic(source).0, 6);
}
