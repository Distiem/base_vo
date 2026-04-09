use sea_orm::{Database, DatabaseConnection, ConnectOptions};
use std::time::Duration;
pub struct Db;
impl Db {
    /// Configura y establece la conexión con la base de datos PostgreSQL
    pub async fn crear_conexion(
        user: &str,
        pass: &str,
        host: &str,
        port: u16,
        db_name: &str,
    ) -> Result<DatabaseConnection, sea_orm::DbErr> {
        let url = format!("postgres://{}:{}@{}:{}/{}", user, pass, host, port, db_name);
        let mut opt = ConnectOptions::new(url);

        // Configuración del pool de conexiones y timeouts
        opt.max_connections(10)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(30))
            .acquire_timeout(Duration::from_secs(30))
            .idle_timeout(Duration::from_secs(10))
            .max_lifetime(Duration::from_secs(1800))
            .sqlx_logging(true);

        Database::connect(opt).await
    }
}