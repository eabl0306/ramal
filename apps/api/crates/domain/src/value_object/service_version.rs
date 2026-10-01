//! Version semantica del servicio.

use thiserror::Error;

/// Motivos por los que una version se rechaza.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ServiceVersionError {
    /// El texto no tiene la forma `major.minor.patch`.
    #[error("service version must look like `major.minor.patch`, got {0:?}")]
    MalformedCore(String),
    /// Alguno de los tres numeros no es un entero sin signo valido.
    #[error("service version segment {segment:?} is not a valid number")]
    NonNumericSegment {
        /// Segmento que no se pudo leer.
        segment: String,
    },
}

/// Version semantica reducida a `major.minor.patch`.
///
/// Sin prerelease ni metadatos de build: el contrato publico expone una version
/// simple y comparable, la misma en `/health/*` y en el documento OpenAPI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ServiceVersion {
    major: u16,
    minor: u16,
    patch: u16,
}

impl ServiceVersion {
    /// Construye una version a partir de sus tres numeros.
    #[must_use]
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Lee una cadena `major.minor.patch`.
    ///
    /// # Errors
    ///
    /// Devuelve [`ServiceVersionError`] si faltan o sobran segmentos o alguno no
    /// es un numero.
    pub fn parse(raw: &str) -> Result<Self, ServiceVersionError> {
        let trimmed = raw.trim();
        let mut segments = trimmed.split('.');
        let (Some(major), Some(minor), Some(patch), None) = (
            segments.next(),
            segments.next(),
            segments.next(),
            segments.next(),
        ) else {
            return Err(ServiceVersionError::MalformedCore(trimmed.to_owned()));
        };
        Ok(Self {
            major: parse_segment(major)?,
            minor: parse_segment(minor)?,
            patch: parse_segment(patch)?,
        })
    }

    /// Numero mayor.
    #[must_use]
    pub const fn major(self) -> u16 {
        self.major
    }

    /// Numero menor.
    #[must_use]
    pub const fn minor(self) -> u16 {
        self.minor
    }

    /// Numero de parche.
    #[must_use]
    pub const fn patch(self) -> u16 {
        self.patch
    }
}

impl std::fmt::Display for ServiceVersion {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

fn parse_segment(segment: &str) -> Result<u16, ServiceVersionError> {
    segment
        .parse::<u16>()
        .map_err(|_| ServiceVersionError::NonNumericSegment {
            segment: segment.to_owned(),
        })
}

#[cfg(test)]
mod tests;
