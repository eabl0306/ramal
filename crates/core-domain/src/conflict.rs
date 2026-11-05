//! Ficheros en conflicto: el texto con marcadores partido en segmentos.
//!
//! El resolvedor necesita los bloques exactos. Se reconoce el marcador del
//! ancestro comun (`|||||||`, de diff3 y zdiff3), que el prototipo ignoraba, y
//! una linea `=======` solo cuenta dentro de un bloque abierto: fuera es un
//! subrayado de Markdown o de RST legitimo (B-21). Es el mismo parser que el de
//! `@git-rudder/domain` en TypeScript, para que motor e interfaz lean igual.

use thiserror::Error;

use crate::status::ConflictCode;

/// Un trozo del fichero en conflicto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictSegment {
    /// Lineas iguales en los dos lados.
    Context {
        /// Lineas sin el salto final.
        lines: Vec<String>,
    },
    /// Un bloque en conflicto.
    Hunk {
        /// Lineas de nuestro lado.
        ours: Vec<String>,
        /// Lineas del ancestro comun, si git escribio marcadores diff3 o zdiff3.
        base: Option<Vec<String>>,
        /// Lineas de su lado.
        theirs: Vec<String>,
    },
}

/// Lo que va detras de cada marcador (`<<<<<<< HEAD`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConflictLabels {
    /// Etiqueta de nuestro lado.
    pub ours: String,
    /// Etiqueta del ancestro, si hay marcador base.
    pub base: Option<String>,
    /// Etiqueta de su lado.
    pub theirs: String,
}

/// Fichero en conflicto ya partido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedConflict {
    /// Segmentos en orden.
    pub segments: Vec<ConflictSegment>,
    /// Etiquetas del primer bloque, si hay alguno.
    pub labels: Option<ConflictLabels>,
}

/// El texto no tiene marcadores bien formados.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum MarkerError {
    /// Un bloque abierto que no se cierra.
    #[error("conflict block left open at line {line}")]
    Unterminated {
        /// Ultima linea del fichero, empezando en 1.
        line: usize,
    },
    /// Un marcador donde no toca.
    #[error("unexpected conflict marker at line {line}")]
    UnexpectedMarker {
        /// Linea del marcador, empezando en 1.
        line: usize,
    },
}

/// Lo que se sabe de un fichero en conflicto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictFile {
    /// Ruta relativa a la raiz.
    pub path: String,
    /// Forma del conflicto (`UU`, `DU`...).
    pub code: ConflictCode,
    /// Alguno de los lados es binario: se elige un lado entero.
    pub binary: bool,
    /// Segmentos si el fichero es de texto y los dos lados existen.
    pub content: Option<ParsedConflict>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MarkerKind {
    Start,
    Base,
    Separator,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Context,
    Ours,
    Base,
    Theirs,
}

/// Marcador de la linea y su etiqueta. Un `\r` final (CRLF) no cuenta.
fn marker_of(line: &str) -> Option<(MarkerKind, &str)> {
    let clean = line.strip_suffix('\r').unwrap_or(line);
    let (kind, rest) = [
        ("<<<<<<<", MarkerKind::Start),
        ("|||||||", MarkerKind::Base),
        ("=======", MarkerKind::Separator),
        (">>>>>>>", MarkerKind::End),
    ]
    .into_iter()
    .find_map(|(prefix, kind)| clean.strip_prefix(prefix).map(|rest| (kind, rest)))?;
    match (kind, rest) {
        (MarkerKind::Separator, "") => Some((kind, "")),
        (MarkerKind::Separator, _) => None,
        (_, "") => Some((kind, "")),
        (_, label) => label.strip_prefix(' ').map(|label| (kind, label)),
    }
}

#[derive(Debug, Default)]
struct Parser {
    state: Option<State>,
    context: Vec<String>,
    ours: Vec<String>,
    base: Option<Vec<String>>,
    theirs: Vec<String>,
    pending: ConflictLabels,
    segments: Vec<ConflictSegment>,
    labels: Option<ConflictLabels>,
}

impl Parser {
    fn state(&self) -> State {
        self.state.unwrap_or(State::Context)
    }

    /// `false` si la linea es un marcador fuera de sitio.
    fn feed(&mut self, line: &str) -> bool {
        match marker_of(line) {
            Some((MarkerKind::Separator | MarkerKind::Base, _))
                if self.state() == State::Context =>
            {
                self.append(line);
                true
            }
            Some((kind, label)) => self.apply(kind, label),
            None => {
                self.append(line);
                true
            }
        }
    }

    fn append(&mut self, line: &str) {
        let line = line.to_owned();
        match self.state() {
            State::Context => self.context.push(line),
            State::Ours => self.ours.push(line),
            State::Base => self.base.get_or_insert_with(Vec::new).push(line),
            State::Theirs => self.theirs.push(line),
        }
    }

    fn apply(&mut self, kind: MarkerKind, label: &str) -> bool {
        match (kind, self.state()) {
            (MarkerKind::Start, State::Context) => {
                self.flush_context();
                self.pending = ConflictLabels {
                    ours: label.to_owned(),
                    ..ConflictLabels::default()
                };
                self.state = Some(State::Ours);
            }
            (MarkerKind::Base, State::Ours) => {
                self.base = Some(Vec::new());
                self.pending.base = Some(label.to_owned());
                self.state = Some(State::Base);
            }
            (MarkerKind::Separator, State::Ours | State::Base) => {
                self.state = Some(State::Theirs);
            }
            (MarkerKind::End, State::Theirs) => self.close_hunk(label),
            _ => return false,
        }
        true
    }

    fn close_hunk(&mut self, label: &str) {
        self.segments.push(ConflictSegment::Hunk {
            ours: std::mem::take(&mut self.ours),
            base: self.base.take(),
            theirs: std::mem::take(&mut self.theirs),
        });
        let mut labels = std::mem::take(&mut self.pending);
        label.clone_into(&mut labels.theirs);
        self.labels.get_or_insert(labels);
        self.state = Some(State::Context);
    }

    fn flush_context(&mut self) {
        if !self.context.is_empty() {
            let lines = std::mem::take(&mut self.context);
            self.segments.push(ConflictSegment::Context { lines });
        }
    }
}

/// Parte el texto con marcadores en segmentos.
///
/// Las lineas se separan por `\n` y conservan un `\r` final si lo tenian, asi el
/// texto se puede reconstruir tal cual.
///
/// # Errors
///
/// [`MarkerError`] si un bloque queda abierto o un marcador aparece donde no toca.
pub fn parse_conflict_markers(text: &str) -> Result<ParsedConflict, MarkerError> {
    let mut parser = Parser::default();
    let lines: Vec<&str> = text.split('\n').collect();
    for (index, line) in lines.iter().enumerate() {
        if !parser.feed(line) {
            return Err(MarkerError::UnexpectedMarker { line: index + 1 });
        }
    }
    if parser.state() != State::Context {
        return Err(MarkerError::Unterminated { line: lines.len() });
    }
    parser.flush_context();
    Ok(ParsedConflict {
        segments: parser.segments,
        labels: parser.labels,
    })
}

#[cfg(test)]
mod tests;
