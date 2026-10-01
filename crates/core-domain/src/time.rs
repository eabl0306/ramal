//! Instantes de git: segundos unix mas el desplazamiento horario del autor.
//!
//! git guarda cada fecha como segundos desde 1970 y la zona en la que se hizo el
//! commit, y el grafo ensena la hora local del autor, no la del lector. Por eso no
//! basta con un instante UTC.
//!
//! Se lee el formato de `--date=raw` (`1790761320 +0200`) y no el ISO de `%aI`:
//! git guarda en la cabecera del commit lo que le den, y hay commits reales
//! (importadores, libgit2, JGit) con zonas como `+0090` o anios de mas de cuatro
//! cifras. `%aI` los imprime tal cual (`+00:90`, `3170843-11-07`) y un parser ISO
//! estricto los rechazaba, con lo que un solo commit raro tumbaba el grafo entero.
//! La salida sigue siendo ISO 8601 estricto, que es lo que la interfaz sabe leer.
//! La conversion de calendario se hace aqui a mano (algoritmo de dias civiles de
//! Howard Hinnant) para no traer chrono al nucleo.

use std::fmt;

use thiserror::Error;

const SECONDS_PER_DAY: i64 = 86_400;
const SECONDS_PER_HOUR: i64 = 3_600;
const SECONDS_PER_MINUTE: i64 = 60;
const MINUTES_PER_DAY: i64 = 1_440;
/// Dias entre el 0000-03-01 proleptico y el 1970-01-01.
const UNIX_EPOCH_DAY: i64 = 719_468;
const DAYS_PER_ERA: i64 = 146_097;
/// `0000-01-01T00:00:00` y `9999-12-31T23:59:59` en segundos unix: el rango de
/// anios de cuatro cifras que admite ISO 8601 sin extensiones (y `Date.parse`).
const FIRST_ISO_SECOND: i64 = -62_167_219_200;
const LAST_ISO_SECOND: i64 = 253_402_300_799;
/// git escribe la zona con `%+05d`: signo y al menos cuatro cifras.
const MIN_ZONE_DIGITS: usize = 4;

/// Fallo al leer una fecha de git.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TimeError {
    /// El texto no tiene la forma `<segundos> <+|-><HHMM>`.
    #[error("timestamp is not in git raw format: {0}")]
    Malformed(String),
    /// Los segundos o el desplazamiento no caben en su tipo.
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

    /// Lee el formato de `--date=raw`: segundos unix, un espacio y la zona como
    /// `+HHMM` o `-HHMM`.
    ///
    /// La zona se interpreta como lo hace git, sin exigir que los minutos bajen
    /// de 60: `+0090` son 90 minutos. Las dos ultimas cifras son los minutos y las
    /// de delante las horas.
    ///
    /// # Errors
    ///
    /// [`TimeError::Malformed`] si la forma no encaja y [`TimeError::OutOfRange`] si
    /// los segundos no caben en un `i64` o el desplazamiento en un `i16`.
    pub fn parse_raw(text: &str) -> Result<Self, TimeError> {
        let malformed = || TimeError::Malformed(text.to_owned());
        let out_of_range = || TimeError::OutOfRange(text.to_owned());
        let (seconds, zone) = text.split_once(' ').ok_or_else(malformed)?;
        let (sign, zone_digits) = match zone.split_at_checked(1) {
            Some(("+", digits)) => (1, digits),
            Some(("-", digits)) => (-1, digits),
            _ => return Err(malformed()),
        };
        if !is_digits(seconds, 1) || !is_digits(zone_digits, MIN_ZONE_DIGITS) {
            return Err(malformed());
        }
        let seconds: i64 = seconds.parse().map_err(|_| out_of_range())?;
        let zone: i64 = zone_digits.parse().map_err(|_| out_of_range())?;
        let offset =
            i16::try_from(sign * (zone / 100 * 60 + zone % 100)).map_err(|_| out_of_range())?;
        Ok(Self::new(seconds, offset))
    }

    /// Texto ISO 8601 estricto en la zona del autor (ver [`fmt::Display`]).
    #[must_use]
    pub fn to_iso8601(self) -> String {
        self.to_string()
    }
}

/// ISO 8601 estricto (`2026-09-30T11:42:00+02:00`), siempre legible por
/// `Date.parse`. Un desplazamiento nulo se escribe `+00:00`.
///
/// git acepta cualquier cosa en la cabecera, asi que el texto se normaliza en
/// vez de copiar el valor raro: una zona de 24 horas o mas no existe y el
/// instante se ensena en UTC, y un instante fuera de los anios 0000 a 9999 se
/// recorta al extremo. Sin esto la interfaz recibia `+100:39` o `10000-01-01` y no
/// podia leer la fecha.
impl fmt::Display for GitTime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let offset = i64::from(self.offset_minutes);
        let offset = if offset.abs() < MINUTES_PER_DAY {
            offset
        } else {
            0
        };
        let local = self
            .seconds
            .saturating_add(offset * SECONDS_PER_MINUTE)
            .clamp(FIRST_ISO_SECOND, LAST_ISO_SECOND);
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

/// Al menos `min_len` cifras decimales y nada mas. `str::parse` no vale solo:
/// acepta un signo delante.
fn is_digits(text: &str, min_len: usize) -> bool {
    text.len() >= min_len && text.bytes().all(|byte| byte.is_ascii_digit())
}

/// Anio, mes y dia de un numero de dias desde 1970-01-01, en el calendario
/// gregoriano proleptico.
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
