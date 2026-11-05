mod game_times;
mod kana;
mod kanji;
mod srs;
mod storage;

use chrono::Local;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use game_times::GameTimes;
use kana::KanaChar;
use kanji::{Kanji, KanjiLookup};
use srs::{ReviewHistory, ReviewSession, SrsCard, StudyStats};
use storage::{lock_data, read_json_or_default, write_json_atomic};

const CARDS_FILE: &str = "srs_cards.json";
const HISTORY_FILE: &str = "review_history.json";
const LEGACY_WEEKLY_STATS_FILE: &str = "weekly_stats.json";
const GAME_TIMES_FILE: &str = "game_times.json";

const SESSION_DUE_LIMIT: usize = 20;
const SESSION_NEW_LIMIT: usize = 20;

fn resource_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource directory: {}", e))
}

fn data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))
}

fn open_kanji_db(app: &AppHandle) -> Result<rusqlite::Connection, String> {
    kanji::open_db(&resource_dir(app)?.join("data/kanji.db"))
}

fn load_cards(dir: &std::path::Path) -> Result<Vec<SrsCard>, String> {
    read_json_or_default(&dir.join(CARDS_FILE))
}

/// Loads the review history, converting the `weekly_stats.json` file written
/// by earlier versions if there is no history file yet.
fn load_history(dir: &std::path::Path) -> Result<ReviewHistory, String> {
    let path = dir.join(HISTORY_FILE);
    if path.exists() {
        return read_json_or_default(&path);
    }
    let legacy: HashMap<String, (i32, f32)> =
        read_json_or_default(&dir.join(LEGACY_WEEKLY_STATS_FILE)).unwrap_or_default();
    Ok(srs::history_from_legacy_weekly_stats(&legacy))
}

#[tauri::command]
fn get_all_kanji(app: AppHandle) -> Result<Vec<Kanji>, String> {
    kanji::all_kanji(&open_kanji_db(&app)?)
}

#[tauri::command]
fn get_kanji(character: String, app: AppHandle) -> Result<KanjiLookup, String> {
    kanji::lookup(&open_kanji_db(&app)?, &character)
}

#[tauri::command]
fn get_kanji_svg(character: String, app: AppHandle) -> Result<String, String> {
    kanji::kanji_svg(&resource_dir(&app)?, &character)
}

#[tauri::command]
fn get_kana_svg(character: String, app: AppHandle) -> Result<String, String> {
    kanji::kana_svg(&resource_dir(&app)?, &character)
}

#[tauri::command]
fn get_kana_data(app: AppHandle) -> Result<Vec<KanaChar>, String> {
    let path = resource_dir(&app)?.join("data/kana.json");
    let json = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    kana::parse_kana_json(&json)
}

/// Saves `time_ms` if it is a new best for `game_key`; returns whether it was.
#[tauri::command]
async fn save_best_time(app: AppHandle, game_key: String, time_ms: u64) -> Result<bool, String> {
    let path = data_dir(&app)?.join(GAME_TIMES_FILE);
    let _guard = lock_data();
    let mut times: GameTimes = read_json_or_default(&path)?;
    let is_best = times.record(&game_key, time_ms);
    if is_best {
        write_json_atomic(&path, &times)?;
    }
    Ok(is_best)
}

#[tauri::command]
async fn get_best_times(app: AppHandle) -> Result<HashMap<String, u64>, String> {
    let path = data_dir(&app)?.join(GAME_TIMES_FILE);
    let _guard = lock_data();
    let times: GameTimes = read_json_or_default(&path)?;
    Ok(times.best_times)
}

#[tauri::command]
async fn get_study_stats(app: AppHandle) -> Result<StudyStats, String> {
    let dir = data_dir(&app)?;
    let _guard = lock_data();
    let cards = load_cards(&dir)?;
    let history = load_history(&dir)?;
    let today = Local::now().date_naive();
    Ok(srs::compute_stats(&cards, &history, today, &Local))
}

#[tauri::command]
async fn start_review_session(app: AppHandle, session_type: String) -> Result<ReviewSession, String> {
    let dir = data_dir(&app)?;
    let _guard = lock_data();
    let cards = load_cards(&dir)?;
    let today = Local::now().date_naive();
    srs::build_session(
        cards,
        &session_type,
        today,
        &Local,
        SESSION_DUE_LIMIT,
        SESSION_NEW_LIMIT,
    )
}

#[tauri::command]
async fn submit_card_review(
    app: AppHandle,
    card_id: String,
    grade: i32,
    review_type: String,
) -> Result<SrsCard, String> {
    // Meaning and reading reviews are scheduled the same way for now.
    let _ = review_type;

    let dir = data_dir(&app)?;
    let _guard = lock_data();
    let mut cards = load_cards(&dir)?;
    let now = Local::now();

    let card = cards
        .iter_mut()
        .find(|c| c.id == card_id)
        .ok_or_else(|| "Card not found".to_string())?;
    srs::apply_review(card, grade, &now)?;
    let updated = card.clone();

    write_json_atomic(&dir.join(CARDS_FILE), &cards)?;

    let mut history = load_history(&dir)?;
    srs::record_review(&mut history, now.date_naive(), grade >= 3);
    write_json_atomic(&dir.join(HISTORY_FILE), &history)?;

    Ok(updated)
}

/// Adds up to `count` new kanji cards and returns how many were added.
#[tauri::command]
async fn add_new_cards(app: AppHandle, count: i32) -> Result<i32, String> {
    let dir = data_dir(&app)?;
    let conn = open_kanji_db(&app)?;
    let _guard = lock_data();
    let mut cards = load_cards(&dir)?;

    let existing: HashSet<String> = cards.iter().map(|c| c.character.clone()).collect();
    let picked = kanji::pick_new_card_characters(&conn, &existing, count.max(0) as usize)?;
    if picked.is_empty() {
        return Ok(0);
    }

    let now = Local::now();
    cards.extend(picked.iter().map(|character| SrsCard::new(character, &now)));
    write_json_atomic(&dir.join(CARDS_FILE), &cards)?;
    Ok(picked.len() as i32)
}

/// Characters of every kanji in the flashcard deck.
#[tauri::command]
async fn get_deck_characters(app: AppHandle) -> Result<Vec<String>, String> {
    let dir = data_dir(&app)?;
    let _guard = lock_data();
    Ok(load_cards(&dir)?.into_iter().map(|c| c.character).collect())
}

/// The flashcard for `character`, if it is in the deck.
#[tauri::command]
async fn get_card(app: AppHandle, character: String) -> Result<Option<SrsCard>, String> {
    let dir = data_dir(&app)?;
    let _guard = lock_data();
    Ok(load_cards(&dir)?.into_iter().find(|c| c.character == character))
}

/// Adds one specific kanji to the flashcards.
#[tauri::command]
async fn add_card(app: AppHandle, character: String) -> Result<SrsCard, String> {
    let kanji = kanji::lookup(&open_kanji_db(&app)?, &character)?;
    if kanji.meanings.is_empty() && kanji.onyomi.is_empty() && kanji.kunyomi.is_empty() {
        return Err(format!("{} has no meanings or readings to study", character));
    }

    let dir = data_dir(&app)?;
    let _guard = lock_data();
    let mut cards = load_cards(&dir)?;
    let card = srs::add_card(&mut cards, &character, &Local::now())?;
    write_json_atomic(&dir.join(CARDS_FILE), &cards)?;
    Ok(card)
}

/// Removes a kanji from the flashcards; returns whether it was in the deck.
#[tauri::command]
async fn remove_card(app: AppHandle, character: String) -> Result<bool, String> {
    let dir = data_dir(&app)?;
    let _guard = lock_data();
    let mut cards = load_cards(&dir)?;
    let removed = srs::remove_card(&mut cards, &character);
    if removed {
        write_json_atomic(&dir.join(CARDS_FILE), &cards)?;
    }
    Ok(removed)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            get_kanji,
            get_all_kanji,
            get_kanji_svg,
            get_kana_svg,
            get_kana_data,
            save_best_time,
            get_best_times,
            get_study_stats,
            start_review_session,
            submit_card_review,
            add_new_cards,
            get_deck_characters,
            get_card,
            add_card,
            remove_card
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application :(");
}
