# OfflineNihongo

A Japanese language learning tool built with Tauri, SvelteKit and TypeScript.

## About

OfflineNihongo is an offline application for studying Japanese kana and kanji. Everything runs locally: the dictionary and stroke order data are bundled with the app, and your progress is stored on your computer.

## Features

- **Kana chart** – hiragana and katakana (gojūon, dakuon/handakuon and yōon) with animated stroke order and pronunciation.
- **Kana speed game** – type the romaji for each kana against the clock. Common alternative spellings (si/shi, tu/tsu, …) are accepted, mistakes and skips are tracked, best times are kept for every combination of settings, and you can replay just the characters you missed.
- **Kanji database** – 13,108 kanji from KANJIDIC2 with readings, meanings, nanori, stroke count, school grade, old JLPT level and frequency rank.
  - Search by English meaning, by reading in kana or romaji (`mizu`, `taberu`), or paste a word (`日本語`) to see its kanji; results are ranked with common kanji first.
  - Browse by frequency, grade, JLPT level or stroke count.
  - Stroke order diagrams with play, stop, reset and show-all controls and adjustable speed.
  - A writing practice pad (mouse, pen or touch) with an optional guide and a stroke counter, also available for kana.
- **Kanji flashcards (spaced repetition)** – meaning and reading reviews (plus optional "which kanji is this?" recall reviews) scheduled with an SM-2 style algorithm.
  - Add kanji automatically by grade and frequency, individually from the Kanji Database, or a whole list at once (e.g. all JLPT 4 kanji).
  - Keyboard shortcuts (Space to reveal, 1–4 to grade), optional typed answers for readings, and missed cards come back later in the same session.
  - Dashboard with due/new/learning/mature counts, a seven-day review forecast, weekly history and daily streak.
- **Pronunciation** – 🔊 buttons use your system's Japanese text-to-speech voice when one is installed.
- **Settings** – dark mode, study session size, and export/import of a backup file.

## Development

Install the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform, then:

```bash
npm install
npm run tauri dev
```

Checks and tests (also run by the `test` GitHub Actions workflow):

```bash
npm run check                   # Svelte/TypeScript type checking
npm test                        # frontend unit tests (Vitest)
cd src-tauri && cargo test      # backend unit tests
```

Study progress is stored as JSON files (`srs_cards.json`, `review_history.json`, `game_times.json`) in the app's local data folder, for example `~/.local/share/OfflineNihongo` on Linux, `%LOCALAPPDATA%\OfflineNihongo` on Windows and `~/Library/Application Support/OfflineNihongo` on macOS. Copies made before an import or reset are kept in its `backups` folder.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

## Dictionary Data Attribution

### KANJIDIC2 Project

This application uses data from the KANJIDIC2 file, which is part of the KANJIDIC Project maintained by the Electronic Dictionary Research and Development Group (EDRDG) and was compiled by Jim Breen.

The KANJIDIC2 dictionary files are made available under the [Creative Commons Attribution-ShareAlike License (V4.0)](https://creativecommons.org/licenses/by-sa/4.0/).

For more information about the KANJIDIC Project, please visit:
- [KANJIDIC Project Information](https://www.edrdg.org/wiki/index.php/KANJIDIC_Project)
- [EDRDG License Information](https://www.edrdg.org/edrdg/licence.html)

In accordance with the license terms, we acknowledge that:
1. The KANJIDIC2 file was used to create the kanji database in this application
2. The original data was created by Jim Breen and the Electronic Dictionary Research and Development Group
3. The inclusion of this data in our application does not indicate any endorsement by the EDRDG
4. Our modifications and use of this data does not remove or diminish the EDRDG's copyright over the files

### KanjiVG Project

The stroke order diagrams used in this application are from the KanjiVG project created by Ulrich Apel.

The KanjiVG data is made available under the [Creative Commons Attribution-ShareAlike License (V3.0)](https://creativecommons.org/licenses/by-sa/3.0/).

For more information about the KanjiVG Project, please visit:
- [KanjiVG Project Website](https://kanjivg.tagaini.net/)
- [KanjiVG GitHub Repository](https://github.com/KanjiVG/kanjivg)

In accordance with the license terms, we acknowledge that:
1. The SVG files used for kanji stroke order diagrams are from the KanjiVG project
2. The original data was created by Ulrich Apel and other contributors to the KanjiVG project
3. The inclusion of this data in our application does not indicate any endorsement by the KanjiVG project
4. Our modifications and use of this data does not remove or diminish the original copyright over the files

### animCJK Project

This application uses SVG stroke order animations for kanji and kana from the [animCJK GitHub repository](https://github.com/parsimonhi/animCJK) by François Mizessyn, distributed under the [GNU Lesser General Public License v3.0](https://www.gnu.org/licenses/lgpl-3.0.html).

## License

This application itself is licensed under [MIT License](LICENSE), but the dictionary data and stroke order diagrams used within it are subject to the license terms of their respective projects as described above.