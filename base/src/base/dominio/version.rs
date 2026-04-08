use std::fmt;
use std::str::FromStr;
use serde::{Serialize, Deserialize};

use crate::base::validadores::base::version::{
    VersionValidador, 
    VersionStringValidador, 
    VersionValidationError, 
    VersionConfig
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Version(i32);

impl Version {
    
    // ---------------------------------------------------------
    // CONSTRUCTOR INTERNO
    // ---------------------------------------------------------

    /// Constructor interno que asegura el paso por el validador lógico.
    fn new(valor: i32) -> Result<Self, VersionValidationError> {
        let valor_validado = VersionValidador::validar(valor)?;
        Ok(Self(valor_validado))
    }

    // ---------------------------------------------------------
    // FÁBRICAS
    // ---------------------------------------------------------

    /// Crea una Version desde un i32 existente.
    pub fn desde_i32(valor: i32) -> Result<Self, VersionValidationError> {
        Self::new(valor)
    }

    /// Crea una Version desde un string validado.
    pub fn desde_string(valor: &str) -> Result<Self, VersionValidationError> {
        let parsed = VersionStringValidador::validar(valor)?;
        Ok(Self(parsed))
    }

    /// Crea una versión inicial (1). Útil para nuevos registros.
    pub fn inicial() -> Self {
        Self(VersionConfig::VALOR_MIN)
    }

    // ---------------------------------------------------------
    // LÓGICA DE NEGOCIO (Optimistic Locking)
    // ---------------------------------------------------------

    /// Genera la siguiente versión para una operación de escritura.
    /// Devuelve un error si se alcanza el límite máximo de i32.
    pub fn siguiente(&self) -> Result<Self, VersionValidationError> {
        let nuevo_valor = self.0.checked_add(1).ok_or(VersionValidationError {
            message: "Se ha alcanzado el límite máximo de versiones para este registro.".into(),
            campo: VersionConfig::CAMPO,
            error_code: crate::base::validadores::base::version::ErrorCodeVersion::VersionTooLarge,
            severity: crate::base::validadores::utilidades_globales::severity::Severity::Critical,
        })?;

        Self::new(nuevo_valor)
    }

    // ---------------------------------------------------------
    // GETTERS
    // ---------------------------------------------------------

    pub fn valor(&self) -> i32 {
        self.0
    }

    // ---------------------------------------------------------
    // VALIDACIÓN DE CONCURRENCIA
    // ---------------------------------------------------------

    /// Compara la versión actual con la versión recuperada de la base de datos.
    /// 
    /// Si no coinciden, significa que otro proceso modificó el registro 
    /// en el intervalo entre la lectura y la escritura (Conflicto).
    pub fn verificar_conflicto(&self, version_db: &Version) -> Result<(), VersionValidationError> {
        if self.0 != version_db.valor() {
            return Err(VersionValidationError {
                message: format!(
                    "Conflicto de concurrencia: la versión proporcionada ({}) no coincide con la versión actual en base de datos ({}).",
                    self.0,
                    version_db.valor()
                ),
                campo:      VersionConfig::CAMPO,
                error_code: crate::base::validadores::base::version::ErrorCodeVersion::VersionFormatoInvalido,
                severity:   crate::base::validadores::utilidades_globales::severity::Severity::Error,
            });
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// IMPLEMENTACIÓN DE TRAITS DE CONVERSIÓN
// ---------------------------------------------------------------------------

impl FromStr for Version {
    type Err = VersionValidationError;

    /// Permite crear una Version usando `"texto".parse()`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::desde_string(s)
    }
}

impl TryFrom<i32> for Version {
    type Error = VersionValidationError;

    /// Permite convertir desde i32 usando try_from / try_into.
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::desde_i32(value)
    }
}

impl TryFrom<&str> for Version {
    type Error = VersionValidationError;

    /// Permite convertir desde &str usando try_from / try_into.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::desde_string(value)
    }
}

// ---------------------------------------------------------------------------
// PRESENTACIÓN
// ---------------------------------------------------------------------------

impl fmt::Display for Version {

    /// Representación estándar del número de versión.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}