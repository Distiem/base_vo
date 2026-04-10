use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::base::validadores::base::id_interno::{
    IdInternoStringValidador,
    IdInternoValidationError,
    IdInternoValidator,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IdInterno(Uuid);

impl IdInterno {

    // ---------------------------------------------------------
    // CONSTRUCTORES
    // ---------------------------------------------------------

    /// Constructor privado que asegura el paso por el validador lógico.
    fn new(valor: Uuid) -> Result<Self, IdInternoValidationError> {
        let valor_validado = IdInternoValidator::validar(valor)?;
        Ok(Self(valor_validado))
    }

    /// Crea un IdInterno desde un objeto Uuid ya existente.
    pub fn desde_uuid(valor: Uuid) -> Result<Self, IdInternoValidationError> {
        Self::new(valor)
    }

    /// Crea un IdInterno a partir de un string validado.
    pub fn desde_str(valor: &str) -> Result<Self, IdInternoValidationError> {
        let uuid_validado = IdInternoStringValidador::validar(valor)?;
        Ok(Self(uuid_validado))
    }

    /// Genera un nuevo IdInterno (UUID v4) válido automáticamente.
    pub fn generar() -> Self {
        Self(Uuid::new_v4())
    }
    
    /// Crea un IdInterno a partir de un String validado.
    pub fn desde_string(valor: String) -> Result<Self, IdInternoValidationError> {
        let uuid_validado = IdInternoStringValidador::validar(&valor)?;
        Ok(Self(uuid_validado))
    }

    // ---------------------------------------------------------
    // GETTERS
    // ---------------------------------------------------------

    /// Devuelve el Uuid interno.
    pub fn valor(&self) -> Uuid {
        self.0
    }

    /// Devuelve la representación en cadena estándar (8-4-4-4-12).
    pub fn to_string_canonical(&self) -> String {
        self.0.hyphenated().to_string()
    }
}

// ---------------------------------------------------------------------------
// IMPLEMENTACIÓN DE TRAITS DE CONVERSIÓN
// ---------------------------------------------------------------------------

impl FromStr for IdInterno {
    type Err = IdInternoValidationError;

    /// Permite crear un IdInterno usando `"texto".parse()`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::desde_str(s)
    }
}

impl TryFrom<Uuid> for IdInterno {
    type Error = IdInternoValidationError;

    /// Permite convertir desde Uuid usando try_from / try_into.
    fn try_from(value: Uuid) -> Result<Self, Self::Error> {
        Self::desde_uuid(value)
    }
}

impl TryFrom<&str> for IdInterno {
    type Error = IdInternoValidationError;

    /// Permite convertir desde &str usando try_from / try_into.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::desde_str(value)
    }
}

// ---------------------------------------------------------------------------
// PRESENTACIÓN
// ---------------------------------------------------------------------------

impl fmt::Display for IdInterno {

    /// Representación estándar del UUID en formato canonical.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.hyphenated())
    }
}

//Falla con cualquier cosa que no sea str
#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use std::fs;

    #[derive(Serialize)]
    struct ProcesamientoResultado {
        entrada: String,
        // Representa éxito o falla claramente en el JSON
        salida: Option<IdInterno>,
        error: Option<String>,
        valido: bool,
    }

    pub fn procesar_ids_json(json_entrada: &str) -> String {
        // 1. Deserializar el array de strings de entrada
        let entradas: Vec<String> = serde_json::from_str(json_entrada)
            .expect("El JSON de entrada no tiene el formato esperado (Array de strings)");

        // 2. Procesar cada entrada usando la lógica de dominio
        let resultados: Vec<ProcesamientoResultado> = entradas
            .into_iter()
            .map(|raw_str| {
                match IdInterno::desde_str(&raw_str) {
                    Ok(id) => ProcesamientoResultado {
                        entrada: raw_str,
                        salida: Some(id),
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

        // 3. Serializar a JSON legible
        serde_json::to_string_pretty(&resultados).unwrap()
    }

    #[test]
    fn test_procesar_ids_json_desde_archivo() {

        // Cargar archivo JSON real
        let json_entrada =
            fs::read_to_string("archivos_json/entrada/uuids.json")
            .expect("No se pudo leer el archivo JSON");

        let resultado = procesar_ids_json(&json_entrada);

        // Guardar resultado
        fs::write("id_interno.json", &resultado)
            .expect("No se pudo escribir el archivo resultado");
    }
}