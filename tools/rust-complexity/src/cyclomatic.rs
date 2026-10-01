//! Complejidad ciclomatica, contada como la regla `complexity` de ESLint en el
//! TypeScript: 1 por la funcion y 1 por cada decision.
//!
//! - Cada `if` (tambien `if let` y `else if`), `for`, `while`, `loop` y el `else`
//!   de un `let ... else`.
//! - Cada operador `&&` y `||`.
//! - Un `match` de n brazos suma n - 1 (un `switch` con `default` en ESLint), mas
//!   1 por cada guarda `if`.
//! - `?` no suma: es propagar un error, como un `throw` que ESLint tampoco cuenta.
//! - Cada closure es una funcion aparte, como una funcion flecha en ESLint; las
//!   funciones anidadas se miden aparte.

use syn::visit::{self, Visit};
use syn::{BinOp, Block, Expr, Item, Local};

/// La ciclomatica de un closure, medido aparte de la funcion que lo contiene.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClosureScore {
    /// Linea donde empieza.
    pub line: usize,
    /// Su complejidad ciclomatica.
    pub cyclomatic: u32,
}

/// La ciclomatica del cuerpo de una funcion y la de cada closure que contiene.
pub fn score(block: &Block) -> (u32, Vec<ClosureScore>) {
    let mut scorer = Cyclomatic::new();
    scorer.visit_block(block);
    (scorer.score, scorer.closures)
}

fn score_expr(expr: &Expr) -> (u32, Vec<ClosureScore>) {
    let mut scorer = Cyclomatic::new();
    scorer.visit_expr(expr);
    (scorer.score, scorer.closures)
}

#[derive(Debug)]
struct Cyclomatic {
    score: u32,
    closures: Vec<ClosureScore>,
}

impl Cyclomatic {
    const fn new() -> Self {
        Self {
            score: 1,
            closures: Vec::new(),
        }
    }

    fn add(&mut self, decisions: usize) {
        self.score = self
            .score
            .saturating_add(u32::try_from(decisions).unwrap_or(u32::MAX));
    }
}

impl<'ast> Visit<'ast> for Cyclomatic {
    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        self.add(1);
        visit::visit_expr_if(self, node);
    }

    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
        if matches!(node.op, BinOp::And(_) | BinOp::Or(_)) {
            self.add(1);
        }
        visit::visit_expr_binary(self, node);
    }

    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        let guards = node.arms.iter().filter(|arm| arm.guard.is_some()).count();
        self.add(node.arms.len().saturating_sub(1) + guards);
        visit::visit_expr_match(self, node);
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        self.add(1);
        visit::visit_expr_for_loop(self, node);
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        self.add(1);
        visit::visit_expr_while(self, node);
    }

    fn visit_expr_loop(&mut self, node: &'ast syn::ExprLoop) {
        self.add(1);
        visit::visit_expr_loop(self, node);
    }

    fn visit_local(&mut self, node: &'ast Local) {
        if node
            .init
            .as_ref()
            .is_some_and(|init| init.diverge.is_some())
        {
            self.add(1);
        }
        visit::visit_local(self, node);
    }

    fn visit_expr_closure(&mut self, node: &'ast syn::ExprClosure) {
        let (cyclomatic, nested) = score_expr(&node.body);
        let line = node.or1_token.span.start().line;
        self.closures.push(ClosureScore { line, cyclomatic });
        self.closures.extend(nested);
    }

    // Las funciones, impl y modulos anidados se miden aparte.
    fn visit_item(&mut self, _node: &'ast Item) {}
}

#[cfg(test)]
mod tests;
