//! Kana list used by the kana speed game, loaded from `data/kana.json`.

use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct KanaChar {
    pub character: String,
    pub romaji: String,
    /// "hiragana" or "katakana"
    pub kana_type: String,
    /// "seion", "dakuon" or "handakuon"
    pub sound_type: String,
    /// "basic" (a single kana) or "combination" (a yōon such as きゃ)
    pub complexity: String,
}

/// Parses the nested kana.json structure:
/// consonant -> vowel -> sound type -> { Hiragana, Katakana, Romaji }.
pub fn parse_kana_json(json: &str) -> Result<Vec<KanaChar>, String> {
    let data: Value =
        serde_json::from_str(json).map_err(|e| format!("Failed to parse kana.json: {}", e))?;

    let mut kana_list = Vec::new();
    let consonants = data.as_object().into_iter().flat_map(|obj| obj.values());
    for vowels in consonants {
        for types in vowels.as_object().into_iter().flat_map(|obj| obj.values()) {
            for (type_name, info) in types.as_object().into_iter().flatten() {
                let field = |name: &str| info.get(name).and_then(Value::as_str);
                let (Some(hiragana), Some(katakana), Some(romaji)) =
                    (field("Hiragana"), field("Katakana"), field("Romaji"))
                else {
                    continue;
                };

                let sound_type = match type_name.as_str() {
                    "Dakuon" => "dakuon",
                    "Handakuon" => "handakuon",
                    _ => "seion",
                };

                // Decide by the kana itself rather than the romaji length:
                // し (shi) is a single kana, じゃ (ja) is a combination.
                let complexity = if hiragana.chars().count() > 1 {
                    "combination"
                } else {
                    "basic"
                };

                for (character, kana_type) in [(hiragana, "hiragana"), (katakana, "katakana")] {
                    kana_list.push(KanaChar {
                        character: character.to_string(),
                        romaji: romaji.to_string(),
                        kana_type: kana_type.to_string(),
                        sound_type: sound_type.to_string(),
                        complexity: complexity.to_string(),
                    });
                }
            }
        }
    }

    Ok(kana_list)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundled_kana() -> Vec<KanaChar> {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/kana.json");
        parse_kana_json(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    fn find<'a>(list: &'a [KanaChar], character: &str) -> &'a KanaChar {
        list.iter()
            .find(|k| k.character == character)
            .unwrap_or_else(|| panic!("{} missing", character))
    }

    #[test]
    fn classifies_complexity_by_kana_not_romaji_length() {
        let list = bundled_kana();
        for basic in ["し", "ち", "つ", "シ", "ツ", "ん"] {
            assert_eq!(find(&list, basic).complexity, "basic", "{}", basic);
        }
        for combination in ["じゃ", "じゅ", "じょ", "ぢゃ", "きゃ", "ジョ"] {
            assert_eq!(find(&list, combination).complexity, "combination", "{}", combination);
        }
    }

    #[test]
    fn katakana_matches_hiragana() {
        let list = bundled_kana();
        assert_eq!(list.len(), 214);
        let hiragana: Vec<_> = list.iter().filter(|k| k.kana_type == "hiragana").collect();
        for h in hiragana {
            let expected: String = h
                .character
                .chars()
                .map(|c| char::from_u32(c as u32 + 0x60).unwrap())
                .collect();
            assert!(
                list.iter().any(|k| k.kana_type == "katakana"
                    && k.character == expected
                    && k.romaji == h.romaji),
                "no katakana {} for {}",
                expected,
                h.character
            );
        }
    }

    #[test]
    fn sound_types() {
        let list = bundled_kana();
        assert_eq!(find(&list, "か").sound_type, "seion");
        assert_eq!(find(&list, "が").sound_type, "dakuon");
        assert_eq!(find(&list, "ぱ").sound_type, "handakuon");
        assert_eq!(find(&list, "ぴょ").sound_type, "handakuon");
    }
}
