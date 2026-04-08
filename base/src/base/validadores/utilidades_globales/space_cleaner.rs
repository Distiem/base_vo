use once_cell::sync::Lazy;
use regex::Regex;
use std::borrow::Cow;
use unicode_normalization::UnicodeNormalization;

// ---------------------------------------------------------------------------
// Patrones estáticos (compilados una única vez al primer uso)
// ---------------------------------------------------------------------------

/// Caracteres de control y de formato que deben eliminarse del texto:
static PATRON_CARACTERES_CONTROL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"[\x00-\x08\x0B-\x0C\x0E-\x1F\x7F-\x9F\u202a-\u202e\u2066-\u2069]|\p{Cc}|\p{Cf}",
    )
    .expect("PATRON_CARACTERES_CONTROL: regex inválida")
});

/// Separadores de espacio y línea no estándar que se normalizan a un espacio:
static PATRON_SEPARADORES: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"[\n\r\t\x0C\x0B\u0085\u00A0\u1680\u2000-\u200A\u2028\u2029\u202F\u205F\u3000]+",
    )
    .expect("PATRON_SEPARADORES: regex inválida")
});

/// Caracteres de ancho cero y pegables que se eliminan silenciosamente:
static PATRON_PEGABLES: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\u00AD\u034F\u061C\u180E\u200B\u200C\u200D\u2060\uFEFF]+")
        .expect("PATRON_PEGABLES: regex inválida")
});

/// Uno o más espacios en blanco consecutivos (`\s+`).
static PATRON_MULTIPLES_ESPACIOS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\s+").expect("PATRON_MULTIPLES_ESPACIOS: regex inválida")
});

// ---------------------------------------------------------------------------
// Struct principal
// ---------------------------------------------------------------------------

/// Limpiador de texto Unicode.
pub struct SpaceCleaner;

impl SpaceCleaner {

    /// Limpia y normaliza un texto.
    /// - `texto`: cadena de entrada
    /// - `longitud_maxima`: límite para evitar procesar textos demasiado largos
    /// - `limpiar_todo`: si es true elimina todos los espacios
    pub fn limpiar<T: AsRef<str>>(
        texto: T,
        longitud_maxima: usize,
        limpiar_todo: bool,
    ) -> String {

        // Obtiene una referencia &str del valor recibido.
        let t: &str = texto.as_ref();

        // Si está vacío no hay nada que limpiar.
        if t.is_empty() {
            return String::new();
        }

        // Si excede el límite, se devuelve sin procesar.
        if t.len() > longitud_maxima {
            return t.to_string();
        }

        // --- Normalización Unicode (NFKC) ---
        let mut buffer: String = t.nfkc().collect();

        // --- Sustituciones con regex ---
        Self::aplicar_regex_inplace(&mut buffer, &PATRON_CARACTERES_CONTROL, " ");
        Self::aplicar_regex_inplace(&mut buffer, &PATRON_PEGABLES, " ");
        Self::aplicar_regex_inplace(&mut buffer, &PATRON_SEPARADORES, " ");

        // Manejo de espacios múltiples según el modo.
        if limpiar_todo {
            Self::aplicar_regex_inplace(&mut buffer, &PATRON_MULTIPLES_ESPACIOS, "");
        } else {
            Self::aplicar_regex_inplace(&mut buffer, &PATRON_MULTIPLES_ESPACIOS, " ");
        }

        // --- Resultado final ---
        if limpiar_todo {
            // No quedan espacios en este modo.
            buffer
        } else {
            // Elimina espacios al inicio y final.
            buffer.trim().to_string()
        }
    }

    /// Aplica un regex sobre el buffer sin copiar si no hay cambios.
    fn aplicar_regex_inplace(buffer: &mut String, re: &Regex, reemplazo: &str) {
        let resultado = re.replace_all(buffer, reemplazo);

        // Solo reemplaza si el regex produjo un nuevo String.
        if let Cow::Owned(nuevo_texto) = resultado {
            *buffer = nuevo_texto;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use std::fs::OpenOptions;
    use std::io::Write;

    const LONGITUD_MAXIMA_DEFAULT: usize = 1_000;

    #[derive(Serialize)]
    struct TestRegistro<'a> {
        entrada: &'a str,
        salida: String,
        longitud_maxima: usize,
        limpiar_todo: bool,
    }

    fn guardar(registro: &TestRegistro) {
        let json = serde_json::to_string(registro).unwrap();

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("space_cleaner_tests.json")
            .unwrap();

        writeln!(file, "{json}").unwrap();
    }

    #[test]
    fn colapsa_espacios_multiples() {
        let entrada = "  hola   mundo  ";

        let salida = SpaceCleaner::limpiar(
            entrada,
            LONGITUD_MAXIMA_DEFAULT,
            false
        );

        guardar(&TestRegistro {
            entrada,
            salida: salida.clone(),
            longitud_maxima: LONGITUD_MAXIMA_DEFAULT,
            limpiar_todo: false,
        });

        assert_eq!(salida, "hola mundo");
    }

    #[test]
    fn elimina_todos_los_espacios() {
        let entrada = "  hola   mundo  ";

        let salida = SpaceCleaner::limpiar(
            entrada,
            LONGITUD_MAXIMA_DEFAULT,
            true
        );

        guardar(&TestRegistro {
            entrada,
            salida: salida.clone(),
            longitud_maxima: LONGITUD_MAXIMA_DEFAULT,
            limpiar_todo: true,
        });

        assert_eq!(salida, "holamundo");
    }

    #[test]
    fn reemplaza_saltos_de_linea() {
        let entrada = "hola\n\t\rmundo";

        let salida = SpaceCleaner::limpiar(
            entrada,
            LONGITUD_MAXIMA_DEFAULT,
            false
        );

        guardar(&TestRegistro {
            entrada,
            salida: salida.clone(),
            longitud_maxima: LONGITUD_MAXIMA_DEFAULT,
            limpiar_todo: false,
        });

        assert_eq!(salida, "hola mundo");
    }

    #[test]
    fn normaliza_nfkc() {
        let entrada = "ﬁle";

        let salida = SpaceCleaner::limpiar(
            entrada,
            LONGITUD_MAXIMA_DEFAULT,
            false
        );

        guardar(&TestRegistro {
            entrada,
            salida: salida.clone(),
            longitud_maxima: LONGITUD_MAXIMA_DEFAULT,
            limpiar_todo: false,
        });

        assert_eq!(salida, "file");
    }
}