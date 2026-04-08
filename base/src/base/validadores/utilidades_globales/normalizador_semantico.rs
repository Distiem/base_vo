pub struct NormalizadorSemantico;

impl NormalizadorSemantico {
    pub fn clave_comparacion(valor: &str) -> String {
        if valor.is_empty() {
            return String::new();
        }

        valor
            .to_lowercase()
            .trim()
            // Paso 1: Normalización NFKD (separa caracteres de sus tildes)
            // Ejemplo: 'ó' se convierte en 'o' + '◌́'
            .chars()
            .map(|c| {
                match c {
                    'á' | 'à' | 'ä' | 'â' => 'a',
                    'é' | 'è' | 'ë' | 'ê' => 'e',
                    'í' | 'ì' | 'ï' | 'î' => 'i',
                    'ó' | 'ò' | 'ö' | 'ô' => 'o',
                    'ú' | 'ù' | 'ü' | 'û' => 'u',
                    'ñ' => 'n', // Opcional: según si quieres que 'año' y 'ano' sean iguales
                    _ => c,
                }
            })
            // Paso 2: Filtrado de ruidos (puntuación, símbolos, etc.)
            .filter(|c| c.is_alphanumeric())
            .collect()
    }

    /// Opcional: Una versión más simple que solo normaliza el caso y espacios,
    /// útil si quieres ser menos agresivo con la eliminación de caracteres.
    pub fn normalizar_basico(valor: &str) -> String {
        valor.trim().to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clave_comparacion_basica() {
        assert_eq!(NormalizadorSemantico::clave_comparacion("Acción"), "accion");
        assert_eq!(NormalizadorSemantico::clave_comparacion("  acción  "), "accion");
    }

    #[test]
    fn test_deteccion_duplicados_semanticos() {
        let genero1 = NormalizadorSemantico::clave_comparacion("Ciencia-Ficción");
        let genero2 = NormalizadorSemantico::clave_comparacion("ciencia ficcion");
        assert_eq!(genero1, genero2);
    }

    #[test]
    fn test_caracteres_especiales() {
        let clave = NormalizadorSemantico::clave_comparacion("¿Género?");
        assert_eq!(clave, "genero");
    }
}