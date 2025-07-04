use rusqlite::{Connection, Result};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

#[derive(Serialize)]
struct Kanji {
    character: String,
    stroke_count: i32,
    grade: i32,
    jlpt_level: i32,
    frequency: i32,
    onyomi: String,
    kunyomi: String,
    meanings: String,
    nanori: String,
}

#[derive(Serialize)]
struct KanjiLookup {
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

#[derive(Serialize)]
struct KanaChar {
    character: String,
    romaji: String,
    kana_type: String, // "hiragana" or "katakana"
    sound_type: String, // "seion", "dakuon", "handakuon"
    complexity: String, // "basic" (single), "combination" (ya/yu/yo)
}

#[derive(Serialize, Deserialize, Default)]
struct GameTimes {
    best_times: HashMap<String, u64>,
}

#[tauri::command]
fn get_all_kanji(app: AppHandle) -> Result<Vec<Kanji>, String> {
    println!("Fetching all kanji from database");

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
        .prepare("SELECT character, stroke_count, grade, jlpt_level, frequency, onyomi, kunyomi, meanings, nanori FROM kanji")
        .map_err(|e| {
            println!("Prepare error: {:?}", e);
            e.to_string()
        })?;

    let kanji_iter = stmt
        .query_map([], |row| {
            Ok(Kanji {
                character: row.get(0)?,
                stroke_count: row.get(1)?,
                grade: row.get(2)?,
                jlpt_level: row.get(3)?,
                frequency: row.get(4)?,
                onyomi: row.get::<_, String>(5).unwrap_or_default(),
                kunyomi: row.get::<_, String>(6).unwrap_or_default(),
                meanings: row.get::<_, String>(7).unwrap_or_default(),
                nanori: row.get::<_, String>(8).unwrap_or_default(),
            })
        })
        .map_err(|e| {
            println!("Query error: {:?}", e);
            e.to_string()
        })?;

    let mut kanji_list = Vec::new();
    for kanji in kanji_iter {
        kanji_list.push(kanji.map_err(|e| e.to_string())?);
    }

    println!("Found {} kanji in database", kanji_list.len());
    Ok(kanji_list)
}

#[tauri::command]
fn get_kanji(character: String, app: AppHandle) -> Result<KanjiLookup, String> {
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
            let meanings_str: String = row.get::<_, String>(7).unwrap_or_default();
            let nanori_str: String = row.get::<_, String>(8).unwrap_or_default();
            
            // Split meanings by semicolon and filter out empty strings
            let meanings: Vec<String> = meanings_str
                .split(';')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
                
            // Split nanori by comma and filter out empty strings
            let nanori: Vec<String> = nanori_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            Ok(KanjiLookup {
                character: row.get(0)?,
                stroke_count: row.get(1)?,
                grade: row.get(2)?,
                jlpt_level: row.get(3)?,
                frequency: row.get(4)?,
                onyomi: row.get::<_, String>(5).unwrap_or_default(),
                kunyomi: row.get::<_, String>(6).unwrap_or_default(),
                meanings,
                nanori,
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

#[tauri::command]
fn get_kana_data(app: AppHandle) -> Result<Vec<KanaChar>, String> {
    println!("Fetching kana data from JSON file");

    // Find the correct path to the bundled resource
    let json_path: PathBuf = app
        .path()
        .resource_dir()
        .map_err(|e| {
            println!("Resource dir error: {:?}", e);
            e.to_string()
        })?
        .join("data/kana.json");

    println!("Resolved JSON path: {:?}", json_path);

    let json_content = std::fs::read_to_string(&json_path)
        .map_err(|e| {
            println!("Failed to read kana.json: {:?}", e);
            e.to_string()
        })?;

    let kana_data: serde_json::Value = serde_json::from_str(&json_content)
        .map_err(|e| {
            println!("Failed to parse kana.json: {:?}", e);
            e.to_string()
        })?;

    let mut kana_list = Vec::new();

    // Parse the nested JSON structure
    if let Some(obj) = kana_data.as_object() {
        for (_consonant_group, vowel_group) in obj {
            if let Some(vowel_obj) = vowel_group.as_object() {
                for (_vowel, types) in vowel_obj {
                    if let Some(types_obj) = types.as_object() {
                        for (type_name, kana_info) in types_obj {
                            if let Some(kana_obj) = kana_info.as_object() {
                                if let (Some(hiragana), Some(katakana), Some(romaji)) = (
                                    kana_obj.get("Hiragana").and_then(|v| v.as_str()),
                                    kana_obj.get("Katakana").and_then(|v| v.as_str()),
                                    kana_obj.get("Romaji").and_then(|v| v.as_str()),
                                ) {
                                    // Determine sound type
                                    let sound_type = match type_name.as_str() {
                                        "Seion" => "seion",
                                        "Dakuon" => "dakuon", 
                                        "Handakuon" => "handakuon",
                                        _ => "seion"
                                    }.to_string();
                                    
                                    // Determine complexity based on romaji length
                                    let complexity = if romaji.len() > 2 {
                                        "combination"
                                    } else {
                                        "basic"
                                    }.to_string();
                                    
                                    // Add hiragana
                                    kana_list.push(KanaChar {
                                        character: hiragana.to_string(),
                                        romaji: romaji.to_string(),
                                        kana_type: "hiragana".to_string(),
                                        sound_type: sound_type.clone(),
                                        complexity: complexity.clone(),
                                    });
                                    
                                    // Add katakana
                                    kana_list.push(KanaChar {
                                        character: katakana.to_string(),
                                        romaji: romaji.to_string(),
                                        kana_type: "katakana".to_string(),
                                        sound_type,
                                        complexity,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    println!("Found {} kana characters", kana_list.len());
    Ok(kana_list)
}

#[tauri::command]
async fn save_best_time(app: AppHandle, game_key: String, time_ms: u64) -> Result<(), String> {
    let app_dir = app.path().app_local_data_dir()
        .map_err(|e| format!("Failed to get app directory: {}", e))?;
    
    // Create app directory if it doesn't exist
    fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create app directory: {}", e))?;
    
    let times_file = app_dir.join("game_times.json");
    
    // Load existing times or create new
    let mut game_times = if times_file.exists() {
        let content = fs::read_to_string(&times_file)
            .map_err(|e| format!("Failed to read times file: {}", e))?;
        serde_json::from_str(&content)
            .unwrap_or_default()
    } else {
        GameTimes::default()
    };
    
    // Update best time if this is better
    let current_best = game_times.best_times.get(&game_key).copied().unwrap_or(u64::MAX);
    if time_ms < current_best {
        game_times.best_times.insert(game_key, time_ms);
        
        // Save updated times
        let content = serde_json::to_string_pretty(&game_times)
            .map_err(|e| format!("Failed to serialize times: {}", e))?;
        fs::write(&times_file, content)
            .map_err(|e| format!("Failed to write times file: {}", e))?;
    }
    
    Ok(())
}

#[tauri::command]
async fn get_best_times(app: AppHandle) -> Result<HashMap<String, u64>, String> {
    let app_dir = app.path().app_local_data_dir()
        .map_err(|e| format!("Failed to get app directory: {}", e))?;
    
    let times_file = app_dir.join("game_times.json");
    
    if !times_file.exists() {
        return Ok(HashMap::new());
    }
    
    let content = fs::read_to_string(&times_file)
        .map_err(|e| format!("Failed to read times file: {}", e))?;
    
    let game_times: GameTimes = serde_json::from_str(&content)
        .unwrap_or_default();
    
    Ok(game_times.best_times)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![get_kanji, get_all_kanji, get_kanji_svg, get_kana_data, save_best_time, get_best_times])
        .run(tauri::generate_context!())
        .expect("error while running tauri application :(");
}
