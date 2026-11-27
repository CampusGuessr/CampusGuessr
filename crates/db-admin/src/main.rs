use campus_db::{init_database, insert_file, load_all, open_admin, open_read_only};
use std::path::Path;

const DB_PATH: &str = "campusguessr.db";
const PHOTO_DIR: &str = "data/photos";

#[derive(Debug, PartialEq)]
enum Command {
    Init,
    List,
    Add {
        file_name: String,
        latitude: f64,
        longitude: f64,
    },
}

fn parse_args(args: &[String]) -> Result<Command, String> {
    match args.first().map(String::as_str) {
        Some("init") => Ok(Command::Init),
        Some("list") => Ok(Command::List),
        Some("add") => {
            if args.len() < 4 {
                return Err("Add braucht vier Argumente. Aufruf wie: add <dateiname> <breite> <laenge> erwartet".to_string());
            }
            let latitude: f64 = args[2]
                .parse()
                .map_err(|_| format!("Breite erwartet, {:?} erhalten", args[2]))?;
            let longitude: f64 = args[3]
                .parse()
                .map_err(|_| format!("Länge erwartet, {:?} erhalten", args[3]))?;
            Ok(Command::Add {
                file_name: args[1].clone(),
                latitude,
                longitude,
            })
        }
        Some(other) => Err(format!(
            "Eingegebener Befehl {other} nicht gültig, nur init, list und add gültig"
        )),
        None => Err("keine Eingabe. init, list und add gültige Befehle".to_string()),
    }
}

fn run(command: Command) -> Result<(), String> {
    match command {
        Command::Init => {
            init_database(Path::new(DB_PATH)).map_err(|error| error.to_string())?;
            println!("Datenbank {DB_PATH} initialisiert");
            Ok(())
        }
        Command::List => {
            let conn = open_read_only(Path::new(DB_PATH)).map_err(|error| {
                format!(
                    "Datenbank mit dem Pfad {} nicht lesbar {error}. Zuerst initialisieren",
                    DB_PATH
                )
            })?;
            let entries = load_all(&conn).map_err(|error| error.to_string())?;
            for entry in &entries {
                println!(
                    "id: {} file_name: {} latitude: {} longitude: {}",
                    entry.id, entry.file_name, entry.latitude, entry.longitude
                );
            }
            println!("{} Einträge", entries.len());
            Ok(())
        }
        Command::Add {
            file_name,
            latitude,
            longitude,
        } => {
            let photo_path = Path::new(PHOTO_DIR).join(&file_name);
            if !photo_path.is_file() {
                return Err(format!(
                    "Bilddatei mit dem Namen {} existiert nicht",
                    file_name
                ));
            }
            if !Path::new(DB_PATH).exists() {
                return Err(format!(
                    "Die Datenbank mit dem Pfad {} existiert nicht",
                    DB_PATH
                ));
            }
            let conn = open_admin(Path::new(DB_PATH)).map_err(|error| error.to_string())?;
            let id = insert_file(&conn, &file_name, latitude, longitude)
                .map_err(|error| format!("Eintrag nicht möglich: {error}"))?;
            println!(
                "Die Datei {} mit der id {} wurde in die Datenbank eingefügt",
                file_name, id
            );
            Ok(())
        }
    }
}
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = parse_args(&args).and_then(run);
    if let Err(message) = result {
        eprintln!("{message}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_init() {
        let args = vec!["init".to_string()];
        let result = parse_args(&args);
        assert_eq!(
            result,
            Ok(Command::Init),
            "Ok(Init) erwartet, {:?} erhalten",
            result
        );
    }

    #[test]
    fn parse_list() {
        let args = vec!["list".to_string()];
        let result = parse_args(&args);
        assert_eq!(
            result,
            Ok(Command::List),
            "Ok(List) erwartet, {:?} erhalten",
            result
        );
    }
    #[test]
    fn parse_unknown_command() {
        let args = vec!["foo".to_string()];
        let result = parse_args(&args);
        assert!(
            matches!(&result, Err(message) if message.contains("foo")),
            "Error mit dem Wort foo erwartet, {:?} erhalten",
            result
        );
    }

    #[test]
    fn parse_without_arguments() {
        let args: Vec<String> = vec![];
        let result = parse_args(&args);
        assert!(result.is_err(), "Err erwartet, {:?} erhalten", result)
    }
    #[test]
    fn parse_add() {
        let args = vec![
            "add".to_string(),
            "a.jpg".to_string(),
            "1.23".to_string(),
            "4.56".to_string(),
        ];
        let result = parse_args(&args);
        assert_eq!(
            result,
            Ok(Command::Add {
                file_name: "a.jpg".to_string(),
                latitude: 1.23,
                longitude: 4.56
            }),
            "Ok(Add) erwartet, {:?} erhalten",
            result
        )
    }
    #[test]
    fn parse_add_without_longitude() {
        let args = vec!["add".to_string(), "a.jpg".to_string(), "1.23".to_string()];
        let result = parse_args(&args);
        assert!(
            matches!(&result, Err(message) if message.contains("<dateiname>")),
            "Err mit Aufrufhinweis erwartet, {:?} erhalten",
            result
        );
    }
    #[test]
    fn parse_add_with_non_numeric_latitude() {
        let args = vec![
            "add".to_string(),
            "a.jpg".to_string(),
            "abc".to_string(),
            "4.56".to_string(),
        ];
        let result = parse_args(&args);
        assert!(
            matches!(&result, Err(message) if message.contains("abc")),
            "Err mit dem Wort abc erwartet, {:?} erhalten",
            result
        );
    }
    #[test]
    fn parse_add_with_non_numeric_longitude() {
        let args = vec![
            "add".to_string(),
            "a.jpg".to_string(),
            "1.23".to_string(),
            "xyz".to_string(),
        ];
        let result = parse_args(&args);
        assert!(
            matches!(&result, Err(message) if message.contains("xyz")),
            "Err mit dem Wort xyz erwartet, {:?} erhalten",
            result
        );
    }
}
