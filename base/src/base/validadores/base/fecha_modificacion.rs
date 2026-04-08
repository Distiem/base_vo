use chrono::{DateTime, Utc, Duration};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::base::validadores::utilidades_globales::severity::Severity;
use crate::base::validadores::utilidades_globales::space_cleaner::SpaceCleaner;
use crate::base::dominio::fecha_creacion::FechaCreacion;

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
// Configuración
// ---------------------------------------------------------------------------

pub struct FechaModificacionConfig;

impl FechaModificacionConfig {

    pub const CAMPO: &'static str              = "fecha_modificacion";

    pub const FUTURO_TOLERANCIA_SEGUNDOS: i64  = 60;
    pub const MINIMA_FECHA_PERMITIDA: &str     = "2020-01-01T00:00:00Z"; //Poner en 2025?
    pub const MAXIMA_FECHA_PERMITIDA: &str     = "2100-01-01T00:00:00Z";

    pub const LONGITUD_MAX_RUIDO: usize        = 150;
    pub const LONGITUD_MAXIMA_PERMITIDA: usize = 64;

}

// ---------------------------------------------------------------------------
// Códigos de error
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCodeFechaModificacion {

    FechaEmpty,
    FechaInvalidFormat,
    FechaTooLong,
    FechaInFuture,
    FechaOutOfRange,
    FechaCorrupta,
    FechaAnteriorACreacion,  

}

impl std::fmt::Display for ErrorCodeFechaModificacion {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let code = match self {
            Self::FechaEmpty             => "fecha_modificacion_empty",
            Self::FechaInvalidFormat     => "fecha_modificacion_invalid_format",
            Self::FechaTooLong           => "fecha_modificacion_too_long",
            Self::FechaInFuture          => "fecha_modificacion_in_future",
            Self::FechaOutOfRange        => "fecha_modificacion_out_of_range",
            Self::FechaCorrupta          => "fecha_modificacion_corrupta",
            Self::FechaAnteriorACreacion => "fecha_modificacion_anterior_a_creacion",
        };

        write!(f, "{code}")
    }

}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct FechaModificacionValidationError {

    pub message:    String,
    pub campo:      &'static str,
    pub error_code: ErrorCodeFechaModificacion,
    pub severity:   Severity,

}

impl std::fmt::Display for FechaModificacionValidationError {

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

impl std::error::Error for FechaModificacionValidationError {}

// ---------------------------------------------------------------------------
// Validador de String  →  DateTime<Utc>
// ---------------------------------------------------------------------------

pub struct FechaModificacionStringValidador;

impl FechaModificacionStringValidador {

    pub fn validar(
        valor:          impl AsRef<str>,
        fecha_creacion: FechaCreacion,
    ) -> Result<DateTime<Utc>, FechaModificacionValidationError> {

        let raw = valor.as_ref();

        // ------------------------------------------------------------------
        // 1 Limpieza
        // ------------------------------------------------------------------

        let limpio = SpaceCleaner::limpiar(
            raw,
            FechaModificacionConfig::LONGITUD_MAX_RUIDO,
            true,
        );

        if limpio.is_empty() {
            return Err(FechaModificacionValidationError {
                message:    "La fecha de modificación no puede estar vacía.".into(),
                campo:      FechaModificacionConfig::CAMPO,
                error_code: ErrorCodeFechaModificacion::FechaEmpty,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 2 Longitud segura
        // ------------------------------------------------------------------

        if limpio.len() > FechaModificacionConfig::LONGITUD_MAXIMA_PERMITIDA {
            return Err(FechaModificacionValidationError {
                message:    "La fecha de modificación excede la longitud máxima permitida.".into(),
                campo:      FechaModificacionConfig::CAMPO,
                error_code: ErrorCodeFechaModificacion::FechaTooLong,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 3 Regex ISO estricto
        // ------------------------------------------------------------------

        if !ISO_8601_STRICT.is_match(&limpio) {
            return Err(FechaModificacionValidationError {
                message:    "Formato de fecha inválido. Se requiere ISO-8601 con zona horaria.".into(),
                campo:      FechaModificacionConfig::CAMPO,
                error_code: ErrorCodeFechaModificacion::FechaInvalidFormat,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 4 Parsing seguro
        // ------------------------------------------------------------------

        let parsed = DateTime::parse_from_rfc3339(&limpio)
            .map_err(|_| FechaModificacionValidationError {
                message:    "No se pudo parsear la fecha de modificación.".into(),
                campo:      FechaModificacionConfig::CAMPO,
                error_code: ErrorCodeFechaModificacion::FechaInvalidFormat,
                severity:   Severity::Error,
            })?
            .with_timezone(&Utc);

        // ------------------------------------------------------------------
        // 5 Rango, futuro y relación con creacion  →  delega al validador DateTime
        // ------------------------------------------------------------------

        FechaModificacionValidador::validar(parsed, fecha_creacion)

    }

}

// ---------------------------------------------------------------------------
// Validador de DateTime<Utc>  →  DateTime<Utc>
// ---------------------------------------------------------------------------

pub struct FechaModificacionValidador;

impl FechaModificacionValidador {

    pub fn validar(
        valor:          DateTime<Utc>,
        fecha_creacion: FechaCreacion,
    ) -> Result<DateTime<Utc>, FechaModificacionValidationError> {

        let fecha = valor.with_timezone(&Utc);

        // ------------------------------------------------------------------
        // 1 Protección contra valores corruptos
        // ------------------------------------------------------------------

        if fecha.timestamp() <= 0 {
            return Err(FechaModificacionValidationError {
                message:    "La fecha de modificación parece inválida o corrupta (timestamp no válido).".into(),
                campo:      FechaModificacionConfig::CAMPO,
                error_code: ErrorCodeFechaModificacion::FechaCorrupta,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 2 Rango permitido absoluto
        // ------------------------------------------------------------------

        let min = DateTime::parse_from_rfc3339(FechaModificacionConfig::MINIMA_FECHA_PERMITIDA)
            .unwrap()
            .with_timezone(&Utc);

        let max = DateTime::parse_from_rfc3339(FechaModificacionConfig::MAXIMA_FECHA_PERMITIDA)
            .unwrap()
            .with_timezone(&Utc);

        if fecha < min || fecha > max {
            return Err(FechaModificacionValidationError {
                message: format!(
                    "La fecha de modificación está fuera del rango permitido. Recibida: {}",
                    fecha.to_rfc3339()
                ),
                campo:      FechaModificacionConfig::CAMPO,
                error_code: ErrorCodeFechaModificacion::FechaOutOfRange,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 3 Tolerancia de futuro
        // ------------------------------------------------------------------

        let ahora = Utc::now();
        let diff  = fecha - ahora;

        if diff > Duration::seconds(FechaModificacionConfig::FUTURO_TOLERANCIA_SEGUNDOS) {
            return Err(FechaModificacionValidationError {
                message: format!(
                    "La fecha de modificación excede la tolerancia hacia el futuro. \
                     Fecha: {} | Ahora: {}",
                    fecha.to_rfc3339(),
                    ahora.to_rfc3339()
                ),
                campo:      FechaModificacionConfig::CAMPO,
                error_code: ErrorCodeFechaModificacion::FechaInFuture,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 4 Relación con fecha_creacion 
        // ------------------------------------------------------------------

        let creacion = fecha_creacion.valor();

        if fecha < creacion {
            return Err(FechaModificacionValidationError {
                message: format!(
                    "La fecha de modificación no puede ser anterior a la de creación. \
                     Modificación: {} | Creación: {}",
                    fecha.to_rfc3339(),
                    creacion.to_rfc3339()
                ),
                campo:      FechaModificacionConfig::CAMPO,
                error_code: ErrorCodeFechaModificacion::FechaAnteriorACreacion,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 5 Normalización final
        // ------------------------------------------------------------------

        Ok(fecha)
    }

}