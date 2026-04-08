use crate::base::validadores::utilidades_globales::severity::Severity;
use crate::base::validadores::utilidades_globales::space_cleaner::SpaceCleaner;

// ---------------------------------------------------------------------------
// Configuración
// ---------------------------------------------------------------------------

pub struct IdSimpleConfig;

impl IdSimpleConfig {

    pub const CAMPO: &'static str = "id_simple";

    // Rango numérico válido (i64)
    pub const VALOR_MIN: i64 = 1;
    pub const VALOR_MAX: i64 = 9_223_372_036_854_775_807;

    // Límites de longitud para el validador de string
    // El valor máximo de i64 tiene 19 dígitos.
    pub const LONGITUD_MIN: usize = 1;
    pub const LONGITUD_MAX: usize = 19;
    pub const LONGITUD_MAX_RUIDO: usize = 25;

}

// ---------------------------------------------------------------------------
// Códigos de error
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCodeIdSimple {

    // --- Validador de string ---
    IdSimpleVacio,              
    IdSimpleTooShort,          
    IdSimpleTooLong,            
    IdSimpleFormatoInvalido,    

    // --- Validador numérico ---
    IdSimpleTooSmall,           
    IdSimpleTooLarge,         

}

impl std::fmt::Display for ErrorCodeIdSimple {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let code = match self {
            Self::IdSimpleVacio           => "id_simple_vacio",
            Self::IdSimpleTooShort        => "id_simple_too_short",
            Self::IdSimpleTooLong         => "id_simple_too_long",
            Self::IdSimpleFormatoInvalido => "id_simple_formato_invalido",
            Self::IdSimpleTooSmall        => "id_simple_too_small",
            Self::IdSimpleTooLarge        => "id_simple_too_large",
        };

        write!(f, "{code}")
    }
}

// ---------------------------------------------------------------------------
// Error de validación
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct IdSimpleValidationError {

    pub message:    String,
    pub campo:      &'static str,
    pub error_code: ErrorCodeIdSimple,
    pub severity:   Severity,

}

impl std::fmt::Display for IdSimpleValidationError {

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

impl std::error::Error for IdSimpleValidationError {}

// ---------------------------------------------------------------------------
// Validador de String  →  i64
// ---------------------------------------------------------------------------

pub struct IdSimpleStringValidador;

impl IdSimpleStringValidador {

    /// Limpia, valida formato y longitud del string, lo parsea a `i64`
    /// y delega la validación de rango a `IdSimpleValidador`.
    pub fn validar(valor: impl AsRef<str>) -> Result<i64, IdSimpleValidationError> {

        let raw = valor.as_ref();

        // ------------------------------------------------------------------
        // 1 Limpieza
        // ------------------------------------------------------------------

        let limpio = SpaceCleaner::limpiar(
            raw,
            IdSimpleConfig::LONGITUD_MAX_RUIDO,
            true,
        );

        if limpio.is_empty() {
            return Err(IdSimpleValidationError {
                message:    "El identificador no puede estar vacío.".into(),
                campo:      IdSimpleConfig::CAMPO,
                error_code: ErrorCodeIdSimple::IdSimpleVacio,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 2 Longitud segura
        // ------------------------------------------------------------------

        let largo = limpio.len();

        if largo < IdSimpleConfig::LONGITUD_MIN {
            return Err(IdSimpleValidationError {
                message: format!(
                    "El identificador es demasiado corto (mínimo {} carácter).",
                    IdSimpleConfig::LONGITUD_MIN
                ),
                campo:      IdSimpleConfig::CAMPO,
                error_code: ErrorCodeIdSimple::IdSimpleTooShort,
                severity:   Severity::Error,
            });
        }

        if largo > IdSimpleConfig::LONGITUD_MAX {
            return Err(IdSimpleValidationError {
                message: format!(
                    "El identificador excede la longitud máxima permitida ({} dígitos).",
                    IdSimpleConfig::LONGITUD_MAX
                ),
                campo:      IdSimpleConfig::CAMPO,
                error_code: ErrorCodeIdSimple::IdSimpleTooLong,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 3 Solo dígitos
        // ------------------------------------------------------------------

        if !limpio.chars().all(|c| c.is_ascii_digit()) {
            return Err(IdSimpleValidationError {
                message: format!(
                    "El identificador '{}' contiene caracteres no numéricos.",
                    limpio
                ),
                campo:      IdSimpleConfig::CAMPO,
                error_code: ErrorCodeIdSimple::IdSimpleFormatoInvalido,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 4 Parseo a i64
        // ------------------------------------------------------------------

        let valor_numerico = limpio.parse::<i64>().map_err(|_| IdSimpleValidationError {
            message: format!(
                "El valor '{}' no se pudo convertir a un entero de 64 bits.",
                limpio
            ),
            campo:      IdSimpleConfig::CAMPO,
            error_code: ErrorCodeIdSimple::IdSimpleFormatoInvalido,
            severity:   Severity::Error,
        })?;

        // ------------------------------------------------------------------
        // 5 Rango lógico  →  delega al validador numérico
        // ------------------------------------------------------------------

        IdSimpleValidador::validar(valor_numerico)

    }

}

// ---------------------------------------------------------------------------
// Validador numérico  i64  →  i64
// ---------------------------------------------------------------------------

pub struct IdSimpleValidador;

impl IdSimpleValidador {

    /// Valida que `valor` esté dentro del rango permitido
    pub fn validar(valor: i64) -> Result<i64, IdSimpleValidationError> {

        // ------------------------------------------------------------------
        // 1 Mínimo
        // ------------------------------------------------------------------

        if valor < IdSimpleConfig::VALOR_MIN {
            return Err(IdSimpleValidationError {
                message: format!(
                    "El identificador {} es menor que el mínimo permitido ({}).",
                    valor,
                    IdSimpleConfig::VALOR_MIN
                ),
                campo:      IdSimpleConfig::CAMPO,
                error_code: ErrorCodeIdSimple::IdSimpleTooSmall,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 2 Máximo
        // ------------------------------------------------------------------

        if valor > IdSimpleConfig::VALOR_MAX {
            return Err(IdSimpleValidationError {
                message: format!(
                    "El identificador {} excede el valor máximo permitido ({}).",
                    valor,
                    IdSimpleConfig::VALOR_MAX
                ),
                campo:      IdSimpleConfig::CAMPO,
                error_code: ErrorCodeIdSimple::IdSimpleTooLarge,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 3 Valor validado
        // ------------------------------------------------------------------

        Ok(valor)

    }
}