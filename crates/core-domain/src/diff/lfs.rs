//! Punteros de Git LFS.
//!
//! Un fichero guardado con LFS es, para git, un texto de tres lineas que apunta
//! al objeto real. Ensenar ese texto como diff no le sirve a nadie: la interfaz
//! dice que es un fichero de LFS y cuanto ocupa.

/// Primera linea de cualquier puntero LFS.
const VERSION_LINE: &str = "version https://git-lfs.github.com/spec/v1";

/// Un puntero de LFS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LfsPointer {
    /// `sha256:<hex>` del objeto real.
    pub oid: String,
    /// Tamano del objeto real en bytes.
    pub size: u64,
}

impl LfsPointer {
    /// Lee un puntero a partir del texto completo del fichero, o de las lineas
    /// de un lado del diff. Devuelve `None` si no lo es.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let mut lines = text.lines().map(str::trim_end);
        if lines.next()? != VERSION_LINE {
            return None;
        }
        let mut oid = None;
        let mut size = None;
        for line in lines.filter(|line| !line.is_empty()) {
            let (key, value) = line.split_once(' ')?;
            match key {
                "oid" => oid = Some(value.to_owned()),
                "size" => size = value.parse().ok(),
                _ => {}
            }
        }
        Some(Self {
            oid: oid?,
            size: size?,
        })
    }
}
