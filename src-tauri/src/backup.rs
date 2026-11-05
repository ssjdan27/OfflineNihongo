//! Exporting and importing study progress as a single JSON file.

use crate::srs::{ReviewHistory, SrsCard};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub const BACKUP_FORMAT: &str = "offline-nihongo-backup";
pub const BACKUP_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Backup {
    pub format: String,
    pub version: u32,
    pub exported_at: String,
    pub cards: Vec<SrsCard>,
    #[serde(default)]
    pub review_history: ReviewHistory,
    #[serde(default)]
    pub best_times: HashMap<String, u64>,
}

impl Backup {
    pub fn new(
        exported_at: String,
        cards: Vec<SrsCard>,
        review_history: ReviewHistory,
        best_times: HashMap<String, u64>,
    ) -> Self {
        Backup {
            format: BACKUP_FORMAT.to_string(),
            version: BACKUP_VERSION,
            exported_at,
            cards,
            review_history,
            best_times,
        }
    }
}

/// What an import restored, for the confirmation message.
#[derive(Serialize, Debug, PartialEq)]
pub struct ImportSummary {
    pub cards: usize,
    pub review_days: usize,
    pub best_times: usize,
}

/// Parses and validates a backup file's contents.
pub fn parse_backup(json: &str) -> Result<Backup, String> {
    let backup: Backup = serde_json::from_str(json)
        .map_err(|e| format!("This doesn't look like an Offline Nihongo backup ({})", e))?;

    if backup.format != BACKUP_FORMAT {
        return Err("This doesn't look like an Offline Nihongo backup".into());
    }
    if backup.version > BACKUP_VERSION {
        return Err(format!(
            "This backup was made by a newer version of the app (format {}); please update first",
            backup.version
        ));
    }

    let mut seen = HashSet::new();
    for card in &backup.cards {
        if card.character.chars().count() != 1 {
            return Err(format!("The backup has an invalid card: {:?}", card.character));
        }
        if !seen.insert(card.character.as_str()) {
            return Err(format!("The backup has more than one card for {}", card.character));
        }
    }
    Ok(backup)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::srs::DayStats;
    use chrono::{FixedOffset, TimeZone};

    fn sample() -> Backup {
        let now = FixedOffset::east_opt(0)
            .unwrap()
            .with_ymd_and_hms(2026, 1, 1, 12, 0, 0)
            .unwrap();
        let mut history = ReviewHistory::new();
        history.insert("2026-01-01".into(), DayStats { reviews: 3, correct: 2 });
        let mut times = HashMap::new();
        times.insert("hiragana-seion-basic".into(), 61234);
        Backup::new(
            now.to_rfc3339(),
            vec![SrsCard::new("水", &now), SrsCard::new("火", &now)],
            history,
            times,
        )
    }

    #[test]
    fn round_trips() {
        let backup = sample();
        let json = serde_json::to_string_pretty(&backup).unwrap();
        assert_eq!(parse_backup(&json).unwrap(), backup);
    }

    #[test]
    fn rejects_other_files() {
        assert!(parse_backup("not json").is_err());
        assert!(parse_backup("[]").is_err());
        let mut other = sample();
        other.format = "something-else".into();
        assert!(parse_backup(&serde_json::to_string(&other).unwrap()).is_err());
    }

    #[test]
    fn rejects_newer_versions_and_bad_cards() {
        let mut newer = sample();
        newer.version = BACKUP_VERSION + 1;
        let error = parse_backup(&serde_json::to_string(&newer).unwrap()).unwrap_err();
        assert!(error.contains("newer version"));

        let mut duplicate = sample();
        duplicate.cards[1].character = "水".into();
        assert!(parse_backup(&serde_json::to_string(&duplicate).unwrap()).is_err());

        let mut word = sample();
        word.cards[0].character = "水曜".into();
        assert!(parse_backup(&serde_json::to_string(&word).unwrap()).is_err());
    }

    #[test]
    fn history_and_best_times_are_optional() {
        let json = r#"{"format":"offline-nihongo-backup","version":1,"exported_at":"x","cards":[]}"#;
        let backup = parse_backup(json).unwrap();
        assert!(backup.review_history.is_empty());
        assert!(backup.best_times.is_empty());
    }
}
