//! Complejidad cognitiva al estilo Sonar (G. Ann Campbell, "Cognitive
//! Complexity", 2017), la misma que mide `sonarjs/cognitive-complexity` en el
//! TypeScript:
//!
//! - `if`, `match`, `for`, `while`, `loop` y el `else` de un `let ... else` suman
//!   1 mas el nivel de anidamiento en el que estan.
//! - `else if` y `else` suman 1, sin anidamiento.
//! - Cada secuencia de operadores logicos iguales suma 1: `a && b && c` suma 1,
//!   `a && b || c` suma 2.
//! - `break` y `continue` con etiqueta suman 1.
//! - Los closures y los bloques `async` no suman, pero anidan lo que llevan dentro.
//! - `?` no suma: es propagar un error, como un `throw` en TypeScript.
//! - Las funciones anidadas se miden aparte.

use syn::visit::{self, Visit};
use syn::{Arm, BinOp, Block, Expr, ExprIf, Item, Local};

/// La complejidad cognitiva del cuerpo de una funcion.
pub fn score(block: &Block) -> u32 {
    let mut scorer = Cognitive::default();
    scorer.visit_block(block);
    scorer.score
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Logical {
    And,
    Or,
}

const fn logical(op: &BinOp) -> Option<Logical> {
    match op {
        BinOp::And(_) => Some(Logical::And),
        BinOp::Or(_) => Some(Logical::Or),
        _ => None,
    }
}

#[derive(Debug, Default)]
struct Cognitive {
    score: u32,
    nesting: u32,
    /// El operador de la secuencia logica en curso, si la hay.
    sequence: Option<Logical>,
}

impl Cognitive {
    /// Una estructura que corta el flujo: suma 1 mas su anidamiento.
    const fn structural(&mut self) {
        self.score += 1 + self.nesting;
    }

    fn nested(&mut self, visit: impl FnOnce(&mut Self)) {
        self.nesting += 1;
        visit(self);
        self.nesting -= 1;
    }

    fn if_chain(&mut self, node: &ExprIf, is_else_if: bool) {
        if is_else_if {
            self.score += 1;
        } else {
            self.structural();
        }
        self.visit_expr(&node.cond);
        self.nested(|scorer| scorer.visit_block(&node.then_branch));
        let Some((_, otherwise)) = &node.else_branch else {
            return;
        };
        if let Expr::If(next) = &**otherwise {
            self.if_chain(next, true);
        } else {
            self.score += 1;
            self.nested(|scorer| scorer.visit_expr(otherwise));
        }
    }
}

impl<'ast> Visit<'ast> for Cognitive {
    // Una secuencia logica solo sigue a traves de sus propios operadores y de los
    // parentesis; cualquier otra expresion empieza de cero.
    fn visit_expr(&mut self, node: &'ast Expr) {
        let continues = match node {
            Expr::Binary(binary) => logical(&binary.op).is_some(),
            Expr::Paren(_) => true,
            _ => false,
        };
        let outer = self.sequence;
        if !continues {
            self.sequence = None;
        }
        visit::visit_expr(self, node);
        self.sequence = outer;
    }

    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
        let Some(op) = logical(&node.op) else {
            visit::visit_expr_binary(self, node);
            return;
        };
        if self.sequence != Some(op) {
            self.score += 1;
        }
        let outer = self.sequence.replace(op);
        self.visit_expr(&node.left);
        self.visit_expr(&node.right);
        self.sequence = outer;
    }

    fn visit_expr_if(&mut self, node: &'ast ExprIf) {
        self.if_chain(node, false);
    }

    // La guarda de cada brazo (un Pat::Guard desde syn 3) se recorre con su
    // patron, asi que paga el anidamiento del match igual que el cuerpo.
    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        self.structural();
        self.visit_expr(&node.expr);
        self.nested(|scorer| node.arms.iter().for_each(|arm: &Arm| scorer.visit_arm(arm)));
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        self.structural();
        self.visit_expr(&node.expr);
        self.nested(|scorer| scorer.visit_block(&node.body));
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        self.structural();
        self.visit_expr(&node.cond);
        self.nested(|scorer| scorer.visit_block(&node.body));
    }

    fn visit_expr_loop(&mut self, node: &'ast syn::ExprLoop) {
        self.structural();
        self.nested(|scorer| scorer.visit_block(&node.body));
    }

    fn visit_expr_closure(&mut self, node: &'ast syn::ExprClosure) {
        self.nested(|scorer| scorer.visit_expr(&node.body));
    }

    fn visit_expr_async(&mut self, node: &'ast syn::ExprAsync) {
        self.nested(|scorer| scorer.visit_block(&node.block));
    }

    fn visit_expr_break(&mut self, node: &'ast syn::ExprBreak) {
        if node.label.is_some() {
            self.score += 1;
        }
        visit::visit_expr_break(self, node);
    }

    fn visit_expr_continue(&mut self, node: &'ast syn::ExprContinue) {
        if node.label.is_some() {
            self.score += 1;
        }
    }

    fn visit_local(&mut self, node: &'ast Local) {
        let Some(init) = &node.init else {
            return;
        };
        self.visit_expr(&init.expr);
        if let Some((_, diverge)) = &init.diverge {
            self.structural();
            self.nested(|scorer| scorer.visit_expr(diverge));
        }
    }

    // Las funciones, impl y modulos anidados se miden aparte.
    fn visit_item(&mut self, _node: &'ast Item) {}
}

#[cfg(test)]
mod tests;
