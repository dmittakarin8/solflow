use rusqlite::Connection;
use std::{env, fs};

#[test]
fn test_db_initialization() {
    let test_db_path = "./test_solflow.db";
    
    let _ = fs::remove_file(test_db_path);
    
    env::set_var("SOLFLOW_DB_PATH", test_db_path);
    
    solflow::db::init_database().expect("Database initialization should succeed");
    
    let conn = Connection::open(test_db_path).expect("Should open test database");
    
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .expect("Should prepare query");
    
    let tables: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .expect("Should query tables")
        .filter_map(Result::ok)
        .collect();
    
    assert!(!tables.is_empty(), "Database should have tables after migrations");
    
    println!("✅ Found {} tables: {:?}", tables.len(), tables);
    
    fs::remove_file(test_db_path).expect("Should clean up test database");
}
