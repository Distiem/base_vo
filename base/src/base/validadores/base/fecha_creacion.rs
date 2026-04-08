use chrono::{DateTime, Utc, Duration};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::base::validadores::utilidades_globales::severity::Severity;
use crate::base::validadores::utilidades_globales::space_cleaner::SpaceCleaner;

// ---------------------------------------------------------------------------
// Regex ISO 8601 estricto
// ---------------------------------------------------------------------------

static ISO_8601_STRICT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d{1,6})?(Z|[+-]\d{2}:\d{2})$"#
    )
    .expect("Regex ISO8601 inválido")
});

// ---------------------------------------------------------------------------
// Configuración compartida
// ---------------------------------------------------------------------------

pub struct FechaConfig;

impl FechaConfig {

    // Compartidas
    pub const CAMPO: &'static str              = "fecha_creacion";
    pub const FUTURO_TOLERANCIA_SEGUNDOS: i64  = 60;
    pub const MINIMA_FECHA_PERMITIDA: &str     = "2020-01-01T00:00:00Z";
    pub const MAXIMA_FECHA_PERMITIDA: &str     = "2100-01-01T00:00:00Z";

    // Exclusivas del validador de string
    pub const LONGITUD_MAX_RUIDO: usize        = 150;
    pub const LONGITUD_MAXIMA_PERMITIDA: usize = 64;

}

// ---------------------------------------------------------------------------
// Códigos de error
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCodeFecha {

    FechaInvalidType,
    FechaEmpty,
    FechaInvalidFormat,
    FechaTooLong,
    FechaInFuture,
    FechaOutOfRange,
    FechaCorrupta,      

}

impl std::fmt::Display for ErrorCodeFecha {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let code = match self {
            Self::FechaInvalidType  => "fecha_invalid_type",
            Self::FechaEmpty        => "fecha_empty",
            Self::FechaInvalidFormat => "fecha_invalid_format",
            Self::FechaTooLong      => "fecha_too_long",
            Self::FechaInFuture     => "fecha_in_future",
            Self::FechaOutOfRange   => "fecha_out_of_range",
            Self::FechaCorrupta     => "fecha_corrupta",
        };

        write!(f, "{code}")
    }

}

// ---------------------------------------------------------------------------
// Error compartido
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct FechaValidationError {

    pub message:    String,
    pub campo:      &'static str,
    pub error_code: ErrorCodeFecha,
    pub severity:   Severity,

}

impl std::fmt::Display for FechaValidationError {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        write!(
            f,
            "[{}][{}] {}: {}",
            self.severity,
            self.campo,
            self.error_code,
            self.message
        )

    }

}

impl std::error::Error for FechaValidationError {}

// ---------------------------------------------------------------------------
// Validador de String  →  DateTime<Utc>
// ---------------------------------------------------------------------------

pub struct FechaCreacionStringValidador;

impl FechaCreacionStringValidador {

    pub fn validar(valor: impl AsRef<str>) -> Result<DateTime<Utc>, FechaValidationError> {

        let raw = valor.as_ref();

        // ------------------------------------------------------------------
        // 1 Limpieza
        // ------------------------------------------------------------------

        let limpio = SpaceCleaner::limpiar(
            raw,
            FechaConfig::LONGITUD_MAX_RUIDO,
            true,
        );

        if limpio.is_empty() {
            return Err(FechaValidationError {
                message:    "La fecha no puede estar vacía.".into(),
                campo:      FechaConfig::CAMPO,
                error_code: ErrorCodeFecha::FechaEmpty,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 2 Longitud segura
        // ------------------------------------------------------------------

        if limpio.len() > FechaConfig::LONGITUD_MAXIMA_PERMITIDA {
            return Err(FechaValidationError {
                message:    "La fecha excede la longitud máxima permitida.".into(),
                campo:      FechaConfig::CAMPO,
                error_code: ErrorCodeFecha::FechaTooLong,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 3 Regex ISO estricto
        // ------------------------------------------------------------------

        if !ISO_8601_STRICT.is_match(&limpio) {
            return Err(FechaValidationError {
                message:    "Formato de fecha inválido. Se requiere ISO-8601 con zona horaria.".into(),
                campo:      FechaConfig::CAMPO,
                error_code: ErrorCodeFecha::FechaInvalidFormat,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 4 Parsing seguro
        // ------------------------------------------------------------------

        let parsed = DateTime::parse_from_rfc3339(&limpio)
            .map_err(|_| FechaValidationError {
                message:    "No se pudo parsear la fecha.".into(),
                campo:      FechaConfig::CAMPO,
                error_code: ErrorCodeFecha::FechaInvalidFormat,
                severity:   Severity::Error,
            })?
            .with_timezone(&Utc);

        // ------------------------------------------------------------------
        // 5 Rango absoluto + futuro  →  delega al validador DateTime
        // ------------------------------------------------------------------

        FechaCreacionValidador::validar(parsed)

    }

}

// ---------------------------------------------------------------------------
// Validador de DateTime<Utc>  →  DateTime<Utc>
// ---------------------------------------------------------------------------

pub struct FechaCreacionValidador;

impl FechaCreacionValidador {

    pub fn validar(valor: DateTime<Utc>) -> Result<DateTime<Utc>, FechaValidationError> {

        let fecha = valor.with_timezone(&Utc);

        // ------------------------------------------------------------------
        // 1 Protección contra valores corruptos
        // ------------------------------------------------------------------

        if fecha.timestamp() <= 0 {
            return Err(FechaValidationError {
                message:    "La fecha parece inválida o corrupta (timestamp no válido).".into(),
                campo:      FechaConfig::CAMPO,
                error_code: ErrorCodeFecha::FechaCorrupta,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 2 Rango permitido absoluto
        // ------------------------------------------------------------------

        let min = DateTime::parse_from_rfc3339(FechaConfig::MINIMA_FECHA_PERMITIDA)
            .unwrap()
            .with_timezone(&Utc);

        let max = DateTime::parse_from_rfc3339(FechaConfig::MAXIMA_FECHA_PERMITIDA)
            .unwrap()
            .with_timezone(&Utc);

        if fecha < min || fecha > max {
            return Err(FechaValidationError {
                message: format!(
                    "La fecha está fuera del rango permitido. Recibida: {}",
                    fecha.to_rfc3339()
                ),
                campo:      FechaConfig::CAMPO,
                error_code: ErrorCodeFecha::FechaOutOfRange,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 3 Tolerancia de futuro
        // ------------------------------------------------------------------

        let ahora = Utc::now();
        let diff  = fecha - ahora;

        if diff > Duration::seconds(FechaConfig::FUTURO_TOLERANCIA_SEGUNDOS) {
            return Err(FechaValidationError {
                message: format!(
                    "La fecha excede la tolerancia hacia el futuro. \
                     Fecha: {} | Ahora: {}",
                    fecha.to_rfc3339(),
                    ahora.to_rfc3339()
                ),
                campo:      FechaConfig::CAMPO,
                error_code: ErrorCodeFecha::FechaInFuture,
                severity:   Severity::Error,
            });
        }

        Ok(fecha)
    }
}