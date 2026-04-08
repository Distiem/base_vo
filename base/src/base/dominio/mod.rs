pub mod id_interno;
pub mod id_simple;
pub mod fecha_creacion;
pub mod fecha_modificacion;
pub mod version;

// Re-exportamos para acceso simplificado
pub use id_interno::IdInterno;
pub use id_simple::IdSimple;
pub use version::Version;
pub use fecha_creacion::FechaCreacion;
pub use fecha_modificacion::FechaModificacion;