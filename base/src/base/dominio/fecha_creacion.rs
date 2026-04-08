use chrono::{DateTime, Utc};
use std::fmt;
use std::str::FromStr;
use serde::{Serialize, Deserialize};

use crate::base::validadores::base::fecha_creacion::{
    FechaCreacionValidador,
    FechaCreacionStringValidador,
    FechaValidationError
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FechaCreacion(DateTime<Utc>);

impl FechaCreacion {

    // ---------------------------------------------------------
    // CONSTRUCTOR INTERNO
    // ---------------------------------------------------------

    /// Constructor interno que asegura el paso por el validador lógico.
    fn new(valor: DateTime<Utc>) -> Result<Self, FechaValidationError> {
        let validada = FechaCreacionValidador::validar(valor)?;
        Ok(Self(validada))
    }

    // ---------------------------------------------------------
    // FÁBRICAS
    // ---------------------------------------------------------

    /// Genera la fecha actual validada.
    pub fn ahora() -> Result<Self, FechaValidationError> {
        Self::new(Utc::now())
    }

    /// Crea una FechaCreacion desde un DateTime existente.
    pub fn desde_datetime(valor: DateTime<Utc>) -> Result<Self, FechaValidationError> {
        Self::new(valor)
    }

    /// Crea una FechaCreacion desde un string validado.
    pub fn desde_string(valor: impl AsRef<str>) -> Result<Self, FechaValidationError> {
        let parsed = FechaCreacionStringValidador::validar(valor)?;
        Ok(Self(parsed))
    }

    // ---------------------------------------------------------
    // API DE DOMINIO
    // ---------------------------------------------------------

    /// Devuelve el DateTime interno.
    pub fn valor(&self) -> DateTime<Utc> {
        self.0
    }

    /// Devuelve la representación RFC3339.
    pub fn to_rfc3339(&self) -> String {
        self.0.to_rfc3339()
    }
}

// ---------------------------------------------------------------------------
// IMPLEMENTACIÓN DE TRAITS DE CONVERSIÓN
// ---------------------------------------------------------------------------

impl FromStr for FechaCreacion {
    type Err = FechaValidationError;

    /// Permite crear una FechaCreacion usando `"texto".parse()`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::desde_string(s)
    }
}

impl TryFrom<DateTime<Utc>> for FechaCreacion {
    type Error = FechaValidationError;

    /// Permite convertir desde DateTime usando try_from / try_into.
    fn try_from(value: DateTime<Utc>) -> Result<Self, Self::Error> {
        Self::desde_datetime(value)
    }
}

impl TryFrom<&str> for FechaCreacion {
    type Error = FechaValidationError;

    /// Permite convertir desde &str usando try_from / try_into.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::desde_string(value)
    }
}

impl TryFrom<String> for FechaCreacion {
    type Error = FechaValidationError;

    /// Permite convertir desde String usando try_from / try_into.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::desde_string(value)
    }
}

// ---------------------------------------------------------------------------
// PRESENTACIÓN
// ---------------------------------------------------------------------------

impl fmt::Display for FechaCreacion {

    /// Representación estándar RFC3339.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_rfc3339())
    }
}

#[cfg(test)]
mod tests2 {
    use super::*;
    use serde::Serialize;
    use std::fs;

    #[derive(Serialize)]
    struct ProcesamientoResultado {
        entrada: String,
        // Usamos Option para representar éxito o falla de forma clara en el JSON
        salida: Option<FechaCreacion>,
        error: Option<String>,
        valido: bool,
    }

    pub fn procesar_fechas_json(json_entrada: &str) -> String {
        // 1. Deserializar el array de strings de entrada
        let entradas: Vec<String> = serde_json::from_str(json_entrada)
            .expect("El JSON de entrada no tiene el formato esperado (Array de strings)");

        // 2. Mapear cada entrada usando tu lógica de negocio
        let resultados: Vec<ProcesamientoResultado> = entradas
            .into_iter()
            .map(|raw_str| {
                match FechaCreacion::desde_string(&raw_str) {
                    Ok(fecha) => ProcesamientoResultado {
                        entrada: raw_str,
                        salida: Some(fecha),
                        error: None,
                        valido: true,
                    },
                    Err(e) => ProcesamientoResultado {
                        entrada: raw_str,
                        salida: None,
                        error: Some(e.to_string()),
                        valido: false,
                    },
                }
            })
            .collect();

        // 3. Serializar a JSON con formato legible (pretty print)
        serde_json::to_string_pretty(&resultados).unwrap()
    }

    #[test]
    fn test_procesar_fechas_json_desde_archivo() {
        // Cargar archivo JSON real
        let json_entrada =
            fs::read_to_string("archivos_json/entrada/fecha_creacion.json")
            .expect("No se pudo leer el archivo JSON");

        let resultado = procesar_fechas_json(&json_entrada);
           
        fs::write("resultado_fecha_creacion.json", &resultado)
            .expect("No se pudo escribir el archivo resultado");
    }
}









#[cfg(test)]
mod tests3 {
    use super::*;
    use serde::Serialize;
    use serde_json::Value;
    use std::fs;

    #[derive(Serialize)]
    struct ProcesamientoResultado {
        entrada: Value,
        salida: Option<FechaCreacion>,
        error: Option<String>,
        valido: bool,
    }
    
    pub fn procesar_fechas_json(json_entrada: &str) -> String {
        // 1. Intentar deserializar cualquier JSON
        let entradas: Vec<Value> = match serde_json::from_str(json_entrada) {
            Ok(v) => v,
            Err(e) => {
                return format!(
                    r#"[{{"entrada": null, "salida": null, "error": "JSON inválido: {}", "valido": false}}]"#,
                    e
                )
            }
        };

        // 2. Procesar cada valor
        let resultados: Vec<ProcesamientoResultado> = entradas
            .into_iter()
            .map(|valor| {
                match valor.as_str() {
                    Some(texto) => {
                        match FechaCreacion::desde_string(texto) {
                            Ok(fecha) => ProcesamientoResultado {
                                entrada: Value::String(texto.to_string()),
                                salida: Some(fecha),
                                error: None,
                                valido: true,
                            },
                            Err(e) => ProcesamientoResultado {
                                entrada: Value::String(texto.to_string()),
                                salida: None,
                                error: Some(e.to_string()),
                                valido: false,
                            },
                        }
                    }
                    None => ProcesamientoResultado {
                        entrada: valor,
                        salida: None,
                        error: Some(
                            "[ERROR][fecha_creacion] tipo_invalido: Se esperaba un string."
                                .to_string(),
                        ),
                        valido: false,
                    },
                }
            })
            .collect();

        // 3. Serializar resultado
        serde_json::to_string_pretty(&resultados).unwrap()
    }

    #[test]
    fn test_procesar_fechas_json_desde_archivo() {
        // Leer archivo JSON real
        let json_entrada = fs::read_to_string("archivos_json/entrada/fecha.json")
            .expect("No se pudo leer el archivo JSON");

        // Procesar fechas
        let resultado = procesar_fechas_json(&json_entrada);

        // Guardar salida para inspección
        fs::write("resultado_fecha.json", &resultado)
            .expect("No se pudo escribir el archivo resultado");

        // Verificación básica
        assert!(!resultado.is_empty());
    }
}


//sin uso?
use serde_json::Value;

#[derive(Serialize)]
struct ProcesamientoResultado {
    entrada: Value,
    salida: Option<FechaCreacion>,
    error: Option<String>,
    valido: bool,
}

pub fn procesar_fechas_json(json_entrada: &str) -> String {
    // 1. Aceptar cualquier JSON
    let entradas: Vec<Value> = match serde_json::from_str(json_entrada) {
        Ok(v) => v,
        Err(e) => {
            return format!(
                r#"[{{"entrada": null, "salida": null, "error": "JSON inválido: {}", "valido": false}}]"#,
                e
            )
        }
    };

    // 2. Procesar cada elemento
    let resultados: Vec<ProcesamientoResultado> = entradas
        .into_iter()
        .map(|valor| {
            match valor.as_str() {
                Some(texto) => {
                    match FechaCreacion::desde_string(texto) {
                        Ok(fecha) => ProcesamientoResultado {
                            entrada: Value::String(texto.to_string()),
                            salida: Some(fecha),
                            error: None,
                            valido: true,
                        },
                        Err(e) => ProcesamientoResultado {
                            entrada: Value::String(texto.to_string()),
                            salida: None,
                            error: Some(e.to_string()),
                            valido: false,
                        },
                    }
                }
                None => ProcesamientoResultado {
                    entrada: valor,
                    salida: None,
                    error: Some(
                        "[ERROR][fecha_creacion] tipo_invalido: Se esperaba un string.".to_string()
                    ),
                    valido: false,
                },
            }
        })
        .collect();

    // 3. Serializar salida
    serde_json::to_string_pretty(&resultados).unwrap()
}