//! Huella de una foto de refs: identifica el historial que ensena el grafo.
//!
//! El log del grafo recorre exactamente las refs de la foto mas HEAD, y un commit
//! no cambia nunca. Con las mismas refs, el mismo HEAD y la misma rama por
//! defecto (que fija el carril de la izquierda), git devuelve las mismas filas con
//! los mismos carriles. Por eso la interfaz pide cada pagina con la huella de la
//! primera, y el motor sabe si sigue hablando del mismo historial. Existe por un
//! fallo real: un commit hecho desde la terminal entre dos paginas hacia que la
//! segunda saliera de otro historial, con filas repetidas y carriles que saltaban.
//!
//! Es FNV-1a de 128 bits escrito a mano. El nucleo no puede traer crates de hash
//! (su lista blanca solo tiene thiserror) y el hasher de std no promete el mismo
//! algoritmo entre versiones de Rust. No hace falta que sea criptografico: solo
//! tiene que cambiar cuando cambian las refs, y una colision accidental en 128
//! bits es despreciable.

use std::fmt;

use super::{Head, RefSnapshot};

const FNV_OFFSET_BASIS: u128 = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d;
const FNV_PRIME: u128 = 0x0000_0000_0100_0000_0000_0000_0000_013b;

impl RefSnapshot {
    /// Huella de la foto: 32 caracteres hexadecimales que cambian si cambia
    /// cualquier ref, HEAD o la rama por defecto.
    ///
    /// El texto que se resume es una linea por dato, con campos separados por un
    /// espacio. No es ambiguo porque git no deja poner espacios ni saltos de
    /// linea en el nombre de una ref.
    #[must_use]
    pub fn fingerprint(&self) -> String {
        let mut hasher = Fnv1a128(FNV_OFFSET_BASIS);
        // Escribir en un hasher no falla nunca: el error de fmt::Write solo lo
        // puede devolver write_str, y este siempre devuelve Ok.
        let _ = self.write_canonical(&mut hasher);
        format!("{:032x}", hasher.0)
    }

    fn write_canonical(&self, out: &mut impl fmt::Write) -> fmt::Result {
        match &self.head {
            Head::Branch { name, target } => {
                write!(out, "head {} ", name.as_str())?;
                match target {
                    Some(sha) => writeln!(out, "{sha}")?,
                    None => writeln!(out, "-")?,
                }
            }
            Head::Detached { target } => writeln!(out, "head - {target}")?,
        }
        match &self.remote_default {
            Some(name) => writeln!(out, "default {}", name.as_str())?,
            None => writeln!(out, "default -")?,
        }
        for git_ref in &self.refs {
            writeln!(out, "{} {}", git_ref.name.as_str(), git_ref.target)?;
        }
        Ok(())
    }
}

/// Acumulador FNV-1a de 128 bits sobre los bytes que se le escriben.
struct Fnv1a128(u128);

impl fmt::Write for Fnv1a128 {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for byte in text.bytes() {
            self.0 ^= u128::from(byte);
            self.0 = self.0.wrapping_mul(FNV_PRIME);
        }
        Ok(())
    }
}
