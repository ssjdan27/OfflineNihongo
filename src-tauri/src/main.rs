// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::command;
use rusqlite::{Connection, Result};
use serde::Serialize;

#[derive(Serialize)]
struct Kanji {
    meaning: String,
    onyomi: String,
    kunyomi: String,
    jlpt_level: i32,
}

#[command]
fn get_kanji(character: String) -> Result<Kanji, String> {
    println!("Searching for kanji: {}", character); // debug line

    let conn = Connection::open("data/kanji.db").map_err(|e| {
        println!("DB open error: {:?}", e);
        e.to_string()
    })?;

    let mut stmt = conn.prepare(
        "SELECT meaning, onyomi, kunyomi, jlpt_level FROM kanji WHERE character = ?1",
    ).map_err(|e| {
        println!("Prepare error: {:?}", e);
        e.to_string()
    })?;

    let kanji = stmt.query_row([character], |row| {
        Ok(Kanji {
            meaning: row.get(0)?,
            onyomi: row.get(1)?,
            kunyomi: row.get(2)?,
            jlpt_level: row.get(3)?,
        })
    }).map_err(|e| {
        println!("Query error: {:?}", e);
        e.to_string()
    })?;

    Ok(kanji)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_kanji])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}