use rusqlite::Connection;
use std::{env, error::Error, fs, path::Path};

pub use crate::sqlite_pragma;

pub fn init_database() -> Result<(), Box<dyn Error>> {
    let db_path = env::var("SOLFLOW_DB_PATH")
        .map_err(|_| "SOLFLOW_DB_PATH environment variable not set")?;

    let conn = Connection::open(&db_path)?;
    
    sqlite_pragma::apply_optimized_pragmas(&conn)?;
    
    run_migrations(&conn)?;
    
    Ok(())
}

fn run_migrations(conn: &Connection) -> Result<(), Box<dyn Error>> {
    let sql_dir = Path::new("sql");
    
    if !sql_dir.exists() {
        return Err("sql/ directory not found".into());
    }

    let mut sql_files: Vec<_> = fs::read_dir(sql_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "sql")
                .unwrap_or(false)
        })
        .collect();

    sql_files.sort_by_key(|entry| entry.file_name());

    let migration_count = sql_files.len();

    for entry in sql_files {
        let path = entry.path();
        let sql = fs::read_to_string(&path)?;
        
        if let Err(e) = conn.execute_batch(&sql) {
            log::warn!("⚠️  Migration {} failed (may be incomplete): {}", 
                       path.file_name().unwrap().to_string_lossy(), e);
        }
    }

    log::info!("✅ Executed {} migrations successfully", migration_count);

    Ok(())
}
