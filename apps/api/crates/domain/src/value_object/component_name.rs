//! Nombre de un componente observable del servicio.

use thiserror::Error;

/// Longitud maxima del nombre de un componente.
const MAX_LENGTH: usize = 64;

/// Motivos por los que un nombre de componente se rechaza.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ComponentNameError {
    /// El nombre estaba vacio o solo tenia espacios.
    #[error("component name must not be empty")]
    Empty,
    /// El nombre pasa del limite de caracteres.
    #[error("component name must not exceed {MAX_LENGTH} characters, got {0}")]
    TooLong(usize),
    /// El nombre lleva un caracter fuera del juego permitido.
    #[error("component name contains an unsupported character: {0:?}")]
    UnsupportedCharacter(char),
}

/// Nombre normalizado de un componente, por ejemplo `database`.
///
/// Es parte del contrato publico de `/health/ready`: los clientes buscan los
/// componentes por este nombre. Solo admite minusculas ASCII, digitos, guion y
/// guion bajo; se recortan los espacios y se pasa a minusculas.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentName(String);

impl ComponentName {
    /// Construye un nombre validado a partir de texto libre.
    ///
    /// # Errors
    ///
    /// Devuelve [`ComponentNameError`] si el texto esta vacio, es demasiado
    /// largo o lleva caracteres no admitidos.
    pub fn parse(raw: &str) -> Result<Self, ComponentNameError> {
        let normalized = raw.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            return Err(ComponentNameError::Empty);
        }
        if normalized.len() > MAX_LENGTH {
            return Err(ComponentNameError::TooLong(normalized.len()));
        }
        if let Some(invalid) = normalized.chars().find(|c| !is_allowed(*c)) {
            return Err(ComponentNameError::UnsupportedCharacter(invalid));
        }
        Ok(Self(normalized))
    }

    /// Texto del nombre.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ComponentName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

const fn is_allowed(character: char) -> bool {
    character.is_ascii_lowercase() || character.is_ascii_digit() || matches!(character, '-' | '_')
}

#[cfg(test)]
mod tests;
