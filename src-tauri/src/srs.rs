//! Spaced-repetition scheduling and study statistics for the kanji flashcards.
//!
//! Everything here is pure: callers pass in the current time and time zone,
//! which keeps the scheduling rules easy to test.

use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;

/// Cards at or above this level count as "mature".
pub const MATURE_LEVEL: i32 = 4;
/// Upper bound for review intervals (about ten years).
pub const MAX_INTERVAL_DAYS: i32 = 3650;

fn default_interval() -> i32 {
    1
}

fn default_ease() -> f32 {
    2.5
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SrsCard {
    pub id: String,
    pub character: String,
    #[serde(default)]
    pub level: i32,
    #[serde(default = "default_interval")]
    pub interval: i32,
    #[serde(default = "default_ease")]
    pub ease_factor: f32,
    /// RFC 3339 timestamp of the next review.
    #[serde(default)]
    pub next_review: String,
    #[serde(default)]
    pub total_reviews: i32,
    #[serde(default)]
    pub correct_reviews: i32,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub last_reviewed: Option<String>,
    #[serde(default)]
    pub streak: i32,
}

/// Parses the timestamps stored in card files. Cards are written as RFC 3339;
/// the naive fallback (read as UTC) matches what earlier versions accepted.
fn parse_timestamp(value: &str) -> Option<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Some(dt.with_timezone(&Utc));
    }
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f")
        .ok()
        .map(|naive| Utc.from_utc_datetime(&naive))
}

impl SrsCard {
    pub fn new<Tz: TimeZone>(character: &str, now: &DateTime<Tz>) -> Self
    where
        Tz::Offset: Display,
    {
        let stamp = now.to_rfc3339();
        SrsCard {
            id: format!("{}_{}", character, now.timestamp()),
            character: character.to_string(),
            level: 0,
            interval: 1,
            ease_factor: default_ease(),
            next_review: stamp.clone(),
            total_reviews: 0,
            correct_reviews: 0,
            created_at: stamp,
            last_reviewed: None,
            streak: 0,
        }
    }

    /// A card that has never been reviewed.
    pub fn is_new(&self) -> bool {
        self.total_reviews == 0
    }

    /// The calendar day (in `tz`) on which the card is next due. A card whose
    /// date can't be read is treated as overdue so it isn't lost.
    pub fn due_date<Tz: TimeZone>(&self, tz: &Tz) -> NaiveDate {
        parse_timestamp(&self.next_review)
            .map(|dt| dt.with_timezone(tz).date_naive())
            .unwrap_or(NaiveDate::MIN)
    }

    /// A previously reviewed card whose next review falls on or before `today`.
    /// New cards are never "due"; they are counted separately.
    pub fn is_due<Tz: TimeZone>(&self, today: NaiveDate, tz: &Tz) -> bool {
        !self.is_new() && self.due_date(tz) <= today
    }
}

/// Adds a new card for `character` unless the deck already has one.
pub fn add_card<Tz: TimeZone>(
    cards: &mut Vec<SrsCard>,
    character: &str,
    now: &DateTime<Tz>,
) -> Result<SrsCard, String>
where
    Tz::Offset: Display,
{
    if cards.iter().any(|card| card.character == character) {
        return Err(format!("{} is already in your flashcards", character));
    }
    let card = SrsCard::new(character, now);
    cards.push(card.clone());
    Ok(card)
}

/// Adds new cards for the characters that aren't in the deck yet and
/// returns how many were added.
pub fn add_cards<Tz: TimeZone>(
    cards: &mut Vec<SrsCard>,
    characters: &[String],
    now: &DateTime<Tz>,
) -> usize
where
    Tz::Offset: Display,
{
    let mut existing: std::collections::HashSet<String> =
        cards.iter().map(|card| card.character.clone()).collect();
    let before = cards.len();
    for character in characters {
        if existing.insert(character.clone()) {
            cards.push(SrsCard::new(character, now));
        }
    }
    cards.len() - before
}

/// Removes the card for `character`; returns whether there was one.
pub fn remove_card(cards: &mut Vec<SrsCard>, character: &str) -> bool {
    let before = cards.len();
    cards.retain(|card| card.character != character);
    cards.len() != before
}

fn scale(interval: i32, factor: f32) -> i32 {
    // `as` saturates on overflow; the result is clamped by the caller.
    (interval as f32 * factor) as i32
}

/// Applies one review to `card` using an SM-2 style schedule.
///
/// Grades: 1 = Again, 2 = Hard, 3 = Good, 4 = Easy. Only 3 and 4 count as
/// correct answers.
pub fn apply_review<Tz: TimeZone>(
    card: &mut SrsCard,
    grade: i32,
    now: &DateTime<Tz>,
) -> Result<(), String>
where
    Tz::Offset: Display,
{
    if !(1..=4).contains(&grade) {
        return Err(format!("Invalid grade: {}", grade));
    }

    card.total_reviews += 1;
    card.last_reviewed = Some(now.to_rfc3339());

    let previous_interval = card.interval.max(1);
    let (interval, ease, level) = match grade {
        4 => {
            let ease = (card.ease_factor + 0.15).min(3.0);
            match card.level {
                0 => (4, ease, 2),
                1 => (10, ease, 3),
                _ => (scale(previous_interval, ease * 1.3), ease, card.level + 1),
            }
        }
        3 => {
            let ease = card.ease_factor;
            match card.level {
                0 => (1, ease, 1),
                1 => (6, ease, 2),
                _ => (scale(previous_interval, ease), ease, card.level + 1),
            }
        }
        2 => (
            scale(previous_interval, 0.6),
            (card.ease_factor - 0.15).max(1.3),
            (card.level - 1).max(0),
        ),
        _ => (1, (card.ease_factor - 0.2).max(1.3), 0),
    };

    if grade >= 3 {
        card.correct_reviews += 1;
        card.streak += 1;
    } else {
        card.streak = 0;
    }

    card.interval = interval.clamp(1, MAX_INTERVAL_DAYS);
    card.ease_factor = ease;
    card.level = level;
    card.next_review = (now.clone() + Duration::days(card.interval as i64)).to_rfc3339();
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SessionStats {
    pub total_reviews: i32,
    pub correct_answers: i32,
    pub session_time: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReviewSession {
    pub cards_due: Vec<SrsCard>,
    pub cards_new: Vec<SrsCard>,
    pub session_stats: SessionStats,
}

/// Picks the cards for a study session. `session_type` is "due", "new" or
/// "mixed". Due cards come most-overdue first; new cards keep the order in
/// which they were added.
pub fn build_session<Tz: TimeZone>(
    cards: Vec<SrsCard>,
    session_type: &str,
    today: NaiveDate,
    tz: &Tz,
    due_limit: usize,
    new_limit: usize,
) -> Result<ReviewSession, String> {
    let (include_due, include_new) = match session_type {
        "due" => (true, false),
        "new" => (false, true),
        "mixed" => (true, true),
        other => return Err(format!("Unknown session type: {}", other)),
    };

    let mut cards_due = Vec::new();
    let mut cards_new = Vec::new();
    for card in cards {
        if card.is_new() {
            if include_new {
                cards_new.push(card);
            }
        } else if include_due && card.is_due(today, tz) {
            cards_due.push(card);
        }
    }

    // `None` (unreadable date) sorts first, i.e. treated as most overdue.
    cards_due.sort_by_key(|card| parse_timestamp(&card.next_review));
    cards_due.truncate(due_limit);
    cards_new.truncate(new_limit);

    Ok(ReviewSession {
        cards_due,
        cards_new,
        session_stats: SessionStats::default(),
    })
}

/// Reviews done on one calendar day.
#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq)]
pub struct DayStats {
    pub reviews: i32,
    pub correct: i32,
}

/// Review counts keyed by local calendar day ("YYYY-MM-DD").
pub type ReviewHistory = BTreeMap<String, DayStats>;

pub fn day_key(day: NaiveDate) -> String {
    day.format("%Y-%m-%d").to_string()
}

pub fn record_review(history: &mut ReviewHistory, day: NaiveDate, correct: bool) {
    let entry = history.entry(day_key(day)).or_default();
    entry.reviews += 1;
    if correct {
        entry.correct += 1;
    }
}

/// Converts the `weekly_stats.json` format used by earlier versions
/// (date -> (reviews, accuracy)) into a review history.
pub fn history_from_legacy_weekly_stats(legacy: &HashMap<String, (i32, f32)>) -> ReviewHistory {
    legacy
        .iter()
        .map(|(date, (reviews, accuracy))| {
            let correct = ((accuracy * *reviews as f32).round() as i32).clamp(0, (*reviews).max(0));
            (
                date.clone(),
                DayStats {
                    reviews: *reviews,
                    correct,
                },
            )
        })
        .collect()
}

fn reviewed_on(history: &ReviewHistory, day: NaiveDate) -> bool {
    history
        .get(&day_key(day))
        .is_some_and(|stats| stats.reviews > 0)
}

/// Consecutive days with at least one review, ending today. If nothing has
/// been reviewed yet today the streak is still alive and counts up to
/// yesterday.
pub fn daily_streak(history: &ReviewHistory, today: NaiveDate) -> i32 {
    let mut day = if reviewed_on(history, today) {
        Some(today)
    } else {
        today.pred_opt()
    };

    let mut streak = 0;
    while let Some(current) = day {
        if !reviewed_on(history, current) {
            break;
        }
        streak += 1;
        day = current.pred_opt();
    }
    streak
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct StudyStats {
    pub total_cards: i32,
    pub cards_due: i32,
    pub cards_new: i32,
    pub cards_learning: i32,
    pub cards_mature: i32,
    pub daily_streak: i32,
    pub total_reviews: i32,
    pub accuracy: f32,
    /// The last seven days as "YYYY-MM-DD", oldest first (the last one is today).
    pub week_dates: Vec<String>,
    pub reviews_this_week: Vec<i32>,
    pub accuracy_this_week: Vec<f32>,
    /// Reviews coming due on each of the next seven days (index 0 = tomorrow).
    pub forecast: Vec<i32>,
}

const FORECAST_DAYS: i64 = 7;

pub fn compute_stats<Tz: TimeZone>(
    cards: &[SrsCard],
    history: &ReviewHistory,
    today: NaiveDate,
    tz: &Tz,
) -> StudyStats {
    let mut cards_due = 0;
    let mut cards_new = 0;
    let mut cards_learning = 0;
    let mut cards_mature = 0;
    let mut total_reviews = 0;
    let mut correct_reviews = 0;
    let mut forecast = vec![0; FORECAST_DAYS as usize];

    for card in cards {
        total_reviews += card.total_reviews;
        correct_reviews += card.correct_reviews;

        if card.is_new() {
            cards_new += 1;
            continue;
        }
        let days_until_due = (card.due_date(tz) - today).num_days();
        if days_until_due <= 0 {
            cards_due += 1;
        } else if days_until_due <= FORECAST_DAYS {
            forecast[(days_until_due - 1) as usize] += 1;
        }
        if card.level >= MATURE_LEVEL {
            cards_mature += 1;
        } else {
            cards_learning += 1;
        }
    }

    let accuracy = if total_reviews > 0 {
        correct_reviews as f32 / total_reviews as f32
    } else {
        0.0
    };

    let mut week_dates = Vec::with_capacity(7);
    let mut reviews_this_week = Vec::with_capacity(7);
    let mut accuracy_this_week = Vec::with_capacity(7);
    for days_ago in (0..7).rev() {
        let day = today - Duration::days(days_ago);
        let stats = history.get(&day_key(day)).copied().unwrap_or_default();
        week_dates.push(day_key(day));
        reviews_this_week.push(stats.reviews);
        accuracy_this_week.push(if stats.reviews > 0 {
            stats.correct as f32 / stats.reviews as f32
        } else {
            0.0
        });
    }

    StudyStats {
        total_cards: cards.len() as i32,
        cards_due,
        cards_new,
        cards_learning,
        cards_mature,
        daily_streak: daily_streak(history, today),
        total_reviews,
        accuracy,
        week_dates,
        reviews_this_week,
        accuracy_this_week,
        forecast,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::FixedOffset;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn at(tz: &FixedOffset, y: i32, m: u32, d: u32, h: u32) -> DateTime<FixedOffset> {
        tz.with_ymd_and_hms(y, m, d, h, 0, 0).unwrap()
    }

    fn reviewed_card(tz: &FixedOffset, grades: &[i32]) -> SrsCard {
        let now = at(tz, 2026, 1, 1, 12);
        let mut card = SrsCard::new("日", &now);
        for grade in grades {
            apply_review(&mut card, *grade, &now).unwrap();
        }
        card
    }

    #[test]
    fn good_reviews_follow_the_schedule() {
        let tz = FixedOffset::east_opt(0).unwrap();
        let now = at(&tz, 2026, 1, 1, 12);
        let mut card = SrsCard::new("日", &now);

        apply_review(&mut card, 3, &now).unwrap();
        assert_eq!((card.level, card.interval), (1, 1));
        apply_review(&mut card, 3, &now).unwrap();
        assert_eq!((card.level, card.interval), (2, 6));
        apply_review(&mut card, 3, &now).unwrap();
        assert_eq!((card.level, card.interval), (3, 15));
        assert_eq!(card.total_reviews, 3);
        assert_eq!(card.correct_reviews, 3);
        assert_eq!(card.streak, 3);
        assert_eq!(card.due_date(&tz), date(2026, 1, 16));
    }

    #[test]
    fn again_resets_and_hard_backs_off() {
        let tz = FixedOffset::east_opt(0).unwrap();
        let mut card = reviewed_card(&tz, &[3, 3, 3]);
        let now = at(&tz, 2026, 1, 1, 12);

        apply_review(&mut card, 2, &now).unwrap();
        assert_eq!(card.level, 2);
        assert_eq!(card.interval, 9); // 15 * 0.6
        assert_eq!(card.streak, 0);
        assert!((card.ease_factor - 2.35).abs() < 1e-5);

        apply_review(&mut card, 1, &now).unwrap();
        assert_eq!((card.level, card.interval), (0, 1));
        assert!((card.ease_factor - 2.15).abs() < 1e-5);
        assert_eq!(card.correct_reviews, 3);
        assert_eq!(card.total_reviews, 5);
    }

    #[test]
    fn ease_is_bounded_and_interval_is_capped() {
        let tz = FixedOffset::east_opt(0).unwrap();
        let card = reviewed_card(&tz, &[4; 40]);
        assert!(card.ease_factor <= 3.0);
        assert_eq!(card.interval, MAX_INTERVAL_DAYS);

        let mut failing = reviewed_card(&tz, &[1; 20]);
        assert!(failing.ease_factor >= 1.3);
        assert!(apply_review(&mut failing, 0, &at(&tz, 2026, 1, 1, 12)).is_err());
        assert!(apply_review(&mut failing, 5, &at(&tz, 2026, 1, 1, 12)).is_err());
    }

    #[test]
    fn due_dates_use_the_local_calendar_day() {
        // 23:00 on Jan 1 in UTC-6 is already Jan 2 in UTC.
        let tz = FixedOffset::west_opt(6 * 3600).unwrap();
        let evening = at(&tz, 2026, 1, 1, 23);
        let mut card = SrsCard::new("日", &evening);
        apply_review(&mut card, 3, &evening).unwrap(); // interval 1 day

        assert_eq!(card.due_date(&tz), date(2026, 1, 2));
        assert!(!card.is_due(date(2026, 1, 1), &tz));
        assert!(card.is_due(date(2026, 1, 2), &tz));
    }

    #[test]
    fn new_cards_are_not_due_and_unreadable_dates_are_overdue() {
        let tz = FixedOffset::east_opt(0).unwrap();
        let card = SrsCard::new("日", &at(&tz, 2026, 1, 1, 12));
        assert!(card.is_new());
        assert!(!card.is_due(date(2030, 1, 1), &tz));

        let mut broken = reviewed_card(&tz, &[3]);
        broken.next_review = "not a date".into();
        assert!(broken.is_due(date(2000, 1, 1), &tz));
    }

    #[test]
    fn sessions_split_new_and_due_cards() {
        let tz = FixedOffset::east_opt(0).unwrap();
        let now = at(&tz, 2026, 1, 1, 12);
        let new_card = SrsCard::new("一", &now);
        let mut due_later = reviewed_card(&tz, &[3, 3]); // due Jan 7
        due_later.id = "later".into();
        let mut due_soon = reviewed_card(&tz, &[3]); // due Jan 2
        due_soon.id = "soon".into();
        let not_due = reviewed_card(&tz, &[3, 3, 3]); // due Jan 16

        let cards = vec![new_card.clone(), due_later, due_soon, not_due];
        let today = date(2026, 1, 10);

        let due = build_session(cards.clone(), "due", today, &tz, 20, 20).unwrap();
        let ids: Vec<_> = due.cards_due.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, vec!["soon", "later"]);
        assert!(due.cards_new.is_empty());

        let new = build_session(cards.clone(), "new", today, &tz, 20, 20).unwrap();
        assert!(new.cards_due.is_empty());
        assert_eq!(new.cards_new, vec![new_card]);

        let mixed = build_session(cards.clone(), "mixed", today, &tz, 1, 20).unwrap();
        assert_eq!(mixed.cards_due.len(), 1);
        assert_eq!(mixed.cards_new.len(), 1);

        assert!(build_session(cards, "bogus", today, &tz, 20, 20).is_err());
    }

    #[test]
    fn stats_count_each_card_once() {
        let tz = FixedOffset::east_opt(0).unwrap();
        let now = at(&tz, 2026, 1, 1, 12);
        let cards = vec![
            SrsCard::new("一", &now),           // new
            reviewed_card(&tz, &[1]),          // failed: learning, due Jan 2
            reviewed_card(&tz, &[3, 3]),       // learning, due Jan 7
            reviewed_card(&tz, &[4, 4, 4]),    // mature
        ];
        let stats = compute_stats(&cards, &ReviewHistory::new(), date(2026, 1, 3), &tz);
        assert_eq!(stats.total_cards, 4);
        assert_eq!(stats.cards_new, 1);
        assert_eq!(stats.cards_due, 1);
        assert_eq!(stats.cards_learning, 2);
        assert_eq!(stats.cards_mature, 1);
        assert_eq!(stats.total_reviews, 6);
        assert!((stats.accuracy - 5.0 / 6.0).abs() < 1e-5);
    }

    #[test]
    fn forecast_counts_reviews_for_the_next_week() {
        let tz = FixedOffset::east_opt(0).unwrap();
        let cards = vec![
            reviewed_card(&tz, &[3]),          // due Jan 2 (tomorrow)
            reviewed_card(&tz, &[1]),          // due Jan 2 (tomorrow)
            reviewed_card(&tz, &[3, 3]),       // due Jan 7 (in 6 days)
            reviewed_card(&tz, &[3, 3, 3]),    // due Jan 16: beyond the forecast
        ];
        let stats = compute_stats(&cards, &ReviewHistory::new(), date(2026, 1, 1), &tz);
        assert_eq!(stats.cards_due, 0);
        assert_eq!(stats.forecast, vec![2, 0, 0, 0, 0, 1, 0]);

        // Once a day has passed, overdue cards move into "due".
        let later = compute_stats(&cards, &ReviewHistory::new(), date(2026, 1, 3), &tz);
        assert_eq!(later.cards_due, 2);
        assert_eq!(later.forecast, vec![0, 0, 0, 1, 0, 0, 0]);
    }

    #[test]
    fn weekly_stats_end_today() {
        let tz = FixedOffset::east_opt(0).unwrap();
        let today = date(2026, 11, 5);
        let mut history = ReviewHistory::new();
        record_review(&mut history, today, true);
        record_review(&mut history, today, false);
        record_review(&mut history, date(2026, 10, 30), true);
        record_review(&mut history, date(2026, 10, 29), true); // 7 days ago: outside the window

        let stats = compute_stats(&[], &history, today, &tz);
        assert_eq!(stats.week_dates.first().unwrap(), "2026-10-30");
        assert_eq!(stats.week_dates.last().unwrap(), "2026-11-05");
        assert_eq!(stats.reviews_this_week, vec![1, 0, 0, 0, 0, 0, 2]);
        assert_eq!(stats.accuracy_this_week[0], 1.0);
        assert_eq!(stats.accuracy_this_week[6], 0.5);
    }

    #[test]
    fn streak_counts_consecutive_days() {
        let today = date(2026, 11, 5);
        let mut history = ReviewHistory::new();
        assert_eq!(daily_streak(&history, today), 0);

        for days_ago in 1..=400 {
            record_review(&mut history, today - Duration::days(days_ago), true);
        }
        // Not reviewed today yet: the streak runs through yesterday, and is
        // no longer capped at 365 days.
        assert_eq!(daily_streak(&history, today), 400);

        record_review(&mut history, today, true);
        assert_eq!(daily_streak(&history, today), 401);

        history.remove(&day_key(today - Duration::days(1)));
        assert_eq!(daily_streak(&history, today), 1);
    }

    #[test]
    fn legacy_weekly_stats_migrate() {
        let mut legacy = HashMap::new();
        legacy.insert("2026-11-01".to_string(), (3, 2.0 / 3.0));
        legacy.insert("2026-11-02".to_string(), (1, 0.0));
        let history = history_from_legacy_weekly_stats(&legacy);
        assert_eq!(history["2026-11-01"], DayStats { reviews: 3, correct: 2 });
        assert_eq!(history["2026-11-02"], DayStats { reviews: 1, correct: 0 });
    }

    #[test]
    fn cards_can_be_added_once_and_removed() {
        let tz = FixedOffset::east_opt(0).unwrap();
        let now = at(&tz, 2026, 1, 1, 12);
        let mut cards = Vec::new();

        let card = add_card(&mut cards, "水", &now).unwrap();
        assert!(card.is_new());
        assert!(add_card(&mut cards, "水", &now).is_err());
        assert_eq!(cards.len(), 1);

        assert!(remove_card(&mut cards, "水"));
        assert!(!remove_card(&mut cards, "水"));
        assert!(cards.is_empty());
    }

    #[test]
    fn bulk_add_skips_cards_already_in_the_deck() {
        let tz = FixedOffset::east_opt(0).unwrap();
        let now = at(&tz, 2026, 1, 1, 12);
        let mut cards = vec![SrsCard::new("水", &now)];
        let input: Vec<String> = ["火", "水", "木", "火"].iter().map(|s| s.to_string()).collect();
        assert_eq!(add_cards(&mut cards, &input, &now), 2);
        let characters: Vec<_> = cards.iter().map(|c| c.character.as_str()).collect();
        assert_eq!(characters, vec!["水", "火", "木"]);
    }

    #[test]
    fn existing_card_files_still_parse() {
        let json = r#"[{"id":"日_1","character":"日","level":2,"interval":6,"ease_factor":2.5,
            "next_review":"2025-07-01T00:00:00+00:00","total_reviews":2,"correct_reviews":2,
            "created_at":"2025-06-20T00:00:00+00:00","last_reviewed":null,"streak":2}]"#;
        let cards: Vec<SrsCard> = serde_json::from_str(json).unwrap();
        assert_eq!(cards[0].level, 2);

        // Missing optional fields fall back to defaults instead of failing.
        let minimal: Vec<SrsCard> =
            serde_json::from_str(r#"[{"id":"x","character":"日"}]"#).unwrap();
        assert_eq!(minimal[0].interval, 1);
        assert_eq!(minimal[0].ease_factor, 2.5);
    }
}
