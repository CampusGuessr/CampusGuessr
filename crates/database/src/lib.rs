use rusqlite::{Connection, Result, Row, params};
use std::path::Path;

pub fn create_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS files (
                  id INTEGER PRIMARY KEY AUTOINCREMENT,
                  name TEXT NOT NULL,
                  coordinates TEXT NOT NULL 
                  )",
        [],
    )?;
    Ok(())
}

pub fn open_admin(path: &Path) -> Result<Connection> {
    let conn = Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_CREATE,
    )?;
    Ok(conn)
}

pub fn init_database(path: &Path) -> Result<()> {
    let conn = open_admin(path)?;
    create_table(&conn)?;
    Ok(())
}

pub fn insert_file(conn: &Connection, name: &str, coordinates: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO files (name, coordinates) VALUES (?1, ?2)",
        params![name, coordinates],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_file_by_id(conn: &Connection, id: i64) -> Result<(String, String)> {
    let mut stmt = conn.prepare("SELECT name, coordinates FROM files WHERE id = ?1")?;
    stmt.query_row(params![id], |row: &Row| {
        let name: String = row.get(0)?;
        let coordinates: String = row.get(1)?;
        Ok((name, coordinates))
    })
}

pub fn delete_file(conn: &Connection, id: i64) -> Result<bool> {
    let rows_affected = conn.execute("DELETE FROM files WHERE id = ?1", params![id])?;
    Ok(rows_affected > 0)
}

pub fn open_read_only(path: &Path) -> Result<Connection> {
    let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn temp_db_path(test_name: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("test_{}.db", test_name));
        path
    }
    fn memory_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        create_table(&conn).unwrap();
        conn
    }
    #[test]
    fn create_table_can_run_twice() {
        let conn = Connection::open_in_memory().unwrap();
        create_table(&conn).unwrap();
        create_table(&conn).unwrap();
    }
    #[test]
    fn open_admin_creates_the_file() {
        let path = temp_db_path("open_admin_creates_the_file");
        if path.exists() {
            std::fs::remove_file(&path).unwrap();
        }
        let conn = open_admin(&path).unwrap();
        assert!(path.exists());
        drop(conn);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn init_database_creates_the_table() {
        let path = temp_db_path("init_database_creates_the_table");
        if path.exists() {
            std::fs::remove_file(&path).unwrap();
        }
        init_database(&path).unwrap();
        {
            let conn = Connection::open(&path).unwrap();
            let mut stmt = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='files'")
                .unwrap();
            let mut rows = stmt.query([]).unwrap();
            assert!(rows.next().unwrap().is_some());
        }
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn init_database_can_run_twice_on_the_same_file() {
        let path = temp_db_path("init_database_can_run_twice_on_the_same_file");
        if path.exists() {
            std::fs::remove_file(&path).unwrap();
        }
        init_database(&path).unwrap();
        init_database(&path).unwrap();

        assert!(path.exists());

        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn insert_returns_new_id() {
        let conn = memory_db();
        let new_id1 = insert_file(&conn, "test_file.txt", "0,0").unwrap();
        assert_eq!(new_id1, 1);
        let new_id2 = insert_file(&conn, "test_file2.txt", "1,1").unwrap();
        assert_eq!(new_id2, 2);
    }
    #[test]
    fn insert_and_read_back() {
        let conn = memory_db();
        let id = insert_file(&conn, "test_file.txt", "0,0").unwrap();
        let (name, coordinates) = get_file_by_id(&conn, id).unwrap();
        assert_eq!(name, "test_file.txt");
        assert_eq!(coordinates, "0,0");
    }

    #[test]
    fn get_unknown_id_returns_no_rows_error() {
        let conn = memory_db();
        let id = insert_file(&conn, "test_file.txt", "0,0").unwrap();
        let result = get_file_by_id(&conn, id + 1);
        assert!(
            matches!(result, Err(rusqlite::Error::QueryReturnedNoRows)),
            "Expected QueryReturnedNoRows error, got {:?}",
            result
        );
    }

    #[test]
    fn special_characters_are_stored_unchanged() {
        let conn = memory_db();
        let special_name = "x'); DROP TABLE files; --";
        let id = insert_file(&conn, special_name, "0,0").unwrap();
        let (name, _) = get_file_by_id(&conn, id).unwrap();
        assert_eq!(name, special_name);
        let id2 = insert_file(&conn, "test_file2.txt", "1,1").unwrap();
        assert_eq!(id2, id + 1);
    }

    #[test]
    fn init_database_keeps_existing_data() {
        let path = temp_db_path("init_database_keeps_existing_data");
        if path.exists() {
            std::fs::remove_file(&path).unwrap();
        }
        init_database(&path).unwrap();
        let conn = open_admin(&path).unwrap();
        let id = insert_file(&conn, "test_file.txt", "0,0").unwrap();
        drop(conn);
        init_database(&path).unwrap();
        let conn = open_admin(&path).unwrap();
        let (name, coordinates) = get_file_by_id(&conn, id).unwrap();
        assert_eq!(name, "test_file.txt");
        assert_eq!(coordinates, "0,0");
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn delete_removes_only_the_given_entry() {
        let conn = memory_db();
        let id1 = insert_file(&conn, "test_file1.txt", "0,0").unwrap();
        let id2 = insert_file(&conn, "test_file2.txt", "1,1").unwrap();
        assert!(delete_file(&conn, id1).unwrap());
        let result1 = get_file_by_id(&conn, id1);
        assert!(
            matches!(result1, Err(rusqlite::Error::QueryReturnedNoRows)),
            "Expected QueryReturnedNoRows error for deleted entry, got {:?}",
            result1
        );
        let result2 = get_file_by_id(&conn, id2);
        assert!(
            result2.is_ok(),
            "Expected to find the second entry, got {:?}",
            result2
        );
        assert!(
            !delete_file(&conn, id1).unwrap(),
            "Expected delete to return false for non-existing entry"
        );
    }
    #[test]
    fn open_read_only_fails_on_missing_file() {
        let path = temp_db_path("open_read_only_fails_on_missing_file");
        if path.exists() {
            std::fs::remove_file(&path).unwrap();
        }
        let result = open_read_only(&path);
        assert!(result.is_err());
        assert!(!path.exists());
    }

    #[test]
    fn open_read_only_can_read_but_not_write() {
        let path = temp_db_path("open_read_only_can_read_but_not_write");
        if path.exists() {
            std::fs::remove_file(&path).unwrap();
        }
        init_database(&path).unwrap();
        let conn = open_admin(&path).unwrap();
        let id = insert_file(&conn, "file.txt", "1.2").unwrap();
        drop(conn);
        let conn = open_read_only(&path).unwrap();
        let (name, coordinates) = get_file_by_id(&conn, id).unwrap();
        assert_eq!(name, "file.txt");
        assert_eq!(coordinates, "1.2");
        let insert_result = insert_file(&conn, "new_file.txt", "1.0");
        assert!(insert_result.is_err());
        let delete_result = delete_file(&conn, id);
        assert!(delete_result.is_err());
        drop(conn);
        std::fs::remove_file(&path).unwrap();
    }
}
