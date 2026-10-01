//! Recorre un fichero y puntua cada funcion: libres, metodos de `impl`, metodos
//! con cuerpo de un `trait`, las anidadas dentro de otras y los closures (estos
//! solo para la ciclomatica, porque la cognitiva los suma a su funcion).

use syn::visit::{self, Visit};
use syn::{Block, File, Signature, Type};

use crate::{cognitive, cyclomatic};

/// Lo que mide cada funcion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionScore {
    /// `funcion`, `Tipo::metodo` o `Tipo::metodo::{closure}`.
    pub name: String,
    /// Linea donde empieza.
    pub line: usize,
    /// Complejidad cognitiva (0 en un closure: cuenta en su funcion).
    pub cognitive: u32,
    /// Complejidad ciclomatica.
    pub cyclomatic: u32,
}

/// Las puntuaciones de todas las funciones de un fichero, en orden de aparicion.
pub fn collect(file: &File) -> Vec<FunctionScore> {
    let mut finder = Finder::default();
    finder.visit_file(file);
    finder.scores
}

#[derive(Debug, Default)]
struct Finder {
    scores: Vec<FunctionScore>,
    /// El tipo o trait del `impl` en el que se esta, para nombrar los metodos.
    owners: Vec<String>,
}

impl Finder {
    fn score(&mut self, signature: &Signature, block: &Block) {
        let name = self.owners.last().map_or_else(
            || signature.ident.to_string(),
            |owner| format!("{owner}::{}", signature.ident),
        );
        let (cyclomatic, closures) = cyclomatic::score(block);
        self.scores.push(FunctionScore {
            name: name.clone(),
            line: signature.ident.span().start().line,
            cognitive: cognitive::score(block),
            cyclomatic,
        });
        self.scores
            .extend(closures.into_iter().map(|closure| FunctionScore {
                name: format!("{name}::{{closure}}"),
                line: closure.line,
                cognitive: 0,
                cyclomatic: closure.cyclomatic,
            }));
    }

    fn within(&mut self, owner: String, visit: impl FnOnce(&mut Self)) {
        self.owners.push(owner);
        visit(self);
        self.owners.pop();
    }
}

fn type_name(ty: &Type) -> String {
    match ty {
        Type::Path(path) => path
            .path
            .segments
            .last()
            .map_or_else(|| "_".to_owned(), |segment| segment.ident.to_string()),
        Type::Reference(reference) => type_name(&reference.elem),
        _ => "_".to_owned(),
    }
}

impl<'ast> Visit<'ast> for Finder {
    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        self.within(type_name(&node.self_ty), |finder| {
            visit::visit_item_impl(finder, node);
        });
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        self.within(node.ident.to_string(), |finder| {
            visit::visit_item_trait(finder, node);
        });
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.score(&node.sig, &node.block);
        visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.score(&node.sig, &node.block);
        visit::visit_impl_item_fn(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        if let Some(block) = &node.default {
            self.score(&node.sig, block);
        }
        visit::visit_trait_item_fn(self, node);
    }
}
