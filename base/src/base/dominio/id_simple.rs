use std::fmt;
use std::str::FromStr;
use serde::{Serialize, Deserialize};

use crate::base::validadores::base::id_simple::{
    IdSimpleValidador, 
    IdSimpleStringValidador, 
    IdSimpleValidationError, 
};

// =========================================================
// --- ID SIMPLE (Value Object) ---
// =========================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IdSimple(Option<i64>);

impl IdSimple {
    
    // ---------------------------------------------------------
    // CONSTRUCTOR INTERNO
    // ---------------------------------------------------------

    /// Constructor interno que asegura el paso por el validador lógico.
    fn new(valor: i64) -> Result<Self, IdSimpleValidationError> {
        let valor_validado = IdSimpleValidador::validar(valor)?;
        Ok(Self(Some(valor_validado)))
    }

    // ---------------------------------------------------------
    // FÁBRICAS
    // ---------------------------------------------------------

    /// Crea un IdSimple desde un i64 existente.
    pub fn desde_i64(valor: i64) -> Result<Self, IdSimpleValidationError> {
        Self::new(valor)
    }

    /// Crea un IdSimple opcional desde Option<i64>.
    pub fn desde_option(valor: Option<i64>) -> Result<Self, IdSimpleValidationError> {
        match valor {
            Some(v) => Self::new(v),
            None => Ok(Self::sin_valor()),
        }
    }

    /// Crea un IdSimple desde un string validado.
    pub fn desde_string(valor: &str) -> Result<Self, IdSimpleValidationError> {
        if valor.trim().is_empty() {
            return Ok(Self::sin_valor());
        }

        let parsed = IdSimpleStringValidador::validar(valor)?;
        Ok(Self(Some(parsed)))
    }

    /// Constructor explícito para ausencia de ID.
    pub fn sin_valor() -> Self {
        Self(None)
    }

    // ---------------------------------------------------------
    // GETTERS & UTILIDADES
    // ---------------------------------------------------------
    
    pub fn valor(&self) -> Option<i64> {
        self.0
    }

    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }

    pub fn unwrap(self) -> i64 {
        self.0.expect("Error: Intento de acceso a un IdSimple sin valor.")
    }

    pub fn unwrap_or(self, default: i64) -> i64 {
        self.0.unwrap_or(default)
    }
}

// ---------------------------------------------------------------------------
// IMPLEMENTACIÓN DE TRAITS DE CONVERSIÓN
// ---------------------------------------------------------------------------

impl FromStr for IdSimple {
    type Err = IdSimpleValidationError;

    /// Permite crear un IdSimple usando `"texto".parse()`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::desde_string(s)
    }
}

impl TryFrom<&str> for IdSimple {
    type Error = IdSimpleValidationError;

    /// Permite convertir desde &str usando try_from / try_into.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::desde_string(value)
    }
}

impl TryFrom<i64> for IdSimple {
    type Error = IdSimpleValidationError;

    /// Permite convertir desde i64 usando try_from / try_into.
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::desde_i64(value)
    }
}

// ---------------------------------------------------------------------------
// PRESENTACIÓN
// ---------------------------------------------------------------------------

impl fmt::Display for IdSimple {
    /// Representación estándar del identificador simple.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(v) => write!(f, "{v}"),
            None => write!(f, "None"),
        }
    }
}




use std::fs::File;
use std::io::BufReader;
use std::path::Path;

// Estructura intermedia para deserializar el JSON crudo
#[derive(Deserialize)]
struct RawInput {
    id_usuario: String,
    id_proyecto: String,
}

/// Función que lee un JSON y transforma sus campos usando IdSimple::from_str
pub fn cargar_ids_desde_json<P: AsRef<Path>>(ruta: P) -> Result<Vec<IdSimple>, String> {
    // 1. Abrir el archivo con un BufReader para mayor eficiencia en lectura
    let archivo = File::open(ruta)
        .map_err(|e| format!("No se pudo abrir el archivo: {}", e))?;
    let lector = BufReader::new(archivo);

    // 2. Deserializar el JSON a nuestra estructura de strings crudos
    // Aquí es donde Serde fallaría si le envías una lista [] o un bool true
    // porque RawInput espera Strings.
    let datos: RawInput = serde_json::from_reader(lector)
        .map_err(|e| format!("Error en el formato JSON: {}", e))?;

    // 3. Procesar cada string usando tu implementación de FromStr
    let id_u = IdSimple::from_str(&datos.id_usuario)
        .map_err(|e| format!("Error en id_usuario: {}", e.message))?;
    
    let id_p = IdSimple::from_str(&datos.id_proyecto)
        .map_err(|e| format!("Error en id_proyecto: {}", e.message))?;

    Ok(vec![id_u, id_p])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    // Necesitamos que IdSimple sea Serializable para guardarlo en un JSON
    // Si no lo tienes, añade #[derive(Serialize)] a su definición
    
    #[test]
    fn test_procesar_y_guardar_json() {
        // 1. Crear un archivo JSON de prueba (Entrada)
        let ruta_entrada = "test_input.json";
        let contenido_input = r#"
        {
            "id_usuario": "USR-123",
            "id_proyecto": "PRJ-456"
        }
        "#;
        let mut file_in = File::create(ruta_entrada).unwrap();
        file_in.write_all(contenido_input.as_bytes()).unwrap();

        // 2. Llamar a tu función para cargar los IDs
        let ids = cargar_ids_desde_json(ruta_entrada).expect("Debería cargar los IDs correctamente");

        // 3. Guardar la salida en otro archivo JSON (Salida)
        let ruta_salida = "test_output.json";
        let archivo_salida = File::create(ruta_salida).expect("No se pudo crear archivo de salida");
        
        // Usamos serde_json para serializar el vector de IdSimple
        serde_json::to_writer_pretty(archivo_salida, &ids)
            .expect("No se pudo escribir el JSON de salida");

        // 4. Verificaciones (Assertions)
        assert_eq!(ids.len(), 2);
        
        // Limpieza opcional de los archivos creados
        let _ = std::fs::remove_file(ruta_entrada);
        let _ = std::fs::remove_file(ruta_salida);
    }
}