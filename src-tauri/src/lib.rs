use rusqlite::{Connection, Result};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use chrono::{DateTime, Utc, Duration, NaiveDateTime, NaiveDate};

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

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SrsCard {
    id: String,
    character: String,
    level: i32,
    interval: i32,
    ease_factor: f32,
    next_review: String, // ISO date string
    total_reviews: i32,
    correct_reviews: i32,
    created_at: String,
    last_reviewed: Option<String>,
    streak: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct ReviewSession {
    cards_due: Vec<SrsCard>,
    cards_new: Vec<SrsCard>,
    session_stats: SessionStats,
}

#[derive(Serialize, Deserialize, Debug)]
struct SessionStats {
    total_reviews: i32,
    correct_answers: i32,
    session_time: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct StudyStats {
    total_cards: i32,
    cards_due: i32,
    cards_new: i32,
    cards_learning: i32,
    cards_mature: i32,
    daily_streak: i32,
    total_reviews: i32,
    accuracy: f32,
    reviews_this_week: Vec<i32>,
    accuracy_this_week: Vec<f32>,
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
        
        // Clean up the SVG content by removing XML declaration and DOCTYPE
        let cleaned_svg = clean_svg_content(&svg_content);
        
        // Return the cleaned SVG content
        Ok(cleaned_svg)
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

// Helper function to calculate daily streak
fn calculate_daily_streak(app_dir: &PathBuf) -> Result<i32, String> {
    let review_log_file = app_dir.join("review_log.json");
    
    if !review_log_file.exists() {
        return Ok(0);
    }
    
    let content = fs::read_to_string(&review_log_file)
        .map_err(|e| format!("Failed to read review log: {}", e))?;
    
    let review_log: HashMap<String, bool> = serde_json::from_str(&content)
        .unwrap_or_default();
    
    let mut streak = 0;
    let today = chrono::Utc::now().date_naive();
    
    for days_back in 0..365 {
        let date = today - chrono::Duration::days(days_back);
        let date_str = date.format("%Y-%m-%d").to_string();
        
        if *review_log.get(&date_str).unwrap_or(&false) {
            streak += 1;
        } else if days_back > 0 {
            // Break the streak if we hit a day without reviews (but not today)
            break;
        }
    }
    
    Ok(streak)
}

// Helper function to calculate weekly stats
fn calculate_weekly_stats(app_dir: &PathBuf) -> Result<(Vec<i32>, Vec<f32>), String> {
    let review_log_file = app_dir.join("weekly_stats.json");
    
    if !review_log_file.exists() {
        return Ok((vec![0; 7], vec![0.0; 7]));
    }
    
    let content = fs::read_to_string(&review_log_file)
        .map_err(|e| format!("Failed to read weekly stats: {}", e))?;
    
    let weekly_stats: HashMap<String, (i32, f32)> = serde_json::from_str(&content)
        .unwrap_or_default();
    
    let mut reviews_this_week = vec![0; 7];
    let mut accuracy_this_week = vec![0.0; 7];
    
    let today = chrono::Utc::now().date_naive();
    
    for i in 0..7 {
        let date = today - chrono::Duration::days(i);
        let date_str = date.format("%Y-%m-%d").to_string();
        
        if let Some((reviews, accuracy)) = weekly_stats.get(&date_str) {
            reviews_this_week[6 - i as usize] = *reviews;
            accuracy_this_week[6 - i as usize] = *accuracy;
        }
    }
    
    Ok((reviews_this_week, accuracy_this_week))
}

// Helper function to record daily review
fn record_daily_review(app_dir: &PathBuf, is_correct: bool) -> Result<(), String> {
    let review_log_file = app_dir.join("review_log.json");
    let weekly_stats_file = app_dir.join("weekly_stats.json");
    
    let today = chrono::Utc::now().date_naive();
    let date_str = today.format("%Y-%m-%d").to_string();
    
    // Update daily review log
    let mut review_log: HashMap<String, bool> = if review_log_file.exists() {
        let content = fs::read_to_string(&review_log_file)
            .map_err(|e| format!("Failed to read review log: {}", e))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    };
    
    review_log.insert(date_str.clone(), true);
    
    let content = serde_json::to_string_pretty(&review_log)
        .map_err(|e| format!("Failed to serialize review log: {}", e))?;
    fs::write(&review_log_file, content)
        .map_err(|e| format!("Failed to write review log: {}", e))?;
    
    // Update weekly stats
    let mut weekly_stats: HashMap<String, (i32, f32)> = if weekly_stats_file.exists() {
        let content = fs::read_to_string(&weekly_stats_file)
            .map_err(|e| format!("Failed to read weekly stats: {}", e))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    };
    
    let (current_reviews, current_accuracy) = weekly_stats.get(&date_str).unwrap_or(&(0, 0.0));
    let new_reviews = current_reviews + 1;
    let new_accuracy = if new_reviews == 1 {
        if is_correct { 1.0 } else { 0.0 }
    } else {
        let total_correct = (current_accuracy * *current_reviews as f32) + if is_correct { 1.0 } else { 0.0 };
        total_correct / new_reviews as f32
    };
    
    weekly_stats.insert(date_str, (new_reviews, new_accuracy));
    
    let content = serde_json::to_string_pretty(&weekly_stats)
        .map_err(|e| format!("Failed to serialize weekly stats: {}", e))?;
    fs::write(&weekly_stats_file, content)
        .map_err(|e| format!("Failed to write weekly stats: {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn get_study_stats(app: AppHandle) -> Result<StudyStats, String> {
    let app_dir = app.path().app_local_data_dir()
        .map_err(|e| format!("Failed to get app directory: {}", e))?;
    
    fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create app directory: {}", e))?;
    
    let cards_file = app_dir.join("srs_cards.json");
    
    if !cards_file.exists() {
        // Initialize with empty stats if no cards exist
        return Ok(StudyStats {
            total_cards: 0,
            cards_due: 0,
            cards_new: 0,
            cards_learning: 0,
            cards_mature: 0,
            daily_streak: 0,
            total_reviews: 0,
            accuracy: 0.0,
            reviews_this_week: vec![0; 7],
            accuracy_this_week: vec![0.0; 7],
        });
    }
    
    let content = fs::read_to_string(&cards_file)
        .map_err(|e| format!("Failed to read cards file: {}", e))?;
    
    let cards: Vec<SrsCard> = serde_json::from_str(&content)
        .unwrap_or_default();
    
    let now = chrono::Utc::now();
    let today = now.date_naive();
    
    let mut cards_due = 0;
    let mut cards_learning = 0;
    let mut cards_mature = 0;
    let mut total_reviews = 0;
    let mut correct_reviews = 0;
    
    for card in &cards {
        total_reviews += card.total_reviews;
        correct_reviews += card.correct_reviews;
        
        // Try to parse the next_review date with better error handling
        let next_review = match chrono::DateTime::parse_from_rfc3339(&card.next_review) {
            Ok(dt) => dt.naive_utc(),
            Err(_) => {
                // Try alternative parsing formats
                match chrono::NaiveDateTime::parse_from_str(&card.next_review, "%Y-%m-%dT%H:%M:%S%.fZ") {
                    Ok(dt) => dt,
                    Err(_) => {
                        // If all parsing fails, default to epoch
                        chrono::NaiveDate::from_ymd_opt(1970, 1, 1)
                            .unwrap()
                            .and_hms_opt(0, 0, 0)
                            .unwrap()
                    }
                }
            }
        };
        
        if next_review.date() <= today {
            cards_due += 1;
        }
        
        if card.level >= 4 {
            cards_mature += 1;
        } else if card.level > 0 {
            cards_learning += 1;
        }
    }
    
    let accuracy = if total_reviews > 0 {
        correct_reviews as f32 / total_reviews as f32
    } else {
        0.0
    };
    
    // Calculate daily streak and weekly stats (simplified for now)
    let daily_streak = calculate_daily_streak(&app_dir)?;
    let (reviews_this_week, accuracy_this_week) = calculate_weekly_stats(&app_dir)?;
    
    Ok(StudyStats {
        total_cards: cards.len() as i32,
        cards_due,
        cards_new: 20, // Simplified - could be calculated based on available kanji
        cards_learning,
        cards_mature,
        daily_streak,
        total_reviews,
        accuracy,
        reviews_this_week,
        accuracy_this_week,
    })
}

#[tauri::command]
async fn start_review_session(app: AppHandle, session_type: String) -> Result<ReviewSession, String> {
    let app_dir = app.path().app_local_data_dir()
        .map_err(|e| format!("Failed to get app directory: {}", e))?;
    
    let cards_file = app_dir.join("srs_cards.json");
    
    let cards: Vec<SrsCard> = if cards_file.exists() {
        let content = fs::read_to_string(&cards_file)
            .map_err(|e| format!("Failed to read cards file: {}", e))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };
    
    let now = chrono::Utc::now();
    let today = now.date_naive();
    
    let mut cards_due: Vec<SrsCard> = cards.clone().into_iter()
        .filter(|card| {
            // Try to parse the next_review date with better error handling
            let next_review = match chrono::DateTime::parse_from_rfc3339(&card.next_review) {
                Ok(dt) => dt.naive_utc(),
                Err(_) => {
                    // Try alternative parsing formats
                    match chrono::NaiveDateTime::parse_from_str(&card.next_review, "%Y-%m-%dT%H:%M:%S%.fZ") {
                        Ok(dt) => dt,
                        Err(_) => {
                            // If all parsing fails, default to epoch (will be considered due)
                            chrono::NaiveDate::from_ymd_opt(1970, 1, 1)
                                .unwrap()
                                .and_hms_opt(0, 0, 0)
                                .unwrap()
                        }
                    }
                }
            };
            next_review.date() <= today
        })
        .collect();
    
    let cards_new: Vec<SrsCard> = match session_type.as_str() {
        "new" | "mixed" => {
            // Generate some new cards (simplified - in real implementation, 
            // this would pull from the kanji database)
            vec![]
        },
        _ => vec![]
    };
    
    // Limit session size
    cards_due.truncate(20);
    
    Ok(ReviewSession {
        cards_due,
        cards_new,
        session_stats: SessionStats {
            total_reviews: 0,
            correct_answers: 0,
            session_time: 0,
        },
    })
}

#[tauri::command]
async fn submit_card_review(
    app: AppHandle, 
    card_id: String, 
    grade: i32, 
    _review_type: String
) -> Result<SrsCard, String> {
    let app_dir = app.path().app_local_data_dir()
        .map_err(|e| format!("Failed to get app directory: {}", e))?;
    
    let cards_file = app_dir.join("srs_cards.json");
    
    let mut cards: Vec<SrsCard> = if cards_file.exists() {
        let content = fs::read_to_string(&cards_file)
            .map_err(|e| format!("Failed to read cards file: {}", e))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };
    
    // Find and update the card
    if let Some(card) = cards.iter_mut().find(|c| c.id == card_id) {
        card.total_reviews += 1;
        card.last_reviewed = Some(chrono::Utc::now().to_rfc3339());
        
        // Enhanced SRS algorithm based on SM-2
        let previous_interval = card.interval;
        let mut new_interval = card.interval;
        let mut new_ease_factor = card.ease_factor;
        let mut new_level = card.level;
        
        if grade >= 3 {
            // Correct answer
            card.correct_reviews += 1;
            card.streak += 1;
            
            match grade {
                3 => {
                    // Good (correct response with some difficulty)
                    new_ease_factor = card.ease_factor; // No change
                    if card.level == 0 {
                        // First time seeing this card
                        new_interval = 1;
                        new_level = 1;
                    } else if card.level == 1 {
                        // Second time
                        new_interval = 6;
                        new_level = 2;
                    } else {
                        // Subsequent reviews
                        new_interval = (previous_interval as f32 * new_ease_factor) as i32;
                        new_level += 1;
                    }
                },
                4 => {
                    // Easy (perfect recall)
                    new_ease_factor = (card.ease_factor + 0.15).min(3.0);
                    if card.level == 0 {
                        // First time - skip to mature faster
                        new_interval = 4;
                        new_level = 2;
                    } else if card.level == 1 {
                        // Second time - longer interval
                        new_interval = 10;
                        new_level = 3;
                    } else {
                        // Subsequent reviews - bonus multiplier
                        new_interval = (previous_interval as f32 * new_ease_factor * 1.3) as i32;
                        new_level += 1;
                    }
                },
                _ => {}
            }
        } else {
            // Incorrect answer
            card.streak = 0;
            
            match grade {
                1 => {
                    // Again (complete failure)
                    new_ease_factor = (card.ease_factor - 0.2).max(1.3);
                    new_interval = 1; // Reset to 1 day
                    new_level = 0; // Reset to learning
                },
                2 => {
                    // Hard (incorrect but partially remembered)
                    new_ease_factor = (card.ease_factor - 0.15).max(1.3);
                    new_interval = ((previous_interval as f32 * 0.6) as i32).max(1);
                    new_level = (card.level - 1).max(0);
                },
                _ => {}
            }
        }
        
        // Apply the updates
        card.interval = new_interval.max(1); // Minimum 1 day
        card.ease_factor = new_ease_factor;
        card.level = new_level;
        
        // Calculate next review date
        let next_review = chrono::Utc::now() + chrono::Duration::days(card.interval as i64);
        card.next_review = next_review.to_rfc3339();
        
        // Clone the card before saving to avoid borrow checker issues
        let updated_card = card.clone();
        
        // Save updated cards
        let content = serde_json::to_string_pretty(&cards)
            .map_err(|e| format!("Failed to serialize cards: {}", e))?;
        fs::write(&cards_file, content)
            .map_err(|e| format!("Failed to write cards file: {}", e))?;
        
        // Record review in daily log
        record_daily_review(&app_dir, grade >= 3)?;
        
        Ok(updated_card)
    } else {
        Err("Card not found".to_string())
    }
}

#[tauri::command]
async fn add_new_cards(app: AppHandle, count: i32) -> Result<(), String> {
    let app_dir = app.path().app_local_data_dir()
        .map_err(|e| format!("Failed to get app directory: {}", e))?;
    
    // Get available kanji from database
    let db_path: PathBuf = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource directory: {}", e))?
        .join("data/kanji.db");
    
    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    // Get existing cards
    let cards_file = app_dir.join("srs_cards.json");
    let existing_cards: Vec<SrsCard> = if cards_file.exists() {
        let content = fs::read_to_string(&cards_file)
            .map_err(|e| format!("Failed to read cards file: {}", e))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };
    
    let existing_chars: std::collections::HashSet<String> = existing_cards
        .iter()
        .map(|card| card.character.clone())
        .collect();
    
    // Get new kanji characters with smart prioritization
    // Priority: Grade 1-6 > JLPT N5/N4 > Frequent kanji > Others
    let mut stmt = conn
        .prepare("
            SELECT character, grade, jlpt_level, frequency, stroke_count 
            FROM kanji 
            WHERE frequency > 0 
            ORDER BY 
                CASE 
                    WHEN grade BETWEEN 1 AND 6 THEN 1
                    WHEN jlpt_level IN (4, 5) THEN 2
                    WHEN jlpt_level IN (2, 3) THEN 3
                    WHEN frequency <= 1000 THEN 4
                    ELSE 5
                END,
                grade ASC,
                frequency ASC,
                stroke_count ASC
            LIMIT ?1
        ")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;
    
    let kanji_iter = stmt
        .query_map([count * 3], |row| {
            Ok((
                row.get::<_, String>(0)?,  // character
                row.get::<_, i32>(1)?,     // grade
                row.get::<_, i32>(2)?,     // jlpt_level
                row.get::<_, i32>(3)?,     // frequency
                row.get::<_, i32>(4)?,     // stroke_count
            ))
        })
        .map_err(|e| format!("Failed to query kanji: {}", e))?;
    
    let mut new_cards = existing_cards;
    let now = chrono::Utc::now();
    let mut added_count = 0;
    
    for kanji_result in kanji_iter {
        if added_count >= count {
            break;
        }
        
        if let Ok((character, _grade, _jlpt_level, _frequency, _stroke_count)) = kanji_result {
            if !existing_chars.contains(&character) {
                let card = SrsCard {
                    id: format!("{}_{}", character, now.timestamp()),
                    character: character.clone(),
                    level: 0,
                    interval: 1,
                    ease_factor: 2.5,
                    next_review: now.to_rfc3339(),
                    total_reviews: 0,
                    correct_reviews: 0,
                    created_at: now.to_rfc3339(),
                    last_reviewed: None,
                    streak: 0,
                };
                
                new_cards.push(card);
                added_count += 1;
            }
        }
    }
    
    // Save updated cards
    let content = serde_json::to_string_pretty(&new_cards)
        .map_err(|e| format!("Failed to serialize cards: {}", e))?;
    fs::write(&cards_file, content)
        .map_err(|e| format!("Failed to write cards file: {}", e))?;
    
    Ok(())
}

fn clean_svg_content(svg_content: &str) -> String {
    // Remove XML declaration
    let mut cleaned = svg_content.to_string();
    
    // Remove XML declaration (<?xml ... ?>)
    if let Some(start) = cleaned.find("<?xml") {
        if let Some(end) = cleaned[start..].find("?>") {
            cleaned = cleaned[..start].to_string() + &cleaned[start + end + 2..];
        }
    }
    
    // Remove DOCTYPE declaration (<!DOCTYPE ... ]>)
    if let Some(start) = cleaned.find("<!DOCTYPE") {
        if let Some(end) = cleaned[start..].find("]>") {
            cleaned = cleaned[..start].to_string() + &cleaned[start + end + 2..];
        }
    }
    
    // Remove any remaining comments at the start
    while cleaned.trim_start().starts_with("<!--") {
        if let Some(start) = cleaned.find("<!--") {
            if let Some(end) = cleaned[start..].find("-->") {
                cleaned = cleaned[..start].to_string() + &cleaned[start + end + 3..];
            } else {
                break;
            }
        } else {
            break;
        }
    }
    
    // Trim whitespace
    cleaned.trim().to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![get_kanji, get_all_kanji, get_kanji_svg, get_kana_data, save_best_time, get_best_times, get_study_stats, start_review_session, submit_card_review, add_new_cards])
        .run(tauri::generate_context!())
        .expect("error while running tauri application :(");
}
