//! Best times for the kana speed game.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct GameTimes {
    #[serde(default)]
    pub best_times: HashMap<String, u64>,
}

impl GameTimes {
    /// Stores `time_ms` if it beats the current best for `game_key`.
    /// Returns whether it was a new best.
    pub fn record(&mut self, game_key: &str, time_ms: u64) -> bool {
        let is_best = self
            .best_times
            .get(game_key)
            .map_or(true, |best| time_ms < *best);
        if is_best {
            self.best_times.insert(game_key.to_string(), time_ms);
        }
        is_best
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_fastest_time() {
        let mut times = GameTimes::default();
        assert!(times.record("hiragana-seion-basic", 5000));
        assert!(!times.record("hiragana-seion-basic", 6000));
        assert!(times.record("hiragana-seion-basic", 4000));
        assert!(times.record("katakana-seion-basic", 9000));
        assert_eq!(times.best_times["hiragana-seion-basic"], 4000);
    }
}
