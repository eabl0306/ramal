//! Rutas entre comillas al estilo de C.
//!
//! git las escribe asi en las cabeceras de un diff cuando llevan caracteres de
//! control, comillas o barras invertidas (con `core.quotepath=off` los acentos
//! ya no se escapan).

/// Quita las comillas y los escapes (`\"`, `\\`, `\t`, `\n` y octales `\303`). Un
/// texto sin comillas se devuelve tal cual.
#[must_use]
pub fn unquote(text: &str) -> String {
    let Some(inner) = text
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    else {
        return text.to_owned();
    };
    let mut bytes = Vec::with_capacity(inner.len());
    let mut chars = inner.bytes().peekable();
    while let Some(byte) = chars.next() {
        if byte != b'\\' {
            bytes.push(byte);
            continue;
        }
        match chars.next() {
            Some(b't') => bytes.push(b'\t'),
            Some(b'n') => bytes.push(b'\n'),
            Some(b'r') => bytes.push(b'\r'),
            Some(digit @ b'0'..=b'7') => bytes.push(octal(digit, &mut chars)),
            Some(other) => bytes.push(other),
            None => bytes.push(b'\\'),
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Un escape octal: la primera cifra ya leida y hasta dos mas.
fn octal(first: u8, chars: &mut std::iter::Peekable<std::str::Bytes<'_>>) -> u8 {
    let mut value = u32::from(first - b'0');
    for _ in 0..2 {
        let Some(next) = chars.next_if(|next| (b'0'..=b'7').contains(next)) else {
            break;
        };
        value = value * 8 + u32::from(next - b'0');
    }
    u8::try_from(value).unwrap_or(b'?')
}

#[cfg(test)]
mod tests;
