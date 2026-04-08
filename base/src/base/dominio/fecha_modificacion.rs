use chrono::{DateTime, Utc};
use std::fmt;
use serde::{Serialize, Deserialize};

use crate::base::dominio::fecha_creacion::FechaCreacion;

use crate::base::validadores::base::fecha_modificacion::{
    FechaModificacionValidador,
    FechaModificacionStringValidador,
    FechaModificacionValidationError
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FechaModificacion(DateTime<Utc>);

impl FechaModificacion {

    // ---------------------------------------------------------
    // CONSTRUCTOR INTERNO
    // ---------------------------------------------------------

    /// Constructor interno que asegura el paso por el validador lógico.
    /// Recibe la fecha de creación para asegurar que la modificación no sea anterior.
    fn new(valor: DateTime<Utc>, fecha_creacion: FechaCreacion) -> Result<Self, FechaModificacionValidationError> {
        let validada = FechaModificacionValidador::validar(valor, fecha_creacion)?;
        Ok(Self(validada))
    }

    // ---------------------------------------------------------
    // FÁBRICAS
    // ---------------------------------------------------------

    /// Genera la fecha de modificación actual validada contra la de creación.
    pub fn ahora(fecha_creacion: FechaCreacion) -> Result<Self, FechaModificacionValidationError> {
        Self::new(Utc::now(), fecha_creacion)
    }

    /// Crea una FechaModificacion desde un DateTime existente.
    pub fn desde_datetime(
        valor: DateTime<Utc>, 
        fecha_creacion: FechaCreacion
    ) -> Result<Self, FechaModificacionValidationError> {
        Self::new(valor, fecha_creacion)
    }

    /// Crea una FechaModificacion desde un string validado.
    pub fn desde_string(
        valor: impl AsRef<str>, 
        fecha_creacion: FechaCreacion
    ) -> Result<Self, FechaModificacionValidationError> {
        let parsed = FechaModificacionStringValidador::validar(valor, fecha_creacion)?;
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
// PRESENTACIÓN
// ---------------------------------------------------------------------------

impl fmt::Display for FechaModificacion {
    /// Representación estándar RFC3339.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_rfc3339())
    }
}


#[cfg(test)]
mod tests2 {
    use super::*;
    use serde::{Serialize, Deserialize};
    use std::fs;

    /// Estructura para mapear los objetos del JSON de entrada.
    #[derive(Deserialize)]
    struct EntradaFecha {
        fecha_creacion: String,
        fecha_modificacion: String,
    }

    /// Estructura de salida detallada para el reporte JSON.
    #[derive(Serialize)]
    struct ProcesamientoResultado {
        entrada_creacion: String,
        entrada_modificacion: String,
        fecha_modificacion_resultante: Option<FechaModificacion>,
        error: Option<String>,
        valido: bool,
    }

    /// Procesa un JSON que contiene pares de fechas de creación y modificación.
    pub fn procesar_validaciones_modificacion(json_entrada: &str) -> String {
        // 1. Deserializar la lista de pares de entrada
        let entradas: Vec<EntradaFecha> = serde_json::from_str(json_entrada)
            .expect("El formato del JSON de entrada es inválido (Array de objetos)");

        // 2. Ejecutar la lógica de validación de dominio para cada par
        let resultados: Vec<ProcesamientoResultado> = entradas
            .into_iter()
            .map(|item| {
                // Paso A: Intentar crear la FechaCreacion (requisito para validar la modificación)
                let res_creacion = FechaCreacion::desde_string(&item.fecha_creacion);

                match res_creacion {
                    Ok(f_creacion) => {
                        // Paso B: Intentar crear la FechaModificacion usando la f_creacion obtenida
                        match FechaModificacion::desde_string(&item.fecha_modificacion, f_creacion) {
                            Ok(f_modificacion) => ProcesamientoResultado {
                                entrada_creacion: item.fecha_creacion,
                                entrada_modificacion: item.fecha_modificacion,
                                fecha_modificacion_resultante: Some(f_modificacion),
                                error: None,
                                valido: true,
                            },
                            Err(e) => ProcesamientoResultado {
                                entrada_creacion: item.fecha_creacion,
                                entrada_modificacion: item.fecha_modificacion,
                                fecha_modificacion_resultante: None,
                                error: Some(format!("Error de Modificación: {}", e)),
                                valido: false,
                            },
                        }
                    }
                    Err(e) => ProcesamientoResultado {
                        entrada_creacion: item.fecha_creacion,
                        entrada_modificacion: item.fecha_modificacion,
                        fecha_modificacion_resultante: None,
                        error: Some(format!("Error en FechaCreacion base: {}", e)),
                        valido: false,
                    },
                }
            })
            .collect();

        // 3. Serializar a JSON con formato legible (Pretty Print)
        serde_json::to_string_pretty(&resultados).unwrap()
    }

    #[test]
    fn test_procesar_validaciones_desde_archivo() {
        // Asegúrate de que esta ruta existe en tu proyecto
        let ruta_entrada = "archivos_json/entrada/fecha_modificacion.json";
        
        let json_entrada = fs::read_to_string(ruta_entrada)
            .unwrap_or_else(|_| {
                // Fallback: Si el archivo no existe, podrías usar un string literal 
                // para que el test no falle en entornos de CI sin archivos.
                "[]".to_string()
            });

        if json_entrada == "[]" {
            println!("Advertencia: No se encontró el archivo de entrada o está vacío.");
            return;
        }

        let resultado_json = procesar_validaciones_modificacion(&json_entrada);
        
        // Guardar el reporte de resultados
        fs::write("resultado_validacion_modificacion.json", &resultado_json)
            .expect("No se pudo escribir el archivo de resultados");
            
        println!("Procesamiento completado. Revisa 'resultado_validacion_modificacion.json'");
    }
}