mod backup;
mod game_times;
mod kana;
mod kanji;
mod srs;
mod storage;

use chrono::Local;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

use backup::{Backup, ImportSummary};
use game_times::GameTimes;
use kana::KanaChar;
use kanji::{Kanji, KanjiLookup};
use srs::{ReviewHistory, ReviewSession, SrsCard, StudyStats};
use storage::{lock_data, read_json_or_default, write_json_atomic};

const CARDS_FILE: &str = "srs_cards.json";
const HISTORY_FILE: &str = "review_history.json";
const LEGACY_WEEKLY_STATS_FILE: &str = "weekly_stats.json";
const GAME_TIMES_FILE: &str = "game_times.json";
const LEGACY_REVIEW_LOG_FILE: &str = "review_log.json";
const BACKUPS_DIR: &str = "backups";

const DEFAULT_SESSION_LIMIT: u32 = 20;
const MAX_SESSION_LIMIT: u32 = 500;

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

fn load_cards(dir: &Path) -> Result<Vec<SrsCard>, String> {
    read_json_or_default(&dir.join(CARDS_FILE))
}

/// Loads the review history, converting the `weekly_stats.json` file written
/// by earlier versions if there is no history file yet.
fn load_history(dir: &Path) -> Result<ReviewHistory, String> {
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

fn session_limit(limit: Option<u32>) -> usize {
    limit.unwrap_or(DEFAULT_SESSION_LIMIT).clamp(1, MAX_SESSION_LIMIT) as usize
}

/// Starts a study session with at most `due_limit` reviews and `new_limit`
/// new cards (20 each by default).
#[tauri::command]
async fn start_review_session(
    app: AppHandle,
    session_type: String,
    due_limit: Option<u32>,
    new_limit: Option<u32>,
) -> Result<ReviewSession, String> {
    let dir = data_dir(&app)?;
    let _guard = lock_data();
    let cards = load_cards(&dir)?;
    let today = Local::now().date_naive();
    srs::build_session(
        cards,
        &session_type,
        today,
        &Local,
        session_limit(due_limit),
        session_limit(new_limit),
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

fn file_timestamp() -> String {
    Local::now().format("%Y-%m-%d-%H%M%S").to_string()
}

fn current_backup(dir: &Path) -> Result<Backup, String> {
    let times: GameTimes = read_json_or_default(&dir.join(GAME_TIMES_FILE))?;
    Ok(Backup::new(
        Local::now().to_rfc3339(),
        load_cards(dir)?,
        load_history(dir)?,
        times.best_times,
    ))
}

/// Copies the current data files into backups/<reason>-<time>/ in the app
/// data folder before they are replaced. The files are copied as-is, so
/// this works even if one of them can't be parsed.
fn save_safety_copy(dir: &Path, reason: &str) -> Result<PathBuf, String> {
    // Never reuse a folder: two imports in the same second would otherwise
    // overwrite the first copy.
    let base = format!("{}-{}", reason, file_timestamp());
    let mut folder = dir.join(BACKUPS_DIR).join(&base);
    let mut n = 2;
    while folder.exists() {
        folder = dir.join(BACKUPS_DIR).join(format!("{}-{}", base, n));
        n += 1;
    }
    fs::create_dir_all(&folder)
        .map_err(|e| format!("Failed to create {}: {}", folder.display(), e))?;
    for name in [
        CARDS_FILE,
        HISTORY_FILE,
        LEGACY_WEEKLY_STATS_FILE,
        LEGACY_REVIEW_LOG_FILE,
        GAME_TIMES_FILE,
    ] {
        let source = dir.join(name);
        if source.exists() {
            fs::copy(&source, folder.join(name))
                .map_err(|e| format!("Failed to back up {}: {}", source.display(), e))?;
        }
    }
    Ok(folder)
}

/// Writes all study progress to a backup file in the Downloads folder and
/// returns its path.
#[tauri::command]
async fn export_backup(app: AppHandle) -> Result<String, String> {
    let dir = data_dir(&app)?;
    let target_dir = app
        .path()
        .download_dir()
        .or_else(|_| app.path().document_dir())
        .unwrap_or_else(|_| dir.join(BACKUPS_DIR));

    let _guard = lock_data();
    Ok(export_backup_to(&dir, &target_dir)?.to_string_lossy().into_owned())
}

fn export_backup_to(dir: &Path, target_dir: &Path) -> Result<PathBuf, String> {
    let backup = current_backup(dir)?;
    let path = target_dir.join(format!("OfflineNihongo-backup-{}.json", file_timestamp()));
    write_json_atomic(&path, &backup)?;
    Ok(path)
}

/// Replaces the flashcards, review history and best times with the
/// contents of a backup file. The current data is copied aside first.
#[tauri::command]
async fn import_backup(app: AppHandle, contents: String) -> Result<ImportSummary, String> {
    let dir = data_dir(&app)?;
    let _guard = lock_data();
    import_backup_into(&dir, &contents)
}

fn import_backup_into(dir: &Path, contents: &str) -> Result<ImportSummary, String> {
    let backup = backup::parse_backup(contents)?;
    save_safety_copy(dir, "before-import")?;
    write_json_atomic(&dir.join(CARDS_FILE), &backup.cards)?;
    write_json_atomic(&dir.join(HISTORY_FILE), &backup.review_history)?;
    let times = GameTimes {
        best_times: backup.best_times,
    };
    write_json_atomic(&dir.join(GAME_TIMES_FILE), &times)?;

    Ok(ImportSummary {
        cards: backup.cards.len(),
        review_days: backup.review_history.len(),
        best_times: times.best_times.len(),
    })
}

/// Deletes all flashcards and review history (kana game best times are
/// kept). Returns the folder holding a copy of the old data.
#[tauri::command]
async fn reset_flashcards(app: AppHandle) -> Result<String, String> {
    let dir = data_dir(&app)?;
    let _guard = lock_data();
    Ok(reset_flashcards_in(&dir)?.to_string_lossy().into_owned())
}

fn reset_flashcards_in(dir: &Path) -> Result<PathBuf, String> {
    let safety_copy = save_safety_copy(dir, "before-reset")?;
    write_json_atomic(&dir.join(CARDS_FILE), &Vec::<SrsCard>::new())?;
    write_json_atomic(&dir.join(HISTORY_FILE), &ReviewHistory::new())?;
    Ok(safety_copy)
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
            remove_card,
            export_backup,
            import_backup,
            reset_flashcards
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application :(");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "offline-nihongo-lib-test-{}-{}",
            name,
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_sample_data(dir: &Path) {
        let now = Local::now();
        let mut cards = vec![SrsCard::new("水", &now), SrsCard::new("火", &now)];
        srs::apply_review(&mut cards[0], 3, &now).unwrap();
        write_json_atomic(&dir.join(CARDS_FILE), &cards).unwrap();

        let mut history = ReviewHistory::new();
        srs::record_review(&mut history, now.date_naive(), true);
        write_json_atomic(&dir.join(HISTORY_FILE), &history).unwrap();

        let mut times = GameTimes::default();
        times.record("hiragana-seion-basic", 61234);
        write_json_atomic(&dir.join(GAME_TIMES_FILE), &times).unwrap();
    }

    #[test]
    fn export_then_import_restores_everything() {
        let source = temp_dir("export-source");
        write_sample_data(&source);
        let exported = export_backup_to(&source, &source.join("exports")).unwrap();

        let target = temp_dir("export-target");
        let summary =
            import_backup_into(&target, &fs::read_to_string(&exported).unwrap()).unwrap();
        assert_eq!(
            summary,
            ImportSummary {
                cards: 2,
                review_days: 1,
                best_times: 1
            }
        );
        assert_eq!(load_cards(&target).unwrap(), load_cards(&source).unwrap());
        assert_eq!(load_history(&target).unwrap(), load_history(&source).unwrap());
    }

    #[test]
    fn import_keeps_a_copy_of_the_replaced_data_even_if_it_is_corrupted() {
        let dir = temp_dir("import-corrupted");
        fs::write(dir.join(CARDS_FILE), "{ broken").unwrap();
        assert!(load_cards(&dir).is_err());

        let backup = Backup::new("x".into(), vec![], ReviewHistory::new(), HashMap::new());
        import_backup_into(&dir, &serde_json::to_string(&backup).unwrap()).unwrap();

        assert!(load_cards(&dir).unwrap().is_empty());
        let copies: Vec<_> = fs::read_dir(dir.join(BACKUPS_DIR)).unwrap().collect();
        assert_eq!(copies.len(), 1);
        let copy = copies[0].as_ref().unwrap().path().join(CARDS_FILE);
        assert_eq!(fs::read_to_string(copy).unwrap(), "{ broken");
    }

    #[test]
    fn safety_copies_never_overwrite_each_other() {
        let dir = temp_dir("safety-copies");
        write_sample_data(&dir);
        let first = save_safety_copy(&dir, "before-import").unwrap();
        let second = save_safety_copy(&dir, "before-import").unwrap();
        assert_ne!(first, second);
        assert!(first.join(CARDS_FILE).exists() && second.join(CARDS_FILE).exists());
    }

    #[test]
    fn invalid_imports_change_nothing() {
        let dir = temp_dir("import-invalid");
        write_sample_data(&dir);
        let before = fs::read_to_string(dir.join(CARDS_FILE)).unwrap();
        assert!(import_backup_into(&dir, "[1, 2, 3]").is_err());
        assert_eq!(fs::read_to_string(dir.join(CARDS_FILE)).unwrap(), before);
        assert!(!dir.join(BACKUPS_DIR).exists());
    }

    #[test]
    fn reset_clears_flashcards_but_keeps_best_times() {
        let dir = temp_dir("reset");
        write_sample_data(&dir);
        let copy = reset_flashcards_in(&dir).unwrap();

        assert!(load_cards(&dir).unwrap().is_empty());
        assert!(load_history(&dir).unwrap().is_empty());
        let times: GameTimes = read_json_or_default(&dir.join(GAME_TIMES_FILE)).unwrap();
        assert_eq!(times.best_times.len(), 1);
        assert!(copy.join(CARDS_FILE).exists());
    }

    #[test]
    fn session_limits_are_clamped() {
        assert_eq!(session_limit(None), 20);
        assert_eq!(session_limit(Some(0)), 1);
        assert_eq!(session_limit(Some(50)), 50);
        assert_eq!(session_limit(Some(100_000)), 500);
    }
}
