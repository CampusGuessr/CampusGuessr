use rusqlite::{Connection, Result, Row, params};
use std::path::Path;

#[derive(Debug, PartialEq, Clone)]
pub struct FileEntry {
    pub id: i64,
    pub file_name: String,
    pub latitude: f64,
    pub longitude: f64,
}

pub fn create_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS files (
                  id INTEGER PRIMARY KEY AUTOINCREMENT,
                  file_name TEXT NOT NULL UNIQUE CHECK (length(file_name) > 0 AND instr(file_name, '/') = 0 AND instr(file_name, '\\') = 0),
                  latitude  REAL NOT NULL CHECK (latitude BETWEEN -90 AND 90),
                  longitude REAL NOT NULL CHECK (longitude BETWEEN -180 AND 180)
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

pub fn insert_file(
    conn: &Connection,
    file_name: &str,
    latitude: f64,
    longitude: f64,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO files (file_name, latitude, longitude) VALUES (?1, ?2, ?3)",
        params![file_name, latitude, longitude],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_file_by_id(conn: &Connection, id: i64) -> Result<(String, f64, f64)> {
    let mut stmt =
        conn.prepare("SELECT file_name, latitude, longitude FROM files WHERE id = ?1")?;
    stmt.query_row(params![id], |row: &Row| {
        let file_name: String = row.get(0)?;
        let latitude: f64 = row.get(1)?;
        let longitude: f64 = row.get(2)?;
        Ok((file_name, latitude, longitude))
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
pub fn load_all(conn: &Connection) -> Result<Vec<FileEntry>> {
    let mut stmt =
        conn.prepare("SELECT id, file_name, latitude, longitude FROM files ORDER BY id")?;
    let rows = stmt.query_map([], |row| {
        Ok(FileEntry {
            id: row.get(0)?,
            file_name: row.get(1)?,
            latitude: row.get(2)?,
            longitude: row.get(3)?,
        })
    })?;
    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }
    Ok(entries)
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
    fn try_insert(
        conn: &Connection,
        file_name: &str,
        latitude: f64,
        longitude: f64,
    ) -> Result<usize> {
        conn.execute(
            "INSERT INTO files (file_name, latitude, longitude) VALUES (?1, ?2, ?3)",
            params![file_name, latitude, longitude],
        )
    }
    fn is_constraint_violation<T>(result: &Result<T>) -> bool {
        matches!(result, Err(rusqlite::Error::SqliteFailure(e, _)) if e.code == rusqlite::ErrorCode::ConstraintViolation)
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
        let new_id1 = insert_file(&conn, "test_file.txt", 0.0, 1.1).unwrap();
        assert_eq!(new_id1, 1);
        let new_id2 = insert_file(&conn, "test_file2.txt", 2.2, 3.3).unwrap();
        assert_eq!(new_id2, 2);
    }
    #[test]
    fn insert_and_read_back() {
        let conn = memory_db();
        let id = insert_file(&conn, "test.txt", 0.0, 1.1).unwrap();
        let (file_name, latitude, longitude) = get_file_by_id(&conn, id).unwrap();
        assert_eq!(file_name, "test.txt");
        assert_eq!(latitude, 0.0);
        assert_eq!(longitude, 1.1)
    }

    #[test]
    fn get_unknown_id_returns_no_rows_error() {
        let conn = memory_db();
        let id = insert_file(&conn, "test_file.txt", 0.0, 1.1).unwrap();
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
        let id = insert_file(&conn, special_name, 0.0, 1.1).unwrap();
        let (file_name, _, _) = get_file_by_id(&conn, id).unwrap();
        assert_eq!(file_name, special_name);
        let id2 = insert_file(&conn, "test_file2.txt", 2.2, 3.3).unwrap();
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
        let id = insert_file(&conn, "test_file.txt", 0.0, 1.1).unwrap();
        drop(conn);
        init_database(&path).unwrap();
        let conn = open_admin(&path).unwrap();
        let (file_name, latitude, longitude) = get_file_by_id(&conn, id).unwrap();
        assert_eq!(file_name, "test_file.txt");
        assert_eq!(latitude, 0.0);
        assert_eq!(longitude, 1.1);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn delete_removes_only_the_given_entry() {
        let conn = memory_db();
        let id1 = insert_file(&conn, "test_file1.txt", 0.0, 1.1).unwrap();
        let id2 = insert_file(&conn, "test_file2.txt", 2.2, 3.3).unwrap();
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
        let id = insert_file(&conn, "file.txt", 0.0, 1.1).unwrap();
        drop(conn);
        let conn = open_read_only(&path).unwrap();
        let (file_name, latitude, longitude) = get_file_by_id(&conn, id).unwrap();
        assert_eq!(file_name, "file.txt");
        assert_eq!(latitude, 0.0);
        assert_eq!(longitude, 1.1);
        let insert_result = insert_file(&conn, "new_file.txt", 2.2, 3.3);
        assert!(insert_result.is_err());
        let delete_result = delete_file(&conn, id);
        assert!(delete_result.is_err());
        drop(conn);
        std::fs::remove_file(&path).unwrap();
    }
    #[test]
    fn schema_accepts_valid_rows() {
        let conn = memory_db();
        let result = try_insert(&conn, "a.jpg", 1.23, 4.56);
        assert!(
            result.is_ok(),
            "normaler Eintrag erwartet, {:?} bekommen",
            result
        );
        let result_ne = try_insert(&conn, "b.jpg", 90.0, 180.0);
        assert!(
            result_ne.is_ok(),
            "Wert zwischen 90/180 erwartet, {:?} bekommen",
            result_ne
        );
        let result_sw = try_insert(&conn, "c.jpg", -90.0, -180.0);
        assert!(
            result_sw.is_ok(),
            "Wert zwischen -90/-180 erwartet, {:?} bekommen",
            result_sw
        );
    }

    #[test]
    fn schema_rejects_out_of_range_coordinates() {
        let conn = memory_db();
        let cases = [
            ("Breite 90.0001", 90.0001, 0.0),
            ("Breite -90.0001", -90.0001, 0.0),
            ("Länge 180.0001", 0.0, 180.0001),
            ("Länge -180.0001", 0.0, -180.0001),
        ];
        for (index, (label, latitude, longitude)) in cases.iter().enumerate() {
            let file_name = format!("bad_{index}.jpg");
            let result = try_insert(&conn, &file_name, *latitude, *longitude);
            assert!(
                is_constraint_violation(&result),
                "erwartet: ConstraintViolation bei {label}, erhalten: {:?}",
                result
            );
        }
    }
    #[test]
    fn schema_rejects_duplicate_file_name() {
        let conn = memory_db();
        let first_insert = try_insert(&conn, "dup.jpg", 1.2, 3.4);
        assert!(
            first_insert.is_ok(),
            "erwartet: erster Insert ok, erhalten: {:?}",
            first_insert
        );
        let second_insert = try_insert(&conn, "dup.jpg", 5.6, 7.8);
        assert!(
            is_constraint_violation(&second_insert),
            "erwartet: ConstraintViolation wegen doppeltem Namen, erhalten: {:?}",
            second_insert
        );
        let third_insert = try_insert(&conn, "other.jpg", 1.2, 3.4);
        assert!(
            third_insert.is_ok(),
            "erwartet: gleicher Ort mit anderem Namen ok, erhalten: {:?}",
            third_insert
        );
    }
    #[test]
    fn load_all_returns_empty_list_for_empty_table() {
        let conn = memory_db();
        let entries = load_all(&conn).unwrap();
        assert!(
            entries.is_empty(),
            "leere Liste erwartet, Liste mit {} erhalten",
            entries.len()
        )
    }
    #[test]
    fn load_all_returns_all_entries_in_id_order() {
        let conn = memory_db();
        let id1 = insert_file(&conn, "a.jpg", 0.0, 1.1).unwrap();
        let id2 = insert_file(&conn, "b.jpg", 2.2, 3.3).unwrap();
        let entries = load_all(&conn).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(
            entries[0],
            FileEntry {
                id: id1,
                file_name: "a.jpg".to_string(),
                latitude: 0.0,
                longitude: 1.1
            }
        );
        assert_eq!(
            entries[1],
            FileEntry {
                id: id2,
                file_name: "b.jpg".to_string(),
                latitude: 2.2,
                longitude: 3.3
            }
        )
    }
}
