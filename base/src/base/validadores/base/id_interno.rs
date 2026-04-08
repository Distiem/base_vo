use uuid::{Uuid, Version};
use crate::base::validadores::utilidades_globales::severity::Severity;
use crate::base::validadores::utilidades_globales::space_cleaner::SpaceCleaner;

// ---------------------------------------------------------------------------
// Configuración
// ---------------------------------------------------------------------------

pub struct IdInternoConfig;

impl IdInternoConfig {
    pub const CAMPO: &'static str = "id_interno";
    
    pub const PERMITIR_UUID_NIL: bool = false;
    pub const VERSION_REQUERIDA: Option<Version> = Some(Version::Random); // UUID v4

    // Un UUID estándar tiene 36 caracteres (8-4-4-4-12)
    pub const LONGITUD_UUID: usize = 36;
    pub const LONGITUD_MAX_RUIDO: usize = 45;
}

// ---------------------------------------------------------------------------
// Códigos de error
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCodeIdInterno {
    // --- Validador de string ---
    IdInternoVacio,
    IdInternoFormatoInvalido,
    IdInternoLongitudInvalida,

    // --- Validador lógico (UUID) ---
    UuidNil,
    UuidInvalidVersion,
}

impl std::fmt::Display for ErrorCodeIdInterno {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::IdInternoVacio               => "id_interno_vacio",
            Self::IdInternoFormatoInvalido     => "id_interno_formato_invalido",
            Self::IdInternoLongitudInvalida    => "id_interno_longitud_invalida",
            Self::UuidNil                      => "uuid_nil",
            Self::UuidInvalidVersion           => "uuid_invalid_version",
        };
        write!(f, "{s}")
    }
}

// ---------------------------------------------------------------------------
// Error de validación
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct IdInternoValidationError {
    pub message:    String,
    pub campo:      &'static str,
    pub error_code: ErrorCodeIdInterno,
    pub severity:   Severity,
}

impl std::fmt::Display for IdInternoValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}][{}] {}: {}",
            self.severity, self.campo, self.error_code, self.message
        )
    }
}

impl std::error::Error for IdInternoValidationError {}

// ---------------------------------------------------------------------------
// Validador de String  →  Uuid
// ---------------------------------------------------------------------------

pub struct IdInternoStringValidador;

impl IdInternoStringValidador {

    /// Limpia el string, valida la longitud exacta y el formato de UUID,
    /// y delega la validación lógica a `IdInternoValidator`.
    pub fn validar(valor: impl AsRef<str>) -> Result<Uuid, IdInternoValidationError> {
        
        let raw = valor.as_ref();

        // ------------------------------------------------------------------
        // 1 Limpieza
        // ------------------------------------------------------------------
        let limpio = SpaceCleaner::limpiar(
            raw, 
            IdInternoConfig::LONGITUD_MAX_RUIDO, 
            true
        );

        if limpio.is_empty() {
            return Err(IdInternoValidationError {
                message:    "El identificador interno no puede estar vacío.".into(),
                campo:      IdInternoConfig::CAMPO,
                error_code: ErrorCodeIdInterno::IdInternoVacio,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 2 Validación de longitud exacta (36 caracteres)
        // ------------------------------------------------------------------
        if limpio.len() != IdInternoConfig::LONGITUD_UUID {
            return Err(IdInternoValidationError {
                message: format!(
                    "Longitud de UUID inválida. Se esperaban {} caracteres y se recibieron {}.",
                    IdInternoConfig::LONGITUD_UUID,
                    limpio.len()
                ),
                campo:      IdInternoConfig::CAMPO,
                error_code: ErrorCodeIdInterno::IdInternoLongitudInvalida,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 3 Parseo a Uuid
        // ------------------------------------------------------------------
        // Llegados aquí, la longitud es correcta, pero el contenido podría no ser hexadecimal
        let uuid_parseado = Uuid::parse_str(&limpio).map_err(|_| IdInternoValidationError {
            message: format!("El valor '{}' no tiene un formato hexadecimal de UUID válido.", limpio),
            campo:      IdInternoConfig::CAMPO,
            error_code: ErrorCodeIdInterno::IdInternoFormatoInvalido,
            severity:   Severity::Error,
        })?;

        // ------------------------------------------------------------------
        // 4 Validación lógica  →  delega al validador de Uuid
        // ------------------------------------------------------------------
        IdInternoValidator::validar(uuid_parseado)
    }
}

// ---------------------------------------------------------------------------
// Validador de Uuid  →  Uuid
// ---------------------------------------------------------------------------

pub struct IdInternoValidator;

impl IdInternoValidator {

    /// Realiza validaciones lógicas sobre una instancia de Uuid ya parseada.
    pub fn validar(valor: Uuid) -> Result<Uuid, IdInternoValidationError> {

        // ------------------------------------------------------------------
        // 1 UUID NIL
        // ------------------------------------------------------------------
        if !IdInternoConfig::PERMITIR_UUID_NIL && valor.is_nil() {
            return Err(IdInternoValidationError {
                message:    "El UUID no puede ser NIL (00000000-0000-0000-0000-000000000000).".into(),
                campo:      IdInternoConfig::CAMPO,
                error_code: ErrorCodeIdInterno::UuidNil,
                severity:   Severity::Error,
            });
        }

        // ------------------------------------------------------------------
        // 2 Validación de versión
        // ------------------------------------------------------------------
        if let Some(version_requerida) = IdInternoConfig::VERSION_REQUERIDA {
            if valor.get_version() != Some(version_requerida) {
                return Err(IdInternoValidationError {
                    message: format!(
                        "El UUID debe ser versión {:?} (Random/v4).",
                        version_requerida
                    ),
                    campo:      IdInternoConfig::CAMPO,
                    error_code: ErrorCodeIdInterno::UuidInvalidVersion,
                    severity:   Severity::Error,
                });
            }
        }

        Ok(valor)
    }
}



#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{BufReader, Write};

    use serde::Serialize;
    use serde_json::Value;
    use uuid::Uuid;
    
    use crate::base::validadores::base::id_interno::IdInternoValidator;

    #[derive(Serialize)]
    struct ValidacionResultado {
        entrada: Value,
        salida: Option<String>,
        valido: bool,
        error: Option<String>,
        codigo_error: Option<String>,
    }

    /// Ejecuta la validación y genera un reporte comparativo entre entrada y salida.
    fn ejecutar_validacion_desde_archivo(
        input_path: &str,
        output_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {

        let file = File::open(input_path)?;
        let reader = BufReader::new(file);
        let lista: Vec<Value> = serde_json::from_reader(reader)?;

        let mut resultados = Vec::with_capacity(lista.len());

        for item in lista {
            match item.as_str() {

                Some(texto_uuid) => {

                    // Paso 1: intentar parsear UUID
                    match Uuid::parse_str(texto_uuid) {

                        Ok(uuid) => {

                            // Paso 2: ejecutar validador
                            match IdInternoValidator::validar(uuid) {

                                Ok(uuid_validado) => {
                                    resultados.push(ValidacionResultado {
                                        entrada: item.clone(),
                                        salida: Some(uuid_validado.to_string()),
                                        valido: true,
                                        error: None,
                                        codigo_error: None,
                                    });
                                }

                                Err(e) => {
                                    resultados.push(ValidacionResultado {
                                        entrada: item.clone(),
                                        salida: None,
                                        valido: false,
                                        error: Some(e.message),
                                        codigo_error: Some(e.error_code.to_string()),
                                    });
                                }
                            }
                        }

                        Err(_) => {
                            resultados.push(ValidacionResultado {
                                entrada: item.clone(),
                                salida: None,
                                valido: false,
                                error: Some("Formato UUID inválido".into()),
                                codigo_error: Some("uuid_parse_error".into()),
                            });
                        }
                    }
                }

                None => {
                    resultados.push(ValidacionResultado {
                        entrada: item,
                        salida: None,
                        valido: false,
                        error: Some("El valor no es una cadena".into()),
                        codigo_error: Some("not_a_string".into()),
                    });
                }
            }
        }

        let json_salida = serde_json::to_string_pretty(&resultados)?;
        let mut file_out = File::create(output_path)?;
        file_out.write_all(json_salida.as_bytes())?;

        Ok(())
    }

    #[test]
    fn test_validador_id_interno_completo() {
        let resultado = ejecutar_validacion_desde_archivo(
            "uuids.json",
            "reporte_uuid.json",
        );

        assert!(
            resultado.is_ok(),
            "Error en el procesamiento: {:?}",
            resultado.err()
        );
    }
}