//! Instantes de git: segundos unix mas el desplazamiento horario del autor.
//!
//! git guarda cada fecha como segundos desde 1970 y la zona en la que se hizo el
//! commit, y el grafo ensena la hora local del autor, no la del lector. Por eso no
//! basta con un instante UTC. El texto de ida y vuelta es el ISO 8601 estricto de
//! `%aI` (`2026-09-30T11:42:00+02:00`), y la conversion de calendario se hace aqui
//! a mano (algoritmo de dias civiles de Howard Hinnant) para no traer chrono al
//! nucleo.

use std::fmt;

use thiserror::Error;

const SECONDS_PER_DAY: i64 = 86_400;
const SECONDS_PER_HOUR: i64 = 3_600;
const SECONDS_PER_MINUTE: i64 = 60;
/// Dias entre el 0000-03-01 proleptico y el 1970-01-01.
const UNIX_EPOCH_DAY: i64 = 719_468;
const DAYS_PER_ERA: i64 = 146_097;
/// Longitud de `YYYY-MM-DDTHH:MM:SS`.
const DATE_TIME_LEN: usize = 19;

/// Fallo al leer una fecha ISO 8601.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TimeError {
    /// El texto no tiene la forma `YYYY-MM-DDTHH:MM:SS` seguida de `Z` o `+HH:MM`.
    #[error("timestamp is not strict ISO 8601: {0}")]
    Malformed(String),
    /// Algun campo se sale de su rango (mes 13, dia 31 de abril...).
    #[error("timestamp has a field out of range: {0}")]
    OutOfRange(String),
}

/// Instante de git con el desplazamiento de la zona del autor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GitTime {
    seconds: i64,
    offset_minutes: i16,
}

impl GitTime {
    /// Construye el instante a partir de los segundos unix y el desplazamiento.
    #[must_use]
    pub const fn new(seconds: i64, offset_minutes: i16) -> Self {
        Self {
            seconds,
            offset_minutes,
        }
    }

    /// Segundos desde 1970-01-01T00:00:00Z.
    #[must_use]
    pub const fn seconds(&self) -> i64 {
        self.seconds
    }

    /// Desplazamiento de la zona del autor respecto a UTC, en minutos.
    #[must_use]
    pub const fn offset_minutes(&self) -> i16 {
        self.offset_minutes
    }

    /// Lee el ISO 8601 estricto de git: `YYYY-MM-DDTHH:MM:SS` mas `Z`, `+HH:MM` o
    /// `-HH:MM`.
    ///
    /// # Errors
    ///
    /// [`TimeError::Malformed`] si la forma no encaja y [`TimeError::OutOfRange`] si
    /// un campo no es una fecha real.
    pub fn parse_iso8601(text: &str) -> Result<Self, TimeError> {
        let malformed = || TimeError::Malformed(text.to_owned());
        let (date_time, zone) = text.split_at_checked(DATE_TIME_LEN).ok_or_else(malformed)?;
        let fields = parse_date_time(date_time).ok_or_else(malformed)?;
        let offset_minutes = parse_zone(zone).ok_or_else(malformed)?;
        if !fields.is_valid() || offset_minutes.abs() >= 100 * 60 {
            return Err(TimeError::OutOfRange(text.to_owned()));
        }
        let local = days_from_civil(fields.year, fields.month, fields.day) * SECONDS_PER_DAY
            + fields.hour * SECONDS_PER_HOUR
            + fields.minute * SECONDS_PER_MINUTE
            + fields.second;
        Ok(Self::new(
            local - i64::from(offset_minutes) * SECONDS_PER_MINUTE,
            offset_minutes,
        ))
    }

    /// Texto ISO 8601 estricto en la zona del autor. Un desplazamiento nulo se
    /// escribe `+00:00`.
    #[must_use]
    pub fn to_iso8601(self) -> String {
        self.to_string()
    }
}

impl fmt::Display for GitTime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let offset = i64::from(self.offset_minutes);
        let local = self.seconds + offset * SECONDS_PER_MINUTE;
        let (year, month, day) = civil_from_days(local.div_euclid(SECONDS_PER_DAY));
        let second_of_day = local.rem_euclid(SECONDS_PER_DAY);
        let sign = if offset < 0 { '-' } else { '+' };
        write!(
            formatter,
            "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}{sign}{:02}:{:02}",
            second_of_day / SECONDS_PER_HOUR,
            second_of_day % SECONDS_PER_HOUR / SECONDS_PER_MINUTE,
            second_of_day % SECONDS_PER_MINUTE,
            offset.abs() / 60,
            offset.abs() % 60,
        )
    }
}

/// Campos de `YYYY-MM-DDTHH:MM:SS` ya convertidos a numero.
struct DateTimeFields {
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
}

impl DateTimeFields {
    fn is_valid(&self) -> bool {
        (1..=12).contains(&self.month)
            && (1..=days_in_month(self.year, self.month)).contains(&self.day)
            && (0..24).contains(&self.hour)
            && (0..60).contains(&self.minute)
            && (0..60).contains(&self.second)
    }
}

fn parse_date_time(text: &str) -> Option<DateTimeFields> {
    let bytes = text.as_bytes();
    let separators = [(4, b'-'), (7, b'-'), (10, b'T'), (13, b':'), (16, b':')];
    if separators
        .iter()
        .any(|(index, expected)| bytes.get(*index) != Some(expected))
    {
        return None;
    }
    Some(DateTimeFields {
        year: digits(text, 0, 4)?,
        month: digits(text, 5, 2)?,
        day: digits(text, 8, 2)?,
        hour: digits(text, 11, 2)?,
        minute: digits(text, 14, 2)?,
        second: digits(text, 17, 2)?,
    })
}

/// `Z` es cero; si no, `+HH:MM` o `-HH:MM`.
fn parse_zone(zone: &str) -> Option<i16> {
    if zone == "Z" {
        return Some(0);
    }
    let sign = match zone.as_bytes().first() {
        Some(b'+') => 1,
        Some(b'-') => -1,
        _ => return None,
    };
    if zone.len() != 6 || zone.as_bytes().get(3) != Some(&b':') {
        return None;
    }
    let hours = digits(zone, 1, 2)?;
    let minutes = digits(zone, 4, 2)?;
    if minutes >= 60 {
        return None;
    }
    i16::try_from(sign * (hours * 60 + minutes)).ok()
}

/// Numero de `len` cifras decimales a partir de `start`. `str::parse` no vale
/// solo: acepta un signo delante.
fn digits(text: &str, start: usize, len: usize) -> Option<i64> {
    let slice = text.get(start..start + len)?;
    if !slice.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    slice.parse().ok()
}

const fn is_leap_year(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

const fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Dias desde 1970-01-01 de una fecha del calendario gregoriano proleptico.
const fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year.rem_euclid(400);
    let month_from_march = (month + 9) % 12;
    let day_of_year = (153 * month_from_march + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * DAYS_PER_ERA + day_of_era - UNIX_EPOCH_DAY
}

/// Inversa de [`days_from_civil`]: anio, mes y dia.
const fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + UNIX_EPOCH_DAY;
    let era = shifted.div_euclid(DAYS_PER_ERA);
    let day_of_era = shifted.rem_euclid(DAYS_PER_ERA);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_from_march = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_from_march + 2) / 5 + 1;
    let month = if month_from_march < 10 {
        month_from_march + 3
    } else {
        month_from_march - 9
    };
    let year = year_of_era + era * 400 + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}
