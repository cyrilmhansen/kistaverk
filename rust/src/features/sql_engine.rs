// SQL Engine Feature Module
// Provides CSV/JSON file import and SQL query execution using SQLite

use crate::state::AppState;
use serde_json::{json, Value};
use sqlite::{Connection, State};
use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    os::unix::io::RawFd,
};

#[cfg(test)]
use tempfile::NamedTempFile;

/// Information about an imported table
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub source_path: String,
    pub row_count: usize,
    pub column_count: usize,
    pub column_names: Vec<String>,
}

/// SQL query result structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub error: Option<String>,
    pub row_count: usize,
}

/// SQL Engine main structure
pub struct SqlEngine {
    conn: Connection,
    tables: HashMap<String, TableInfo>,
}

impl std::fmt::Debug for SqlEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqlEngine")
            .field("tables", &self.tables)
            .finish()
    }
}

impl Clone for SqlEngine {
    fn clone(&self) -> Self {
        // For simplicity, we'll just create a new empty engine
        // In a real implementation, we'd need to copy the data
        SqlEngine {
            conn: Connection::open(":memory:").expect("Failed to create connection for clone"),
            tables: self.tables.clone(),
        }
    }
}

impl SqlEngine {
    /// Create a new SQL engine with in-memory database
    pub fn new() -> Result<Self, String> {
        let conn = Connection::open(":memory:")
            .map_err(|e| format!("Failed to create SQLite connection: {}", e))?;
        
        Ok(Self {
            conn,
            tables: HashMap::new(),
        })
    }

    /// Import a CSV file as a table
    pub fn import_csv(&mut self, path: &str, fd: Option<RawFd>, table_name: &str) -> Result<TableInfo, String> {
        // Read CSV file
        let csv_content = if let Some(fd_num) = fd {
            read_file_from_fd(fd_num)?
        } else {
            read_file_from_path(path)?
        };

        // Parse CSV to determine schema
        let mut rdr = csv::Reader::from_reader(csv_content.as_bytes());
        let headers = rdr.headers()
            .map_err(|e| format!("Failed to read CSV headers: {}", e))?
            .iter()
            .map(|h| sanitize_column_name(h))
            .collect::<Vec<String>>();

        if headers.is_empty() {
            return Err("CSV file has no columns".to_string());
        }

        // Create table
        let create_table_sql = format!(
            "CREATE TABLE {} ({})",
            table_name,
            headers.iter()
                .map(|col| format!("{} TEXT", col))
                .collect::<Vec<String>>()
                .join(", ")
        );

        self.conn.execute(create_table_sql)
            .map_err(|e| format!("Failed to create table: {}", e))?;

        // Insert data
        let mut row_count = 0;
        for record in rdr.records() {
            let record = record.map_err(|e| format!("Failed to read CSV record: {}", e))?;
            
            let values = record.iter()
                .map(|v| format!("'{}'", v.replace("'", "''")))
                .collect::<Vec<String>>();

            let insert_sql = format!(
                "INSERT INTO {} VALUES ({})",
                table_name,
                values.join(", ")
            );

            self.conn.execute(insert_sql)
                .map_err(|e| format!("Failed to insert row: {}", e))?;
            row_count += 1;
        }

        let table_info = TableInfo {
            name: table_name.to_string(),
            source_path: path.to_string(),
            row_count,
            column_count: headers.len(),
            column_names: headers,
        };

        self.tables.insert(table_name.to_string(), table_info.clone());
        Ok(table_info)
    }

    /// Import a JSON file as a table
    pub fn import_json(&mut self, path: &str, fd: Option<RawFd>, table_name: &str) -> Result<TableInfo, String> {
        // Read JSON file
        let json_content = if let Some(fd_num) = fd {
            read_file_from_fd(fd_num)?
        } else {
            read_file_from_path(path)?
        };

        // Parse JSON
        let json_value: serde_json::Value = serde_json::from_str(&json_content)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        // Determine if it's an array or object
        let (headers, rows) = if json_value.is_array() {
            // Array of objects
            let array = json_value.as_array().unwrap();
            if array.is_empty() {
                return Err("JSON array is empty".to_string());
            }

            // Get headers from first object
            let first_obj = &array[0];
            if !first_obj.is_object() {
                return Err("JSON array should contain objects".to_string());
            }

            let headers = first_obj.as_object().unwrap()
                .keys()
                .map(|k| sanitize_column_name(k))
                .collect::<Vec<String>>();

            if headers.is_empty() {
                return Err("JSON objects have no properties".to_string());
            }

            // Create table
            let create_table_sql = format!(
                "CREATE TABLE {} ({})",
                table_name,
                headers.iter()
                    .map(|col| format!("{} TEXT", col))
                    .collect::<Vec<String>>()
                    .join(", ")
            );

            self.conn.execute(create_table_sql)
                .map_err(|e| format!("Failed to create table: {}", e))?;

            // Insert data
            let mut row_count = 0;
            for item in array {
                if !item.is_object() {
                    continue;
                }
                
                let obj = item.as_object().unwrap();
                let values = headers.iter()
                    .map(|col| {
                        obj.get(col)
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .replace("'", "''")
                    })
                    .map(|v| format!("'{}'", v))
                    .collect::<Vec<String>>();

                let insert_sql = format!(
                    "INSERT INTO {} VALUES ({})",
                    table_name,
                    values.join(", ")
                );

                self.conn.execute(insert_sql)
                    .map_err(|e| format!("Failed to insert row: {}", e))?;
                row_count += 1;
            }

            (headers, row_count)
        } else if json_value.is_object() {
            // Single object - treat as single row
            let obj = json_value.as_object().unwrap();
            let headers = obj.keys()
                .map(|k| sanitize_column_name(k))
                .collect::<Vec<String>>();

            if headers.is_empty() {
                return Err("JSON object has no properties".to_string());
            }

            // Create table
            let create_table_sql = format!(
                "CREATE TABLE {} ({})",
                table_name,
                headers.iter()
                    .map(|col| format!("{} TEXT", col))
                    .collect::<Vec<String>>()
                    .join(", ")
            );

            self.conn.execute(create_table_sql)
                .map_err(|e| format!("Failed to create table: {}", e))?;

            // Insert single row
            let values = headers.iter()
                .map(|col| {
                    obj.get(col)
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .replace("'", "''")
                })
                .map(|v| format!("'{}'", v))
                .collect::<Vec<String>>();

            let insert_sql = format!(
                "INSERT INTO {} VALUES ({})",
                table_name,
                values.join(", ")
            );

            self.conn.execute(insert_sql)
                .map_err(|e| format!("Failed to insert row: {}", e))?;

            (headers, 1)
        } else {
            return Err("JSON should be an array or object".to_string());
        };

        let table_info = TableInfo {
            name: table_name.to_string(),
            source_path: path.to_string(),
            row_count: rows,
            column_count: headers.len(),
            column_names: headers,
        };

        self.tables.insert(table_name.to_string(), table_info.clone());
        Ok(table_info)
    }

    /// Execute a SQL query
    pub fn execute_query(&self, query: &str) -> QueryResult {
        let mut stmt = match self.conn.prepare(query) {
            Ok(stmt) => stmt,
            Err(e) => return QueryResult {
                columns: vec![],
                rows: vec![],
                error: Some(format!("SQL preparation error: {}", e)),
                row_count: 0,
            },
        };

        // Get column names
        let columns: Vec<String> = stmt.column_names()
            .iter()
            .map(|col| col.to_string())
            .collect();

        // Execute query and collect results
        let mut rows = Vec::new();
        let mut row_count = 0;

        loop {
            match stmt.next() {
                Ok(State::Row) => {
                    let mut row = Vec::new();
                    for i in 0..columns.len() {
                        let value = match stmt.read::<String, usize>(i) {
                            Ok(value) => value,
                            Err(e) => format!("<error: {}", e),
                        };
                        row.push(value);
                    }
                    rows.push(row);
                    row_count += 1;
                }
                Ok(State::Done) => break,
                Err(e) => return QueryResult {
                    columns,
                    rows: vec![],
                    error: Some(format!("SQL execution error: {}", e)),
                    row_count: 0,
                },
            }
        }

        QueryResult {
            columns,
            rows,
            error: None,
            row_count,
        }
    }

    /// Get list of imported tables
    pub fn get_tables(&self) -> Vec<TableInfo> {
        self.tables.values().cloned().collect()
    }

    /// Clear all tables
    pub fn clear_all(&mut self) -> Result<(), String> {
        for table_name in self.tables.keys() {
            let drop_sql = format!("DROP TABLE IF EXISTS {}", table_name);
            self.conn.execute(drop_sql)
                .map_err(|e| format!("Failed to drop table {}: {}", table_name, e))?;
        }
        self.tables.clear();
        Ok(())
    }
}

/// Read file from file descriptor
fn read_file_from_fd(fd: RawFd) -> Result<String, String> {
    use std::os::unix::io::FromRawFd;
    let mut file = unsafe { File::from_raw_fd(fd) };
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| format!("Failed to read from fd {}: {}", fd, e))?;
    // Don't close the file - it's owned by Android
    std::mem::forget(file);
    Ok(content)
}

/// Read file from path
fn read_file_from_path(path: &str) -> Result<String, String> {
    let mut file = File::open(path)
        .map_err(|e| format!("Failed to open file {}: {}", path, e))?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| format!("Failed to read file {}: {}", path, e))?;
    Ok(content)
}

/// Sanitize column names for SQLite
fn sanitize_column_name(name: &str) -> String {
    name.replace(|c: char| !c.is_ascii_alphanumeric() && c != '_', "_")
        .trim_matches('_')
        .to_string()
}

/// Public interface functions for router integration
pub fn handle_sql_action(state: &mut AppState, action: &str, bindings: &HashMap<String, String>) -> Value {
    match action {
        "sql_screen" => {
            render_sql_screen(state)
        }
        "sql_import_csv" => {
            let path = bindings.get("path").unwrap_or(&"".to_string()).to_string();
            let fd = bindings.get("fd").and_then(|f| f.parse().ok());
            let table_name = bindings.get("table_name").cloned().unwrap_or_else(|| "table1".to_string());
            handle_import_csv(state, &path, fd, &table_name)
        }
        "sql_import_json" => {
            let path = bindings.get("path").unwrap_or(&"".to_string()).to_string();
            let fd = bindings.get("fd").and_then(|f| f.parse().ok());
            let table_name = bindings.get("table_name").cloned().unwrap_or_else(|| "table1".to_string());
            handle_import_json(state, &path, fd, &table_name)
        }
        "sql_execute" => {
            let query = bindings.get("query").unwrap_or(&"".to_string()).to_string();
            handle_execute_query(state, &query)
        }
        "sql_clear_all" => {
            handle_clear_all(state)
        }
        _ => json!({"error": "unknown_sql_action"}),
    }
}

fn handle_import_csv(state: &mut AppState, path: &str, fd: Option<RawFd>, table_name: &str) -> Value {
    let path_str = path.to_string();
    let table_name_str = table_name.to_string();
    
    let mut engine = match state.sql_engine.take() {
        Some(engine) => engine,
        None => match SqlEngine::new() {
            Ok(engine) => engine,
            Err(e) => return json!({"error": e}),
        },
    };

    match engine.import_csv(&path_str, fd, &table_name_str) {
        Ok(table_info) => {
            state.sql_engine = Some(engine);
            json!({
                "screen": "sql_query",
                "tables": [table_info],
                "message": format!("Imported {} rows from {}", table_info.row_count, table_info.source_path)
            })
        }
        Err(e) => {
            state.sql_engine = Some(engine);
            json!({"error": e})
        }
    }
}

fn handle_import_json(state: &mut AppState, path: &str, fd: Option<RawFd>, table_name: &str) -> Value {
    let path_str = path.to_string();
    let table_name_str = table_name.to_string();
    
    let mut engine = match state.sql_engine.take() {
        Some(engine) => engine,
        None => match SqlEngine::new() {
            Ok(engine) => engine,
            Err(e) => return json!({"error": e}),
        },
    };

    match engine.import_json(&path_str, fd, &table_name_str) {
        Ok(table_info) => {
            state.sql_engine = Some(engine);
            json!({
                "screen": "sql_query",
                "tables": [table_info],
                "message": format!("Imported {} rows from {}", table_info.row_count, table_info.source_path)
            })
        }
        Err(e) => {
            state.sql_engine = Some(engine);
            json!({"error": e})
        }
    }
}

fn handle_execute_query(state: &mut AppState, query: &str) -> Value {
    let query_str = query.to_string();
    let engine = match &state.sql_engine {
        Some(engine) => engine,
        None => return json!({"error": "No SQL engine initialized"}),
    };

    let result = engine.execute_query(&query_str);
    
    if let Some(error) = result.error {
        json!({
            "screen": "sql_query",
            "error": error,
            "query": query_str
        })
    } else {
        json!({
            "screen": "sql_query",
            "result": {
                "columns": result.columns,
                "rows": result.rows,
                "row_count": result.row_count
            },
            "query": query_str
        })
    }
}

fn handle_clear_all(state: &mut AppState) -> Value {
    let mut engine = match state.sql_engine.take() {
        Some(engine) => engine,
        None => return json!({"screen": "sql_query"}),
    };

    match engine.clear_all() {
        Ok(_) => {
            state.sql_engine = Some(engine);
            json!({
                "screen": "sql_query",
                "tables": [],
                "message": "All tables cleared"
            })
        }
        Err(e) => {
            state.sql_engine = Some(engine);
            json!({"error": e})
        }
    }
}

/// Render the SQL query screen
pub fn render_sql_screen(state: &AppState) -> Value {
    let tables = if let Some(engine) = &state.sql_engine {
        engine.get_tables()
    } else {
        // Initialize new engine if none exists
        match SqlEngine::new() {
            Ok(engine) => engine.get_tables(),
            Err(_) => return json!({"error": "Failed to initialize SQL engine"}),
        }
    };

    let tables_list = render_tables_list(&tables);

    json!({
        "type": "Column",
        "children": [
            {
                "type": "Section",
                "title": "SQL Query Lab",
                "subtitle": "Import CSV/JSON files and run SQL queries",
                "children": [
                    {
                        "type": "Text",
                        "text": "Import data files to create tables:",
                        "size": 16
                    },
                    {
                        "type": "Button",
                        "text": "Import CSV/JSON File",
                        "action": "sql_import",
                        "requires_file_picker": true,
                        "content_description": "Import CSV or JSON file for SQL querying"
                    },
                    tables_list,
                    {
                        "type": "TextInput",
                        "bind_key": "sql_query",
                        "text": "",
                        "hint": "SELECT * FROM users WHERE age > 30",
                        "max_lines": 8,
                        "content_description": "SQL query editor",
                        "debounce_ms": 300
                    },
                    {
                        "type": "Button",
                        "text": "Execute Query",
                        "action": "sql_execute",
                        "content_description": "Execute SQL query"
                    },
                    {
                        "type": "Button",
                        "text": "Clear All Tables",
                        "action": "sql_clear_all",
                        "content_description": "Clear all imported tables"
                    }
                ]
            }
        ]
    })
}

fn render_tables_list(tables: &[TableInfo]) -> Value {
    if tables.is_empty() {
        return json!({
            "type": "Text",
            "text": "No tables imported yet.",
            "size": 14,
            "content_description": "No tables imported"
        });
    }

    let table_items = tables.iter().map(|table| {
        json!({
            "type": "Card",
            "children": [
                {
                    "type": "Text",
                    "text": format!("📄 {} ({} rows, {} columns)", table.name, table.row_count, table.column_count),
                    "size": 16
                },
                {
                    "type": "Text",
                    "text": format!("Source: {}", table.source_path),
                    "size": 12
                },
                {
                    "type": "Text",
                    "text": format!("Columns: {}", table.column_names.join(", ")),
                    "size": 12
                }
            ]
        })
    }).collect::<Vec<Value>>();

    json!({
        "type": "VirtualList",
        "children": table_items,
        "estimated_item_height": 120
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_csv_import_and_query() {
        // Create a temporary CSV file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,NYC").unwrap();
        writeln!(temp_file, "Bob,25,LA").unwrap();
        writeln!(temp_file, "Charlie,35,Chicago").unwrap();

        let path = temp_file.path().to_str().unwrap().to_string();

        // Create SQL engine and import CSV
        let mut engine = SqlEngine::new().unwrap();
        let table_info = engine.import_csv(&path, None, "users").unwrap();

        // Verify table info
        assert_eq!(table_info.name, "users");
        assert_eq!(table_info.row_count, 3);
        assert_eq!(table_info.column_count, 3);
        assert_eq!(table_info.column_names, vec!["name", "age", "city"]);

        // Test simple query
        let result = engine.execute_query("SELECT * FROM users");
        assert!(result.error.is_none());
        assert_eq!(result.columns, vec!["name", "age", "city"]);
        assert_eq!(result.row_count, 3);

        // Test filtered query
        let result = engine.execute_query("SELECT name, age FROM users WHERE age > 30");
        assert!(result.error.is_none());
        assert_eq!(result.columns, vec!["name", "age"]);
        assert_eq!(result.row_count, 1);
        assert_eq!(result.rows[0][0], "Charlie");
        assert_eq!(result.rows[0][1], "35");
    }

    #[test]
    fn test_json_array_import_and_query() {
        // Create a temporary JSON file with array
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"[
            {{"name": "Alice", "age": "30", "city": "NYC"}},
            {{"name": "Bob", "age": "25", "city": "LA"}},
            {{"name": "Charlie", "age": "35", "city": "Chicago"}}
        ]"#).unwrap();

        let path = temp_file.path().to_str().unwrap().to_string();

        // Create SQL engine and import JSON
        let mut engine = SqlEngine::new().unwrap();
        let table_info = engine.import_json(&path, None, "people").unwrap();

        // Verify table info
        assert_eq!(table_info.name, "people");
        assert_eq!(table_info.row_count, 3);
        assert_eq!(table_info.column_count, 3);
        // Column names should contain the expected fields (order may vary in JSON)
        let mut expected_columns = vec!["name", "age", "city"];
        let mut actual_columns = table_info.column_names.clone();
        expected_columns.sort();
        actual_columns.sort();
        assert_eq!(actual_columns, expected_columns);

        // Test query
        let result = engine.execute_query("SELECT name FROM people WHERE city = 'LA'");
        assert!(result.error.is_none());
        assert_eq!(result.columns, vec!["name"]);
        assert_eq!(result.row_count, 1);
        assert_eq!(result.rows[0][0], "Bob");
    }

    #[test]
    fn test_json_object_import() {
        // Create a temporary JSON file with single object
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"name": "Alice", "age": "30", "city": "NYC"}}"#).unwrap();

        let path = temp_file.path().to_str().unwrap().to_string();

        // Create SQL engine and import JSON
        let mut engine = SqlEngine::new().unwrap();
        let table_info = engine.import_json(&path, None, "single_person").unwrap();

        // Verify table info
        assert_eq!(table_info.name, "single_person");
        assert_eq!(table_info.row_count, 1);
        assert_eq!(table_info.column_count, 3);
        // Column names should contain the expected fields (order may vary in JSON)
        let mut expected_columns = vec!["name", "age", "city"];
        let mut actual_columns = table_info.column_names.clone();
        expected_columns.sort();
        actual_columns.sort();
        assert_eq!(actual_columns, expected_columns);

        // Test query
        let result = engine.execute_query("SELECT * FROM single_person");
        assert!(result.error.is_none());
        assert_eq!(result.row_count, 1);
        // Check that Alice is in one of the columns (order may vary)
        let alice_found = result.rows[0].iter().any(|col| col == "Alice");
        assert!(alice_found);
    }

    #[test]
    fn test_invalid_csv_handling() {
        // Create a temporary CSV file with no headers
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Alice,30,NYC").unwrap();

        let path = temp_file.path().to_str().unwrap().to_string();

        // Create SQL engine and try to import invalid CSV
        let mut engine = SqlEngine::new().unwrap();
        let result = engine.import_csv(&path, None, "invalid");

        // Should return an error
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_json_handling() {
        // Create a temporary JSON file with invalid content
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "not valid json").unwrap();

        let path = temp_file.path().to_str().unwrap().to_string();

        // Create SQL engine and try to import invalid JSON
        let mut engine = SqlEngine::new().unwrap();
        let result = engine.import_json(&path, None, "invalid");

        // Should return an error
        assert!(result.is_err());
    }

    #[test]
    fn test_sql_error_handling() {
        let mut engine = SqlEngine::new().unwrap();
        
        // Test invalid SQL query
        let result = engine.execute_query("SELECT FROM nonexistent");
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("SQL"));
        
        // Test query on non-existent table
        let result = engine.execute_query("SELECT * FROM does_not_exist");
        assert!(result.error.is_some());
    }

    #[test]
    fn test_column_name_sanitization() {
        // Create a CSV with special characters in column names
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "first name,age,city-name").unwrap();
        writeln!(temp_file, "Alice,30,NYC").unwrap();

        let path = temp_file.path().to_str().unwrap().to_string();

        let mut engine = SqlEngine::new().unwrap();
        let table_info = engine.import_csv(&path, None, "test_table").unwrap();

        // Column names should be sanitized
        assert_eq!(table_info.column_names, vec!["first_name", "age", "city_name"]);

        // Should be able to query with sanitized names
        let result = engine.execute_query("SELECT first_name FROM test_table");
        assert!(result.error.is_none());
        assert_eq!(result.row_count, 1);
    }

    #[test]
    fn test_get_tables_and_clear() {
        let engine = SqlEngine::new().unwrap();
        
        // Should start with no tables
        assert!(engine.get_tables().is_empty());
        
        // Create a temporary CSV file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age").unwrap();
        writeln!(temp_file, "Alice,30").unwrap();

        let path = temp_file.path().to_str().unwrap().to_string();
        
        // Import CSV
        let mut engine = engine.clone();
        engine.import_csv(&path, None, "table1").unwrap();
        assert_eq!(engine.get_tables().len(), 1);
        
        // Import another CSV
        let mut temp_file2 = NamedTempFile::new().unwrap();
        writeln!(temp_file2, "id,value").unwrap();
        writeln!(temp_file2, "1,test").unwrap();

        let path2 = temp_file2.path().to_str().unwrap().to_string();
        engine.import_csv(&path2, None, "table2").unwrap();
        assert_eq!(engine.get_tables().len(), 2);
        
        // Clear all tables
        engine.clear_all().unwrap();
        assert!(engine.get_tables().is_empty());
    }
}