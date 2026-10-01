#![allow(
    clippy::expect_used,
    reason = "tests fail loudly on purpose; a panic here is the assertion"
)]

use super::score;

/// La cognitiva de una funcion escrita como texto.
fn cognitive(source: &str) -> u32 {
    let function: syn::ItemFn = syn::parse_str(source).expect("the fixture is valid Rust");
    score(&function.block)
}

fn sequential_ifs(count: usize) -> String {
    let body = (0..count).fold(String::new(), |body, value| {
        body + &format!("if x == {value} {{ r += 1; }}")
    });
    format!("fn f(x: usize) -> usize {{ let mut r = 0; {body} r }}")
}

#[test]
fn each_flat_if_adds_one() {
    assert_eq!(cognitive(&sequential_ifs(14)), 14);
    assert_eq!(cognitive(&sequential_ifs(16)), 16);
}

// La funcion que clippy::cognitive_complexity deja pasar con el umbral en 15:
// clippy no penaliza el anidamiento ni cuenta los operadores logicos.
#[test]
fn nesting_and_boolean_operators_add_up_like_sonar() {
    let source = "fn f(v: &[i32], a: bool, b: bool, c: bool) -> i32 {
        let mut r = 0;
        for x in v {                      // +1
            for y in v {                  // +2
                if *x > 3 && a || b {     // +3, && y || +2
                    r += y;
                } else if *y > 2 {        // +1
                    r -= 1;
                } else {                  // +1
                    r += 2;
                }
                while r > 100 {           // +3
                    r -= 10;
                }
                if c || a {               // +3, || +1
                    r += 1;
                }
            }
        }
        r
    }";
    assert_eq!(cognitive(source), 17);
}

#[test]
fn a_loop_counts_and_nests_what_it_contains() {
    let source = "fn f(mut r: i32) -> i32 { loop { if r > 10 { break; } r += 1; } r }";
    assert_eq!(cognitive(source), 3);
}

#[test]
fn labeled_jumps_add_one_each() {
    let source = "fn f(v: &[Vec<i32>]) -> i32 {
        let mut r = 0;
        'outer: for row in v {             // +1
            for x in row {                 // +2
                if *x < 0 { continue 'outer; } // +3, +1
                if *x > 9 { break 'outer; }    // +3, +1
                r += x;
            }
        }
        r
    }";
    assert_eq!(cognitive(source), 11);
}

#[test]
fn else_if_chains_do_not_pay_for_nesting() {
    let source = "fn f(x: i32) -> i32 {
        for _ in 0..1 {                        // +1
            if x == 1 { return 1; }            // +2
            else if x == 2 { return 2; }       // +1
            else if x == 3 { return 3; }       // +1
            else { return 4; }                 // +1
        }
        0
    }";
    assert_eq!(cognitive(source), 6);
}

#[test]
fn sequences_of_like_operators_count_once() {
    assert_eq!(
        cognitive("fn f(a: bool, b: bool, c: bool) -> bool { a && b && c }"),
        1
    );
    assert_eq!(
        cognitive("fn f(a: bool, b: bool, c: bool) -> bool { a && b || c }"),
        2
    );
    assert_eq!(
        cognitive("fn f(a: bool, b: bool, c: bool) -> bool { a || b && c }"),
        2
    );
    assert_eq!(
        cognitive("fn f(a: bool, b: bool, c: bool) -> bool { (a || b) && c }"),
        2
    );
    assert_eq!(
        cognitive("fn f(a: bool, b: bool) -> bool { g(a && b) && a }"),
        2
    );
}

#[test]
fn closures_nest_what_they_contain() {
    let source = "fn f(v: &[i32]) -> i32 {
        if v.is_empty() {                                             // +1
            return v.iter().map(|x| if *x > 0 { 1 } else { 0 }).sum(); // +3, +1
        }
        0
    }";
    assert_eq!(cognitive(source), 5);
}

#[test]
fn match_and_let_else_are_structures() {
    let source = "fn f(o: Option<i32>, v: &[Option<i32>]) -> i32 {
        match o {                                 // +1
            Some(x) if x > 0 => 1,
            Some(_) => { if v.is_empty() { 2 } else { 3 } } // +2, +1
            None => 0,
        };
        for item in v {                           // +1
            let Some(x) = item else { continue }; // +2
            let _ = x;
        }
        0
    }";
    assert_eq!(cognitive(source), 7);
}

#[test]
fn the_question_mark_and_nested_functions_add_nothing() {
    let source = "fn f(s: &str) -> Result<i32, std::num::ParseIntError> {
        fn inner(a: bool) -> i32 { if a { 1 } else { 2 } }
        let a: i32 = s.parse()?;
        let b: i32 = s.trim().parse()?;
        Ok(a + b + inner(true))
    }";
    assert_eq!(cognitive(source), 0);
}

// Los tests siguientes cubren lo que syn 3 representa distinto que syn 2 (las
// guardas viven en el patron del brazo, los closures cambiaron de campos) y las
// construcciones que mas facil se rompen al subir de version. Los valores
// esperados son los que daba la version sobre syn 2.

#[test]
fn guards_are_scored_inside_the_nesting_of_their_match() {
    let operators = "fn f(o: Option<i32>, a: bool, b: bool) -> i32 {
        match o {                          // +1
            Some(x) if x > 0 && a => 1,    // && +1
            Some(0) | None if b || a => 2, // || +1
            | Some(1) if a => 3,
            _ => 4,
        }
    }";
    assert_eq!(cognitive(operators), 3);
    let nested = "fn f(o: Option<i32>, v: &[i32]) -> i32 {
        match o {                                                                // +1
            Some(x) if v.iter().any(|y| if *y > x { true } else { false }) => 1, // +3, +1
            Some(x) if match x { 1 => true, _ => false } => 2,                   // +2
            _ => 0,
        }
    }";
    assert_eq!(cognitive(nested), 7);
}

#[test]
fn closures_nest_whatever_their_modifiers() {
    let source = "fn f() {
        let _ = || 1;
        let _ = move |x: &i32| if *x > 0 { 1 } else { 0 };                      // +2, +1
        let _ = for<'a> |x: &'a i32| -> &'a i32 { if *x > 0 { x } else { x } }; // +2, +1
        let _ = async |x: i32| { if x > 0 { 1 } else { 0 } };                   // +2, +1
        let _ = async move |x: i32| x;
        let _ = const |x: i32| x;
        let _ = |x: i32| -> i32 { match x { 0 => 1, 1 => 2, _ => 3 } };         // +2
        let _ = |x: i32| move |y: i32| if x > y { x } else { y };               // +3, +1
    }";
    assert_eq!(cognitive(source), 15);
}

#[test]
fn let_chains_are_one_sequence_of_operators() {
    let if_chain = "fn f(a: Option<i32>, b: Option<i32>) -> i32 {
        if let Some(x) = a && let Some(y) = b && x > y { x } // +1, && +1
        else if let Some(x) = a || b.is_some() { 0 }         // +1, || +1
        else { 1 }                                           // +1
    }";
    assert_eq!(cognitive(if_chain), 5);
    let while_chain = "fn f(mut v: Vec<i32>, a: bool) -> i32 {
        let mut r = 0;
        while let Some(x) = v.pop() && a && x > 0 { r += x; } // +1, && +1
        r
    }";
    assert_eq!(cognitive(while_chain), 2);
}

#[test]
fn a_let_else_nests_what_its_else_contains() {
    let source = "fn f(v: &[Option<i32>]) -> i32 {
        let Some(first) = v.first() else { return 0 };     // +1
        let Some(x) = first else {                         // +1
            if v.is_empty() { return 1 } else { return 2 } // +2, +1
        };
        for item in v {                                    // +1
            let Some(y) = item else { continue };          // +2
            let _ = y;
        }
        *x
    }";
    assert_eq!(cognitive(source), 8);
}
