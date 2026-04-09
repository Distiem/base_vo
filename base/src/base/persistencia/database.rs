use sea_orm::{ConnectionTrait, Database, Statement};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct DbResponse {
    pub estado: String,
    pub mensaje: String,
}

pub struct DbAdmin;

impl DbAdmin {
    /// Crea una base de datos en PostgreSQL si no existe.
    pub async fn crear_base_de_datos(
        db_name: &str,
        db_host: &str,
        db_port: u16,
        db_user: &str,
        db_pass: &str,
        owner: Option<&str>,
        encoding: Option<&str>,
        template: Option<&str>,
    ) -> DbResponse {
        let url = format!("postgres://{}:{}@{}:{}/postgres", db_user, db_pass, db_host, db_port);
        
        // Conectamos usando sea_orm
        let db = match Database::connect(&url).await {
            Ok(conn) => conn,
            Err(e) => return DbResponse { estado: "error".to_string(), mensaje: e.to_string() },
        };

        // 1. Verificar si existe
        let query = Statement::from_sql_and_values(
            db.get_database_backend(),
            "SELECT 1 FROM pg_database WHERE datname = $1",
            [db_name.into()],
        );

        let existe = db.query_one(query).await.unwrap_or(None).is_some();

        if existe {
            return DbResponse {
                estado: "existe".to_string(),
                mensaje: format!("La base de datos '{}' ya existe", db_name),
            };
        }

        // 2. Construcción del SQL 
        let encoding = encoding.unwrap_or("UTF8");
        let template = template.unwrap_or("template1");
        let mut sql = format!("CREATE DATABASE {} WITH ENCODING '{}' TEMPLATE {}", db_name, encoding, template);
        
        if let Some(o) = owner {
            sql.push_str(&format!(" OWNER {}", o));
        }

        // 3. Ejecutar
        match db.execute(Statement::from_string(db.get_database_backend(), sql)).await {
            Ok(_) => DbResponse {
                estado: "ok".to_string(),
                mensaje: format!("Base de datos '{}' creada correctamente", db_name),
            },
            Err(e) => DbResponse {
                estado: "error".to_string(),
                mensaje: e.to_string(),
            },
        }
    }

    /// Elimina una base de datos en PostgreSQL.
    pub async fn eliminar_base_de_datos(
        db_name: &str,
        db_host: &str,
        db_port: u16,
        db_user: &str,
        db_pass: &str,
        forzar: bool,
    ) -> DbResponse {
        let url = format!("postgres://{}:{}@{}:{}/postgres", db_user, db_pass, db_host, db_port);
        let db = match Database::connect(&url).await {
            Ok(conn) => conn,
            Err(e) => return DbResponse { estado: "error".to_string(), mensaje: e.to_string() },
        };

        // Terminar conexiones si se fuerza (Postgres >= 13)
        if forzar {
            let terminate_sql = "
                SELECT pg_terminate_backend(pid) 
                FROM pg_stat_activity 
                WHERE datname = $1 AND pid <> pg_backend_pid()";
            
            let _ = db.execute(Statement::from_sql_and_values(
                db.get_database_backend(),
                terminate_sql,
                [db_name.into()],
            )).await;
        }

        let drop_sql = format!("DROP DATABASE IF EXISTS {}", db_name);
        match db.execute(Statement::from_string(db.get_database_backend(), drop_sql)).await {
            Ok(_) => DbResponse {
                estado: "ok".to_string(),
                mensaje: format!("Base de datos '{}' eliminada", db_name),
            },
            Err(e) => DbResponse {
                estado: "error".to_string(),
                mensaje: e.to_string(),
            },
        }
    }

    /// Devuelve los nombres de las bases de datos existentes.
    pub async fn listar_bases_de_datos(
        db_host: &str,
        db_port: u16,
        db_user: &str,
        db_pass: &str,
        incluir_sistema: bool,
    ) -> Vec<String> {
        let url = format!("postgres://{}:{}@{}:{}/postgres", db_user, db_pass, db_host, db_port);
        let db = Database::connect(&url).await.expect("Error al conectar");

        let mut sql = "SELECT datname FROM pg_database WHERE datistemplate = false".to_string();
        if !incluir_sistema {
            sql.push_str(" AND datname NOT IN ('postgres')");
        }
        sql.push_str(" ORDER BY datname");

        let rows = db.query_all(Statement::from_string(db.get_database_backend(), sql))
            .await
            .unwrap_or_default();

        rows.into_iter()
            .map(|row| row.try_get_by_index::<String>(0).unwrap_or_default())
            .collect()
    }
}