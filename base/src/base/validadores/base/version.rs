use crate::base::validadores::utilidades_globales::severity::Severity;
use crate::base::validadores::utilidades_globales::space_cleaner::SpaceCleaner;

// ---------------------------------------------------------------------------
// Configuración
// ---------------------------------------------------------------------------
pub struct VersionConfig;

impl VersionConfig {

    pub const CAMPO: &'static str = "version_lock";

    // Rango numérico válido para Optimistic Locking
    pub const VALOR_MIN: i32 = 1;
    pub const VALOR_MAX: i32 = 2_147_483_647;

    // Límites de longitud para el validador de string
    pub const LONGITUD_MIN: usize = 1;
    pub const LONGITUD_MAX: usize = 10;
    pub const LONGITUD_MAX_RUIDO: usize = 15;

}

// ---------------------------------------------------------------------------
// Códigos de error
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCodeVersion {

    // --- Validador de string ---
    VersionVacia,              
    VersionTooShort,          
    VersionTooLong,            
    VersionFormatoInvalido,    

    // --- Validador numérico ---
    VersionTooSmall,           
    VersionTooLarge,         

}

impl std::fmt::Display for ErrorCodeVersion {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let code = match self {
            Self::VersionVacia           => "version_vacia",
            Self::VersionTooShort        => "version_too_short",
            Self::VersionTooLong         => "version_too_long",
            Self::VersionFormatoInvalido => "version_formato_invalido",
            Self::VersionTooSmall        => "version_too_small",
            Self::VersionTooLarge        => "version_too_large",
        };

        write!(f, "{code}")
    }
}

// ---------------------------------------------------------------------------
// Error de validación
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct VersionValidationError {

    pub message:    String,
    pub campo:      &'static str,
    pub error_code: ErrorCodeVersion,
    pub severity:   Severity,

}

impl std::fmt::Display for VersionValidationError {

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

impl std::error::Error for VersionValidationError {}

// ---------------------------------------------------------------------------
// Validador de String  →  i32
// ---------------------------------------------------------------------------

pub struct VersionStringValidador;

impl VersionStringValidador {

    /// Limpia, valida formato y longitud del string, lo parsea a `i32`
    /// y delega la validación de rango a `VersionValidador`.
    pub fn validar(valor: impl AsRef<str>) -> Result<i32, VersionValidationError> {

        let raw = valor.as_ref();

        // ------------------------------------------------------------------
        // 1 Limpieza
        // ------------------------------------------------------------------

        let limpio = SpaceCleaner::limpiar(
            raw,
            VersionConfig::LONGITUD_MAX_RUIDO,
            true,
        );

        if limpio.is_empty() {
            return Err(VersionValidationError {
                message:    "La versión de control no puede estar vacía.".into(),
                campo:      VersionConfig::CAMPO,
                error_code: ErrorCodeVersion::VersionVacia,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 2 Longitud segura
        // ------------------------------------------------------------------

        let largo = limpio.len();

        if largo < VersionConfig::LONGITUD_MIN {
            return Err(VersionValidationError {
                message: format!(
                    "La versión es demasiado corta (mínimo {} carácter).",
                    VersionConfig::LONGITUD_MIN
                ),
                campo:      VersionConfig::CAMPO,
                error_code: ErrorCodeVersion::VersionTooShort,
                severity:   Severity::Error,
            });
        }

        if largo > VersionConfig::LONGITUD_MAX {
            return Err(VersionValidationError {
                message: format!(
                    "La versión excede la longitud máxima permitida ({} dígitos).",
                    VersionConfig::LONGITUD_MAX
                ),
                campo:      VersionConfig::CAMPO,
                error_code: ErrorCodeVersion::VersionTooLong,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 3 Solo dígitos
        // ------------------------------------------------------------------

        if !limpio.chars().all(|c| c.is_ascii_digit()) {
            return Err(VersionValidationError {
                message: format!(
                    "La versión '{}' contiene caracteres no numéricos.",
                    limpio
                ),
                campo:      VersionConfig::CAMPO,
                error_code: ErrorCodeVersion::VersionFormatoInvalido,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 4 Parseo a i32
        // ------------------------------------------------------------------

        let valor_numerico = limpio.parse::<i32>().map_err(|_| VersionValidationError {
            message: format!(
                "El valor '{}' no se pudo convertir a un entero de 32 bits.",
                limpio
            ),
            campo:      VersionConfig::CAMPO,
            error_code: ErrorCodeVersion::VersionFormatoInvalido,
            severity:   Severity::Error,
        })?;

        // ------------------------------------------------------------------
        // 5 Rango lógico  →  delega al validador numérico
        // ------------------------------------------------------------------

        VersionValidador::validar(valor_numerico)

    }

}

// ---------------------------------------------------------------------------
// Validador numérico  i32  →  i32
// ---------------------------------------------------------------------------

pub struct VersionValidador;

impl VersionValidador {

    /// Valida que `valor` esté dentro del rango permitido para versiones
    pub fn validar(valor: i32) -> Result<i32, VersionValidationError> {

        // ------------------------------------------------------------------
        // 1 Mínimo
        // ------------------------------------------------------------------

        if valor < VersionConfig::VALOR_MIN {
            return Err(VersionValidationError {
                message: format!(
                    "La versión {} es menor que el mínimo permitido ({}).",
                    valor,
                    VersionConfig::VALOR_MIN
                ),
                campo:      VersionConfig::CAMPO,
                error_code: ErrorCodeVersion::VersionTooSmall,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 2 Máximo
        // ------------------------------------------------------------------

        if valor > VersionConfig::VALOR_MAX {
            return Err(VersionValidationError {
                message: format!(
                    "La versión {} excede el valor máximo permitido ({}).",
                    valor,
                    VersionConfig::VALOR_MAX
                ),
                campo:      VersionConfig::CAMPO,
                error_code: ErrorCodeVersion::VersionTooLarge,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 3 Valor validado
        // ------------------------------------------------------------------

        Ok(valor)

    }

}