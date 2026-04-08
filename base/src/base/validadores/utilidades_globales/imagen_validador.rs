use image::{GenericImageView, ImageError};
use infer;
use std::fmt;
use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;
use serde::Serialize;

// ================= ERRORES =================

/// Error de validación con mensaje y detalles opcionales clave-valor.
#[derive(Debug)]
pub struct ImagenValidationError {
    pub message: String,
    pub details: std::collections::HashMap<String, String>,
}

impl ImagenValidationError {
    /// Crea un error simple sin detalles.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            details: std::collections::HashMap::new(),
        }
    }

    /// Crea un error con contexto adicional en pares clave-valor.
    pub fn with_details(
        message: impl Into<String>,
        details: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        Self {
            message: message.into(),
            details: details
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        }
    }
}

impl fmt::Display for ImagenValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.details.is_empty() {
            write!(f, "{}", self.message)
        } else {
            let details: Vec<String> = self
                .details
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect();
            write!(f, "{} | {{{}}}", self.message, details.join(", "))
        }
    }
}

impl std::error::Error for ImagenValidationError {}

// ================= CONFIGURACIÓN =================

/// Configuración completa del validador: formatos, tamaños, dimensiones y reglas opcionales.
#[derive(Debug, Clone)]
pub struct ImagenConfig {
    pub extensiones_permitidas: Vec<String>,
    pub mimes_permitidos: Vec<String>,

    pub max_bytes: usize,
    pub min_bytes: usize,

    pub ancho_max: u32,
    pub alto_max: u32,
    pub ancho_min: u32,
    pub alto_min: u32,

    /// Límite de píxeles totales para prevenir ataques DoS con imágenes gigantes.
    pub max_image_pixels: u64,
    /// Si `true`, rechaza imágenes animadas (GIF u otros formatos con múltiples frames).
    pub validar_animacion: bool,

    pub validar_aspect_ratio: bool,
    pub aspect_ratio_min: f64,
    pub aspect_ratio_max: f64,

    /// Si `true`, rechaza imágenes en escala de grises o monocromáticas.
    pub rechazar_monocromo: bool,
}

impl Default for ImagenConfig {
    fn default() -> Self {
        Self {
            extensiones_permitidas: vec![
                "jpg".into(),
                "jpeg".into(),
                "png".into(),
                "webp".into(),
            ],
            mimes_permitidos: vec![
                "image/jpeg".into(),
                "image/jpg".into(),
                "image/png".into(),
                "image/webp".into(),
            ],
            max_bytes: 20 * 1024 * 1024, // 10 MB
            min_bytes: 1024,             // 1 KB
            ancho_max: 8000,
            alto_max: 8000,
            ancho_min: 100,
            alto_min: 100,
            max_image_pixels: 30_000_000,
            validar_animacion: true,
            validar_aspect_ratio: false,
            aspect_ratio_min: 0.2,
            aspect_ratio_max: 5.0,
            rechazar_monocromo: false,
        }
    }
}

impl ImagenConfig {
    /// Verifica coherencia interna de la configuración antes de usarla.
    pub fn validate(&self) -> Result<(), ImagenValidationError> {
        if self.max_bytes <= self.min_bytes {
            return Err(ImagenValidationError::new(
                "max_bytes debe ser mayor que min_bytes",
            ));
        }
        if self.ancho_min >= self.ancho_max || self.alto_min >= self.alto_max {
            return Err(ImagenValidationError::new("Dimensiones min/max inválidas"));
        }
        if self.validar_aspect_ratio && self.aspect_ratio_min >= self.aspect_ratio_max {
            return Err(ImagenValidationError::new(
                "aspect_ratio_min debe ser menor que aspect_ratio_max",
            ));
        }
        Ok(())
    }
}

// ================= METADATA DE RESULTADO =================

/// Metadatos extraídos de una imagen que pasó todas las validaciones.
#[derive(Debug, Clone, Serialize)]
pub struct ImagenMetadata {
    pub width: u32,
    pub height: u32,
    pub mime: String,
    pub extension: String,
    pub bytes: usize,
    pub size_formatted: String,
    pub format: String,
    pub mode: String,
    pub aspect_ratio: f64,
    pub total_pixels: u64,
    pub megapixels: f64,
}

// ================= UTILIDADES =================

/// Convierte bytes a unidad legible (KB, MB, etc.).
pub fn formatear_bytes(n: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut value = n as f64;
    let mut unit = UNITS[0];
    for u in &UNITS[1..] {
        if value < 1024.0 {
            break;
        }
        value /= 1024.0;
        unit = u;
    }
    format!("{:.1} {}", value, unit)
}

/// Extrae y normaliza la extensión del nombre de archivo.
pub fn obtener_extension(nombre: &str) -> Option<String> {
    Path::new(nombre)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
}

// ================= VALIDADOR PRINCIPAL =================

/// Valida imágenes según la configuración provista.
pub struct ImagenValidator {
    pub config: ImagenConfig,
}

impl Default for ImagenValidator {
    fn default() -> Self {
        Self {
            config: ImagenConfig::default(),
        }
    }
}

impl ImagenValidator {
    /// Crea un validador verificando que la configuración sea coherente.
    pub fn new(config: ImagenConfig) -> Result<Self, ImagenValidationError> {
        config.validate()?;
        Ok(Self { config })
    }

    /// Verifica que el nombre tenga una extensión permitida. Retorna la extensión en minúsculas.
    fn validar_nombre_extension(&self, nombre: &str) -> Result<String, ImagenValidationError> {
        if nombre.trim().is_empty() {
            return Err(ImagenValidationError::new("Nombre de archivo vacío"));
        }

        let ext = obtener_extension(nombre).unwrap_or_default();

        if ext.is_empty() || !self.config.extensiones_permitidas.contains(&ext) {
            return Err(ImagenValidationError::with_details(
                format!("Extensión no permitida: '{}'", ext),
                [("permitidas", self.config.extensiones_permitidas.join(", "))],
            ));
        }

        Ok(ext)
    }

    /// Detecta el MIME real leyendo solo los primeros 2 KB del archivo.
    fn validar_mime_rapido<R: Read>(&self, reader: &mut R) -> Result<String, ImagenValidationError> {
        let mut header = vec![0u8; 2048];
        let n = reader
            .read(&mut header)
            .map_err(|e| ImagenValidationError::new(format!("Error leyendo header: {}", e)))?;
        header.truncate(n);

        let kind = infer::get(&header)
            .ok_or_else(|| ImagenValidationError::new("Formato de archivo no identificable"))?;

        let mime = kind.mime_type().to_string();

        if mime == "image/gif" {
            return Err(ImagenValidationError::new("GIF no permitido"));
        }

        if !self.config.mimes_permitidos.contains(&mime) {
            return Err(ImagenValidationError::with_details(
                format!("MIME no soportado: '{}'", mime),
                [("permitidos", self.config.mimes_permitidos.join(", "))],
            ));
        }

        Ok(mime)
    }

    /// Verifica que el tamaño del archivo esté dentro del rango configurado.
    fn validar_tamano(&self, size: usize) -> Result<(), ImagenValidationError> {
        if size < self.config.min_bytes {
            return Err(ImagenValidationError::with_details(
                format!("Archivo muy pequeño: {}", formatear_bytes(size)),
                [("min", formatear_bytes(self.config.min_bytes))],
            ));
        }
        if size > self.config.max_bytes {
            return Err(ImagenValidationError::with_details(
                format!("Archivo muy grande: {}", formatear_bytes(size)),
                [("max", formatear_bytes(self.config.max_bytes))],
            ));
        }
        Ok(())
    }

    /// Decodifica la imagen completa para verificar integridad, píxeles totales, formato y modo de color.
    fn validar_integridad_profunda(
        &self,
        data: &[u8],
    ) -> Result<(u32, u32, String, String), ImagenValidationError> {
        let img = image::load_from_memory(data).map_err(|e| match e {
            ImageError::IoError(_) => ImagenValidationError::new("Imagen corrupta o inválida"),
            ImageError::Unsupported(_) => ImagenValidationError::new("Formato de imagen no soportado"),
            _ => ImagenValidationError::new(format!("Error decodificando imagen: {}", e)),
        })?;

        let (ancho, alto) = img.dimensions();
        let total_pixels = ancho as u64 * alto as u64;

        if total_pixels > self.config.max_image_pixels {
            return Err(ImagenValidationError::with_details(
                format!("Imagen excede el límite de píxeles: {}", total_pixels),
                [("max", self.config.max_image_pixels.to_string())],
            ));
        }

        let formato = image::guess_format(data)
            .map(|f| format!("{:?}", f))
            .unwrap_or_else(|_| "Unknown".into());

        let modo = match img.color() {
            image::ColorType::L8 | image::ColorType::L16 => "L",
            image::ColorType::La8 | image::ColorType::La16 => "LA",
            image::ColorType::Rgb8 | image::ColorType::Rgb16 | image::ColorType::Rgb32F => "RGB",
            image::ColorType::Rgba8 | image::ColorType::Rgba16 | image::ColorType::Rgba32F => "RGBA",
            _ => "Other",
        }
        .to_string();

        Ok((ancho, alto, formato, modo))
    }

    /// Comprueba que ancho y alto estén dentro de los límites configurados.
    fn validar_dimensiones(&self, ancho: u32, alto: u32) -> Result<(), ImagenValidationError> {
        if ancho > self.config.ancho_max || alto > self.config.alto_max {
            return Err(ImagenValidationError::with_details(
                format!("Dimensiones excedidas: {}x{}", ancho, alto),
                [("max", format!("{}x{}", self.config.ancho_max, self.config.alto_max))],
            ));
        }
        if ancho < self.config.ancho_min || alto < self.config.alto_min {
            return Err(ImagenValidationError::with_details(
                format!("Dimensiones insuficientes: {}x{}", ancho, alto),
                [("min", format!("{}x{}", self.config.ancho_min, self.config.alto_min))],
            ));
        }
        Ok(())
    }

    /// Valida que la proporción ancho/alto esté en el rango permitido. No-op si está desactivado.
    fn validar_aspect_ratio(&self, ancho: u32, alto: u32) -> Result<(), ImagenValidationError> {
        if !self.config.validar_aspect_ratio {
            return Ok(());
        }
        let ratio = ancho as f64 / alto as f64;
        if !(self.config.aspect_ratio_min..=self.config.aspect_ratio_max).contains(&ratio) {
            return Err(ImagenValidationError::with_details(
                format!("Aspect ratio inválido: {:.2}", ratio),
                [(
                    "rango",
                    format!("{:.2}–{:.2}", self.config.aspect_ratio_min, self.config.aspect_ratio_max),
                )],
            ));
        }
        Ok(())
    }

    /// Rechaza imágenes en escala de grises si `rechazar_monocromo` está activo.
    fn validar_modo_color(&self, modo: &str) -> Result<(), ImagenValidationError> {
        if self.config.rechazar_monocromo && (modo == "1" || modo == "L") {
            return Err(ImagenValidationError::with_details(
                "Imagen monocromática no permitida",
                [("modo", modo.to_string())],
            ));
        }
        Ok(())
    }

    /// Punto de entrada principal: valida una imagen desde bytes en memoria.
    ///
    /// Ejecuta en orden: extensión → MIME → tamaño → integridad → dimensiones → opcionales.
    pub fn validar_bytes(
        &self,
        data: &[u8],
        filename: &str,
    ) -> Result<ImagenMetadata, ImagenValidationError> {
        if data.is_empty() {
            return Err(ImagenValidationError::new("Archivo no proporcionado"));
        }

        let ext = self.validar_nombre_extension(filename)?;
        let mime = self.validar_mime_rapido(&mut Cursor::new(data))?;
        self.validar_tamano(data.len())?;

        let (ancho, alto, formato, modo) = self.validar_integridad_profunda(data)?;

        self.validar_dimensiones(ancho, alto)?;
        self.validar_aspect_ratio(ancho, alto)?;
        self.validar_modo_color(&modo)?;

        Ok(ImagenMetadata {
            width: ancho,
            height: alto,
            mime,
            extension: ext,
            bytes: data.len(),
            size_formatted: formatear_bytes(data.len()),
            format: formato,
            mode: modo,
            aspect_ratio: (ancho as f64 / alto as f64 * 100.0).round() / 100.0,
            total_pixels: ancho as u64 * alto as u64,
            megapixels: (ancho as f64 * alto as f64 / 1_000_000.0 * 10.0).round() / 10.0,
        })
    }

    /// Lee el archivo de disco y delega en `validar_bytes`. Maneja errores de I/O y permisos.
    pub fn validar_desde_ruta(&self, ruta: &str) -> Result<ImagenMetadata, ImagenValidationError> {
        let path = Path::new(ruta);

        if !path.exists() {
            return Err(ImagenValidationError::new(format!(
                "Archivo no existe: {}",
                ruta
            )));
        }
        if !path.is_file() {
            return Err(ImagenValidationError::new(format!(
                "No es un archivo regular: {}",
                ruta
            )));
        }

        let data = fs::read(path).map_err(|e| match e.kind() {
            std::io::ErrorKind::PermissionDenied => {
                ImagenValidationError::new(format!("Sin permisos de lectura: {}", ruta))
            }
            _ => ImagenValidationError::new(format!("Error leyendo archivo: {}", e)),
        })?;

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();

        self.validar_bytes(&data, filename)
    }
}















#[cfg(test)]
mod tests {
    use crate::base::validadores::utilidades_globales::imagen_validador::ImagenValidator;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn validar_imagen_y_guardar_json() {
        let validador = ImagenValidator::default();
        let ruta_imagen = "static/portada/gur.png";
        let ruta_json = "resultado_validacion.json";

        match validador.validar_desde_ruta(ruta_imagen) {
            Ok(meta) => {
                println!("✅ Imagen válida: {}x{}", meta.width, meta.height);

                let json_string = serde_json::to_string_pretty(&meta)
                    .expect("❌ Error al serializar a JSON");

                guardar_json(ruta_json, &json_string)
                    .expect("❌ Error al escribir el archivo JSON");

                println!("💾 Metadatos guardados exitosamente en: {}", ruta_json);

                // Validación básica para el test
                assert!(meta.width > 0);
                assert!(meta.height > 0);
            }
            Err(e) => {
                panic!("❌ Error de validación: {}", e);
            }
        }
    }

    fn guardar_json(ruta: &str, contenido: &str) -> std::io::Result<()> {
        let mut archivo = File::create(ruta)?;
        archivo.write_all(contenido.as_bytes())?;
        Ok(())
    }
}