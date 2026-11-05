//! Lectura de instantes ISO 8601 estrictos, los que manda la interfaz (los
//! filtros de fecha del grafo).
//!
//! Se aceptan las dos formas que escribe una interfaz: una fecha sola
//! (`2026-09-30`, medianoche UTC) y un instante completo con zona
//! (`2026-09-30T10:00:00Z`, `2026-09-30T10:00:00.250+02:00`). Lo demas, o un dia
//! que no existe, es un error: un filtro mal escrito no se adivina.

use super::{
    GitTime, SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE, TimeError, UNIX_EPOCH_DAY,
};

impl GitTime {
    /// Lee `YYYY-MM-DD` o `YYYY-MM-DDTHH:MM:SS[.fraccion](Z|+HH:MM|-HH:MM)`. La
    /// fraccion de segundo se descarta.
    ///
    /// # Errors
    ///
    /// [`TimeError::Malformed`] si la forma no encaja y [`TimeError::OutOfRange`]
    /// si un campo no existe en el calendario (mes 13, 30 de febrero, 24:00).
    pub fn parse_iso8601(text: &str) -> Result<Self, TimeError> {
        let malformed = || TimeError::Malformed(text.to_owned());
        let out_of_range = || TimeError::OutOfRange(text.to_owned());
        let (date, rest) = text.split_at_checked(10).ok_or_else(malformed)?;
        let [year, month, day] = fields::<3>(date, '-', &[4, 2, 2]).ok_or_else(malformed)?;
        if !(1..=12).contains(&month) || day < 1 || day > days_in_month(year, month) {
            return Err(out_of_range());
        }
        let (second_of_day, offset) = if rest.is_empty() {
            (0, 0)
        } else {
            clock_and_zone(text, rest)?
        };
        let days = days_from_civil(year, month, day);
        let seconds =
            days * SECONDS_PER_DAY + second_of_day - i64::from(offset) * SECONDS_PER_MINUTE;
        Ok(Self::new(seconds, offset))
    }
}

/// `THH:MM:SS[.f](zona)` de `text`: segundo del dia y desplazamiento en minutos.
fn clock_and_zone(text: &str, rest: &str) -> Result<(i64, i16), TimeError> {
    let malformed = || TimeError::Malformed(text.to_owned());
    let out_of_range = || TimeError::OutOfRange(text.to_owned());
    let clock = rest.strip_prefix('T').ok_or_else(malformed)?;
    let (time, zone) = clock.split_at_checked(8).ok_or_else(malformed)?;
    let [hour, minute, second] = fields::<3>(time, ':', &[2, 2, 2]).ok_or_else(malformed)?;
    let zone = skip_fraction(zone).ok_or_else(malformed)?;
    let (sign, hours, minutes) = if zone == "Z" {
        (1, 0, 0)
    } else {
        let (sign, digits) = zone.split_at_checked(1).ok_or_else(malformed)?;
        let [hours, minutes] = fields::<2>(digits, ':', &[2, 2]).ok_or_else(malformed)?;
        let sign = match sign {
            "+" => 1,
            "-" => -1,
            _ => return Err(malformed()),
        };
        (sign, hours, minutes)
    };
    if hour > 23 || minute > 59 || second > 59 || hours > 23 || minutes > 59 {
        return Err(out_of_range());
    }
    let offset = i16::try_from(sign * (hours * 60 + minutes)).map_err(|_| out_of_range())?;
    Ok((
        hour * SECONDS_PER_HOUR + minute * SECONDS_PER_MINUTE + second,
        offset,
    ))
}

/// Quita `.123` del principio si esta (al menos una cifra).
fn skip_fraction(zone: &str) -> Option<&str> {
    let Some(fraction) = zone.strip_prefix('.') else {
        return Some(zone);
    };
    let digits = fraction.bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 {
        return None;
    }
    fraction.get(digits..)
}

/// `N` numeros separados por `separator`, cada uno con exactamente sus cifras.
fn fields<const N: usize>(text: &str, separator: char, widths: &[usize; N]) -> Option<[i64; N]> {
    let mut parts = text.split(separator);
    let mut out = [0; N];
    for (slot, width) in out.iter_mut().zip(widths) {
        let part = parts.next()?;
        if part.len() != *width || !part.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        *slot = part.parse().ok()?;
    }
    parts.next().is_none().then_some(out)
}

const fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

const fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        2 if is_leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Dias desde 1970-01-01 de una fecha del calendario gregoriano proleptico (la
/// inversa de `civil_from_days`, tambien de Howard Hinnant).
const fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year.rem_euclid(400);
    let month_from_march = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year = (153 * month_from_march + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - UNIX_EPOCH_DAY
}
