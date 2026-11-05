//! Read-only access to the bundled kanji database and stroke-order SVGs.

use rusqlite::{Connection, OpenFlags, OptionalExtension, Row};
use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;

#[derive(Serialize, Debug, Clone)]
pub struct Kanji {
    pub character: String,
    pub stroke_count: i32,
    pub grade: i32,
    pub jlpt_level: i32,
    pub frequency: i32,
    pub onyomi: String,
    pub kunyomi: String,
    /// Meanings separated by "; " (a single meaning can contain commas).
    pub meanings: String,
    pub nanori: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct KanjiLookup {
    pub character: String,
    pub stroke_count: i32,
    pub grade: i32,
    pub jlpt_level: i32,
    pub frequency: i32,
    pub onyomi: String,
    pub kunyomi: String,
    pub meanings: Vec<String>,
    pub nanori: Vec<String>,
}

const COLUMNS: &str =
    "character, stroke_count, grade, jlpt_level, frequency, onyomi, kunyomi, meanings, nanori";

/// Opens the database read-only. Installed apps usually can't write to their
/// resource directory, and read-only mode also avoids creating an empty
/// database file if the real one is missing.
pub fn open_db(path: &Path) -> Result<Connection, String> {
    Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("Failed to open kanji database at {}: {}", path.display(), e))
}

fn text(row: &Row, index: usize) -> String {
    row.get::<_, Option<String>>(index)
        .ok()
        .flatten()
        .unwrap_or_default()
}

fn int(row: &Row, index: usize) -> i32 {
    row.get::<_, Option<i32>>(index).ok().flatten().unwrap_or(0)
}

fn kanji_from_row(row: &Row) -> rusqlite::Result<Kanji> {
    Ok(Kanji {
        character: row.get(0)?,
        stroke_count: int(row, 1),
        grade: int(row, 2),
        jlpt_level: int(row, 3),
        frequency: int(row, 4),
        onyomi: text(row, 5),
        kunyomi: text(row, 6),
        meanings: text(row, 7),
        nanori: text(row, 8),
    })
}

fn split_list(value: &str, separator: char) -> Vec<String> {
    value
        .split(separator)
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect()
}

impl From<Kanji> for KanjiLookup {
    fn from(kanji: Kanji) -> Self {
        KanjiLookup {
            meanings: split_list(&kanji.meanings, ';'),
            nanori: split_list(&kanji.nanori, ','),
            character: kanji.character,
            stroke_count: kanji.stroke_count,
            grade: kanji.grade,
            jlpt_level: kanji.jlpt_level,
            frequency: kanji.frequency,
            onyomi: kanji.onyomi,
            kunyomi: kanji.kunyomi,
        }
    }
}

pub fn all_kanji(conn: &Connection) -> Result<Vec<Kanji>, String> {
    let mut stmt = conn
        .prepare(&format!("SELECT {} FROM kanji", COLUMNS))
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], kanji_from_row)
        .map_err(|e| e.to_string())?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())
}

pub fn lookup(conn: &Connection, character: &str) -> Result<KanjiLookup, String> {
    conn.query_row(
        &format!("SELECT {} FROM kanji WHERE character = ?1", COLUMNS),
        [character],
        kanji_from_row,
    )
    .optional()
    .map_err(|e| e.to_string())?
    .map(KanjiLookup::from)
    .ok_or_else(|| format!("Kanji {} is not in the dictionary", character))
}

/// Picks up to `count` kanji to add as new flashcards, skipping the ones in
/// `exclude`. School-grade kanji come first, then the easier old-JLPT levels,
/// then by frequency. Kanji without any English meaning are never picked.
pub fn pick_new_card_characters(
    conn: &Connection,
    exclude: &HashSet<String>,
    count: usize,
) -> Result<Vec<String>, String> {
    if count == 0 {
        return Ok(Vec::new());
    }

    // No LIMIT: rows already in the deck are skipped below, so a fixed limit
    // would eventually return only existing cards and add nothing. Rows are
    // read lazily and the loop stops once enough cards have been picked.
    let mut stmt = conn
        .prepare(
            "SELECT character FROM kanji
             WHERE (frequency > 0 OR grade BETWEEN 1 AND 8)
               AND meanings IS NOT NULL AND meanings <> ''
             ORDER BY
                 CASE
                     WHEN grade BETWEEN 1 AND 6 THEN 1
                     WHEN jlpt_level IN (3, 4) THEN 2
                     WHEN jlpt_level IN (1, 2) THEN 3
                     WHEN frequency BETWEEN 1 AND 1000 THEN 4
                     ELSE 5
                 END,
                 grade ASC,
                 CASE WHEN frequency > 0 THEN frequency ELSE 100000 END ASC,
                 stroke_count ASC,
                 character ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?;

    let mut picked = Vec::with_capacity(count);
    for row in rows {
        let character = row.map_err(|e| e.to_string())?;
        if !exclude.contains(&character) {
            picked.push(character);
            if picked.len() == count {
                break;
            }
        }
    }
    Ok(picked)
}

fn single_char(character: &str) -> Result<char, String> {
    let mut chars = character.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => Ok(c),
        _ => Err("Only one character is allowed".into()),
    }
}

/// Returns the stroke-order SVG for a kanji: the animated AnimCJK version if
/// available (decimal code point file names), otherwise the static KanjiVG
/// one (5-digit hex file names).
pub fn kanji_svg(resource_dir: &Path, character: &str) -> Result<String, String> {
    let code = single_char(character)? as u32;
    let animated = resource_dir
        .join("data/kanji_svg_animated/svgsJa")
        .join(format!("{}.svg", code));
    let static_svg = resource_dir
        .join("data/kanji_svg")
        .join(format!("{:05x}.svg", code));

    for path in [&animated, &static_svg] {
        if path.exists() {
            let content = std::fs::read_to_string(path)
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
            return Ok(clean_svg_content(&content));
        }
    }
    Err(format!("No stroke order diagram for {}", character))
}

/// Returns the animated AnimCJK stroke-order SVG for a single kana.
pub fn kana_svg(resource_dir: &Path, character: &str) -> Result<String, String> {
    let code = single_char(character)? as u32;
    let path = resource_dir
        .join("data/kana_svg_animated/svgsJaKana")
        .join(format!("{}.svg", code));
    let content = std::fs::read_to_string(&path)
        .map_err(|_| format!("No stroke order diagram for {}", character))?;
    Ok(clean_svg_content(&content))
}

/// Strips the XML declaration, DOCTYPE and leading license comment so the
/// SVG can be inlined into the page.
pub fn clean_svg_content(svg_content: &str) -> String {
    let mut cleaned = svg_content.to_string();

    if let Some(start) = cleaned.find("<?xml") {
        if let Some(end) = cleaned[start..].find("?>") {
            cleaned = cleaned[..start].to_string() + &cleaned[start + end + 2..];
        }
    }

    if let Some(start) = cleaned.find("<!DOCTYPE") {
        if let Some(end) = cleaned[start..].find("]>") {
            cleaned = cleaned[..start].to_string() + &cleaned[start + end + 2..];
        }
    }

    if cleaned.trim_start().starts_with("<!--") {
        if let Some(svg_start) = cleaned.find("<svg") {
            let before_svg = &cleaned[..svg_start];
            if before_svg.contains("<!--") && before_svg.contains("-->") {
                cleaned = cleaned[svg_start..].to_string();
            }
        }
    }

    cleaned.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn resource_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn db() -> Connection {
        open_db(&resource_dir().join("data/kanji.db")).unwrap()
    }

    #[test]
    fn looks_up_kanji_and_splits_meanings_on_semicolons() {
        let kanji = lookup(&db(), "移").unwrap();
        assert!(kanji.meanings.contains(&"catch (cold, fire)".to_string()));
        assert!(kanji.meanings.contains(&"shift".to_string()));
        assert!(lookup(&db(), "x").is_err());
    }

    #[test]
    fn loads_every_kanji() {
        let all = all_kanji(&db()).unwrap();
        assert!(all.len() > 13000);
        assert!(all.iter().any(|k| k.character == "日" && k.grade == 1));
    }

    #[test]
    fn new_cards_keep_coming_after_the_first_batches() {
        let conn = db();
        let mut deck: HashSet<String> = HashSet::new();
        // Regression test: with the old `LIMIT count * 3` this stopped adding
        // cards once the deck held 15 cards.
        for _ in 0..10 {
            let picked = pick_new_card_characters(&conn, &deck, 5).unwrap();
            assert_eq!(picked.len(), 5);
            for character in picked {
                assert!(deck.insert(character), "picked a duplicate");
            }
        }
        assert!(deck.contains("一"));
        assert!(pick_new_card_characters(&conn, &deck, 0).unwrap().is_empty());
    }

    #[test]
    fn finds_animated_and_static_svgs() {
        let dir = resource_dir();
        let animated = kanji_svg(&dir, "日").unwrap();
        assert!(animated.starts_with("<svg"));
        assert!(animated.contains("acjk"));

        let fallback = kanji_svg(&dir, "喘").unwrap();
        assert!(fallback.starts_with("<svg"));
        assert!(!fallback.contains("<!DOCTYPE"));

        assert!(kanji_svg(&dir, "日本").is_err());
        assert!(kana_svg(&dir, "あ").unwrap().starts_with("<svg"));
        assert!(kana_svg(&dir, "ゃ").is_ok());
    }
}
