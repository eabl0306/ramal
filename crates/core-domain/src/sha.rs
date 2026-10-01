//! Nombres de objeto de git: el SHA-1 completo y la forma corta que da `%h`.
//!
//! El completo se guarda como 20 bytes y no como texto: el grafo lo usa de clave
//! en tablas hash para miles de commits, y un `Copy` de 20 bytes no reserva
//! memoria. Solo se admite SHA-1 (40 hex): los repositorios SHA-256 siguen siendo
//! experimentales en git y el protocolo no los contempla todavia.

use std::fmt;
use std::str::FromStr;

use thiserror::Error;

/// Longitud en hexadecimal de un SHA-1 completo.
pub const SHA_HEX_LEN: usize = 40;

/// Longitud minima de una forma corta: git nunca abrevia por debajo de 4.
pub const SHORT_SHA_MIN_LEN: usize = 4;

const SHA_BYTES: usize = 20;

/// Fallo al leer un nombre de objeto.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ShaError {
    /// La longitud no es la esperada.
    #[error("object name has {found} hex characters; expected {expected}")]
    InvalidLength {
        /// Longitud o rango aceptado, para el mensaje.
        expected: &'static str,
        /// Longitud recibida.
        found: usize,
    },
    /// Algun caracter no es hexadecimal.
    #[error("object name contains a non hex character")]
    InvalidCharacter,
}

/// SHA-1 completo de un objeto de git.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Sha([u8; SHA_BYTES]);

impl Sha {
    /// Lee 40 caracteres hexadecimales, en mayusculas o minusculas.
    ///
    /// # Errors
    ///
    /// [`ShaError`] si la longitud no es 40 o hay un caracter que no es hexadecimal.
    pub fn parse(text: &str) -> Result<Self, ShaError> {
        if text.len() != SHA_HEX_LEN {
            return Err(ShaError::InvalidLength {
                expected: "40",
                found: text.len(),
            });
        }
        let mut bytes = [0_u8; SHA_BYTES];
        let (pairs, _) = text.as_bytes().as_chunks::<2>();
        for (slot, [high, low]) in bytes.iter_mut().zip(pairs) {
            *slot = (hex_value(*high)? << 4) | hex_value(*low)?;
        }
        Ok(Self(bytes))
    }

    /// Texto hexadecimal en minusculas, como lo escribe git.
    #[must_use]
    pub fn to_hex(self) -> String {
        self.to_string()
    }
}

impl fmt::Display for Sha {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0
            .iter()
            .try_for_each(|byte| write!(formatter, "{byte:02x}"))
    }
}

impl fmt::Debug for Sha {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Sha({self})")
    }
}

impl FromStr for Sha {
    type Err = ShaError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text)
    }
}

/// Forma corta de un nombre de objeto, tal como la abrevia git (`%h`).
///
/// Su longitud depende de `core.abbrev` y del tamano del repositorio, asi que no
/// se calcula: se guarda la que dio git.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShortSha(String);

impl ShortSha {
    /// Lee entre 4 y 40 caracteres hexadecimales y los guarda en minusculas.
    ///
    /// # Errors
    ///
    /// [`ShaError`] si la longitud se sale del rango o hay un caracter que no es
    /// hexadecimal.
    pub fn parse(text: &str) -> Result<Self, ShaError> {
        if !(SHORT_SHA_MIN_LEN..=SHA_HEX_LEN).contains(&text.len()) {
            return Err(ShaError::InvalidLength {
                expected: "4 to 40",
                found: text.len(),
            });
        }
        if !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ShaError::InvalidCharacter);
        }
        Ok(Self(text.to_ascii_lowercase()))
    }

    /// El texto de la forma corta.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ShortSha {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

const fn hex_value(byte: u8) -> Result<u8, ShaError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(ShaError::InvalidCharacter),
    }
}
