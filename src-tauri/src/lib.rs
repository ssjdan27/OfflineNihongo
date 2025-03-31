use tauri::{command, AppHandle, Manager};
use rusqlite::{Connection, Result};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
struct Kanji {
    character: String,
    stroke_count: i32,
    grade: i32,
    jlpt_level: i32,
    frequency: i32,
    onyomi: String,
    kunyomi: String,
    meanings: Vec<String>,
    nanori: Vec<String>
}

#[command]
fn get_kanji(character: String, app: AppHandle) -> Result<Kanji, String> {
    println!("Searching for kanji: {}", character);

    // 🔍 Find the correct path to the bundled resource
    let db_path: PathBuf = app
        .path()
        .resource_dir()
        .map_err(|e| {
            println!("Resource dir error: {:?}", e);
            e.to_string()
        })?
        .join("data/kanji.db");

    println!("Resolved DB path: {:?}", db_path);

    let conn = Connection::open(db_path).map_err(|e| {
        println!("DB open error: {:?}", e);
        e.to_string()
    })?;

    let mut stmt = conn.prepare("SELECT * FROM kanji WHERE character = ?1").map_err(|e| {
        println!("Prepare error: {:?}", e);
        e.to_string()
    })?;

    let kanji = stmt.query_row([character], |row| {
        Ok(Kanji {
            character: row.get(0)?,
            stroke_count: row.get(1)?,
            grade: row.get(2)?,
            jlpt_level: row.get(3)?,
            frequency: row.get(4)?,
            onyomi: row.get(5)?,
            kunyomi: row.get(6)?,
            meanings: row.get::<_, String>(7)?.split(';').map(String::from).collect(),
            nanori: row.get::<_, String>(8)?.split(',').map(String::from).collect()
        })
    }).map_err(|e| {
        println!("Query error: {:?}", e);
        e.to_string()
    })?;

    Ok(kanji)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_kanji])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}