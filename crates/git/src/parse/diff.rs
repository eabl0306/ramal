//! Diff unificado de git (`git diff`, `git show`), con o sin
//! `--word-diff=porcelain`.
//!
//! Se respeta el orden de git, que nunca intercala: dentro de un cambio, primero
//! las lineas quitadas y despues las anadidas (B-17). Las rutas no se sacan de
//! `diff --git a/x b/y`, que es ambiguo con espacios: quien llama sabe que fichero
//! pidio, y la ruta de origen de un renombrado sale de `rename from`.

use git_rudder_core_domain::diff::{DiffLine, FileStatus, Hunk, LineKind, WordSegment};

use super::quote::unquote;

/// Un fichero del diff, sin interpretar todavia (binario, LFS, imagen).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileSection {
    /// La linea `diff --git ...`.
    pub header: String,
    /// Ruta de origen de un renombrado o una copia.
    pub old_path: Option<String>,
    /// Ruta de destino de un renombrado (`rename to`).
    pub new_path: Option<String>,
    /// Que le paso.
    pub status: FileStatus,
    /// git dijo que es binario.
    pub binary: bool,
    /// Hunks.
    pub hunks: Vec<Hunk>,
}

impl FileSection {
    fn new(header: &str) -> Self {
        Self {
            header: header.to_owned(),
            old_path: None,
            new_path: None,
            status: FileStatus::Modified,
            binary: false,
            hunks: Vec::new(),
        }
    }

    /// La seccion es la de `path` (en el lado nuevo o en el viejo).
    #[must_use]
    pub fn is_about(&self, path: &str) -> bool {
        self.new_path.as_deref() == Some(path)
            || self.old_path.as_deref() == Some(path)
            || self.header.ends_with(&format!(" b/{path}"))
            || self.header.ends_with(&format!("b/{path}\""))
    }
}

/// Lee todos los ficheros del diff. `words` dice si se pidio
/// `--word-diff=porcelain`.
#[must_use]
pub fn parse_diff(text: &str, words: bool) -> Vec<FileSection> {
    let mut sections: Vec<FileSection> = Vec::new();
    let mut hunk: Option<HunkBuilder> = None;
    // Se parte por `\n` y no con `lines()`, que tambien se come el `\r` de un
    // fichero CRLF: la diferencia entre `a` y `a\r` tiene que verse.
    let body = text.strip_suffix('\n').unwrap_or(text);
    for line in body.split('\n') {
        if line.is_empty() && hunk.is_none() {
            continue;
        }
        if let Some(header) = line.strip_prefix("diff --git ") {
            finish(&mut sections, &mut hunk);
            sections.push(FileSection::new(header));
            continue;
        }
        let Some(section) = sections.last_mut() else {
            continue;
        };
        if let Some(start) = parse_hunk_header(line) {
            if let Some(done) = hunk.take() {
                section.hunks.push(done.finish());
            }
            hunk = Some(HunkBuilder::new(start, words));
            continue;
        }
        match hunk.as_mut() {
            Some(builder) => builder.feed(line),
            None => read_meta(section, line),
        }
    }
    finish(&mut sections, &mut hunk);
    sections
}

fn finish(sections: &mut [FileSection], hunk: &mut Option<HunkBuilder>) {
    if let (Some(section), Some(done)) = (sections.last_mut(), hunk.take()) {
        section.hunks.push(done.finish());
    }
}

fn read_meta(section: &mut FileSection, line: &str) {
    if read_path_meta(section, line) {
        return;
    }
    if line.starts_with("new file mode") {
        section.status = FileStatus::Added;
    } else if line.starts_with("deleted file mode") {
        section.status = FileStatus::Deleted;
    } else if let Some(mode) = line.strip_prefix("new mode ") {
        if is_type_change(mode) {
            section.status = FileStatus::TypeChanged;
        }
    } else if line.starts_with("Binary files ") || line == "GIT binary patch" {
        section.binary = true;
    }
}

/// Las cabeceras de renombrado y de copia. `true` si la linea era una de ellas.
fn read_path_meta(section: &mut FileSection, line: &str) -> bool {
    if let Some(from) = line.strip_prefix("rename from ") {
        section.status = FileStatus::Renamed;
        section.old_path = Some(unquote(from));
    } else if let Some(from) = line.strip_prefix("copy from ") {
        section.status = FileStatus::Copied;
        section.old_path = Some(unquote(from));
    } else if let Some(to) = line
        .strip_prefix("rename to ")
        .or_else(|| line.strip_prefix("copy to "))
    {
        section.new_path = Some(unquote(to));
    } else {
        return false;
    }
    true
}

/// Un cambio de modo a enlace o a submodulo es un cambio de tipo; el bit de
/// ejecucion no.
fn is_type_change(new_mode: &str) -> bool {
    matches!(new_mode.trim(), "120000" | "160000")
}

/// Primera linea y lineas de cada lado de `@@ -a,b +c,d @@ seccion`.
#[derive(Debug, Clone)]
struct HunkStart {
    old_start: u32,
    old_lines: u32,
    new_start: u32,
    new_lines: u32,
    section: String,
}

fn parse_hunk_header(line: &str) -> Option<HunkStart> {
    let rest = line.strip_prefix("@@ -")?;
    let (ranges, section) = rest.split_once(" @@")?;
    let (old, new) = ranges.split_once(" +")?;
    let (old_start, old_lines) = range(old)?;
    let (new_start, new_lines) = range(new)?;
    Some(HunkStart {
        old_start,
        old_lines,
        new_start,
        new_lines,
        section: section.strip_prefix(' ').unwrap_or(section).to_owned(),
    })
}

/// `a` o `a,b` (sin `,b` son una linea).
fn range(text: &str) -> Option<(u32, u32)> {
    match text.split_once(',') {
        Some((start, count)) => Some((start.parse().ok()?, count.parse().ok()?)),
        None => Some((text.parse().ok()?, 1)),
    }
}

#[derive(Debug)]
struct HunkBuilder {
    start: HunkStart,
    lines: Vec<DiffLine>,
    next_old: u32,
    next_new: u32,
    words: Option<Vec<WordSegment>>,
}

impl HunkBuilder {
    fn new(start: HunkStart, words: bool) -> Self {
        Self {
            next_old: start.old_start,
            next_new: start.new_start,
            start,
            lines: Vec::new(),
            words: words.then(Vec::new),
        }
    }

    fn feed(&mut self, line: &str) {
        if line.starts_with('\\') {
            if let Some(last) = self.lines.last_mut() {
                last.no_newline_at_end = true;
            }
            return;
        }
        if self.words.is_some() {
            self.feed_word(line);
            return;
        }
        let (kind, text) = match line.split_at_checked(1) {
            Some(("+", text)) => (LineKind::Added, text),
            Some(("-", text)) => (LineKind::Removed, text),
            Some((_, text)) => (LineKind::Context, text),
            None => (LineKind::Context, ""),
        };
        self.push_line(kind, text.to_owned(), Vec::new());
    }

    /// En porcelain cada trozo va en su linea con su signo, y `~` cierra la
    /// linea del fichero.
    fn feed_word(&mut self, line: &str) {
        let Some(segments) = self.words.as_mut() else {
            return;
        };
        let (kind, text) = match line.split_at_checked(1) {
            Some(("~", _)) => {
                let segments = std::mem::take(segments);
                self.close_word_line(segments);
                return;
            }
            Some(("+", text)) => (LineKind::Added, text),
            Some(("-", text)) => (LineKind::Removed, text),
            Some((_, text)) => (LineKind::Context, text),
            None => (LineKind::Context, ""),
        };
        segments.push(WordSegment {
            kind,
            text: text.to_owned(),
        });
    }

    /// Una linea solo con trozos anadidos es anadida; solo con quitados, quitada;
    /// cualquier otra existe en los dos lados.
    fn close_word_line(&mut self, segments: Vec<WordSegment>) {
        let has = |wanted: LineKind| segments.iter().any(|segment| segment.kind == wanted);
        let kind = match (
            has(LineKind::Added),
            has(LineKind::Removed),
            has(LineKind::Context),
        ) {
            (true, false, false) => LineKind::Added,
            (false, true, false) => LineKind::Removed,
            _ => LineKind::Context,
        };
        let text = segments
            .iter()
            .filter(|segment| kind == LineKind::Removed || segment.kind != LineKind::Removed)
            .map(|segment| segment.text.as_str())
            .collect();
        self.push_line(kind, text, segments);
    }

    fn push_line(&mut self, kind: LineKind, text: String, words: Vec<WordSegment>) {
        let old_line = (kind != LineKind::Added).then_some(self.next_old);
        let new_line = (kind != LineKind::Removed).then_some(self.next_new);
        self.next_old += u32::from(old_line.is_some());
        self.next_new += u32::from(new_line.is_some());
        self.lines.push(DiffLine {
            kind,
            old_line,
            new_line,
            text,
            words,
            no_newline_at_end: false,
        });
    }

    fn finish(mut self) -> Hunk {
        if let Some(segments) = self.words.take().filter(|pending| !pending.is_empty()) {
            self.words = Some(Vec::new());
            self.close_word_line(segments);
        }
        Hunk {
            old_start: self.start.old_start,
            old_lines: self.start.old_lines,
            new_start: self.start.new_start,
            new_lines: self.start.new_lines,
            section: self.start.section,
            lines: self.lines,
        }
    }
}

#[cfg(test)]
mod tests;
