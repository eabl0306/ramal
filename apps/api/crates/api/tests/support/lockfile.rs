//! Grafo de dependencias leido de `Cargo.lock`, linea a linea y sin crate de
//! TOML, como en Gastonomo: el formato es estable y asi el test no arrastra
//! dependencias propias.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Mapa `paquete -> dependencias directas`.
pub type Graph = BTreeMap<String, BTreeSet<String>>;

/// Lee el `Cargo.lock` de la raiz del workspace.
pub fn graph() -> Graph {
    let path = super::repo_root().join("Cargo.lock");
    let lock =
        std::fs::read_to_string(path).expect("Cargo.lock should exist at the workspace root");
    parse(&lock)
}

/// Construye el grafo a partir del texto de un `Cargo.lock`.
///
/// Cada `[[package]]` abre un paquete; su lista `dependencies = [...]` trae
/// entradas como `"serde"` o `"serde 1.0.229"` cuando hay varias versiones, y
/// solo interesa el nombre.
pub fn parse(lock: &str) -> Graph {
    let mut graph = Graph::new();
    let mut current: Option<String> = None;
    let mut in_dependencies = false;
    for line in lock.lines().map(str::trim) {
        if line == "[[package]]" {
            current = None;
            in_dependencies = false;
        } else if let Some(name) = line.strip_prefix("name = ") {
            let name = unquote(name);
            graph.entry(name.clone()).or_default();
            current = Some(name);
        } else if line == "dependencies = [" {
            in_dependencies = true;
        } else if in_dependencies && line == "]" {
            in_dependencies = false;
        } else if in_dependencies {
            push_dependency(&mut graph, current.as_deref(), line);
        }
    }
    graph
}

fn push_dependency(graph: &mut Graph, owner: Option<&str>, raw: &str) {
    let Some(owner) = owner else { return };
    let entry = unquote(raw.trim_end_matches(','));
    if let Some(name) = entry.split_whitespace().next() {
        graph
            .entry(owner.to_owned())
            .or_default()
            .insert(name.to_owned());
    }
}

fn unquote(raw: &str) -> String {
    raw.trim().trim_matches('"').to_owned()
}

/// Todo lo que `root` alcanza, sin incluirse a si mismo.
pub fn transitive_closure(graph: &Graph, root: &str) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut queue = VecDeque::from([root.to_owned()]);
    while let Some(package) = queue.pop_front() {
        for dependency in graph.get(&package).into_iter().flatten() {
            if seen.insert(dependency.clone()) {
                queue.push_back(dependency.clone());
            }
        }
    }
    seen
}
