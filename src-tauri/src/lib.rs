use rusqlite::{Connection, Result};
use serde::Serialize;
use std::path::PathBuf;
use tauri::{command, AppHandle, Manager};

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
    nanori: Vec<String>,
}

#[command]
fn get_kanji(character: String, app: AppHandle) -> Result<Kanji, String> {
    println!("Searching for kanji: {}", character);

    // Find the correct path to the bundled resource
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

    let mut stmt = conn
        .prepare("SELECT * FROM kanji WHERE character = ?1")
        .map_err(|e| {
            println!("Prepare error: {:?}", e);
            e.to_string()
        })?;

    let kanji = stmt
        .query_row([character], |row| {
            Ok(Kanji {
                character: row.get(0)?,
                stroke_count: row.get(1)?,
                grade: row.get(2)?,
                jlpt_level: row.get(3)?,
                frequency: row.get(4)?,
                onyomi: row.get(5)?,
                kunyomi: row.get(6)?,
                meanings: row
                    .get::<_, String>(7)?
                    .split(';')
                    .map(String::from)
                    .collect(),
                nanori: row
                    .get::<_, String>(8)?
                    .split(',')
                    .map(String::from)
                    .collect(),
            })
        })
        .map_err(|e| {
            println!("Query error: {:?}", e);
            e.to_string()
        })?;

    Ok(kanji)
}

#[tauri::command]
fn get_kanji_svg(character: String, app: AppHandle) -> Result<String, String> {
    if character.chars().count() != 1 {
        return Err("Only one character is allowed".into());
    }

    let unicode_hex = format!("{:05x}", character.chars().next().unwrap() as u32);
    let svg_filename = format!("{}.svg", unicode_hex);

    // Use the resource_dir() to get the correct path to bundled resources
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource directory: {}", e))?;
    
    // Join with the kanji_svg directory and filename
    let svg_path = resource_dir.join("data/kanji_svg").join(&svg_filename);

    println!("Looking for SVG at path: {:?}", svg_path);

    // Check if the file exists
    if svg_path.exists() {
        println!("SVG found at: {:?}", svg_path);
        
        // Read the file content instead of returning the path
        let svg_content = std::fs::read_to_string(&svg_path)
            .map_err(|e| format!("Failed to read SVG file: {}", e))?;
        
        // Return the SVG content directly
        Ok(svg_content)
    } else {
        Err(format!("SVG not found for {} (looked at: {:?})", character, svg_path))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![get_kanji, get_kanji_svg])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
