// Kanji search: matches characters, English meanings and readings (typed as
// kana or romaji), and ranks the best matches first.

import { type Kanji, splitMeanings } from './kanji';
import { romajiToHiragana, toHiragana } from './romaji';

interface IndexedKanji {
  kanji: Kanji;
  /** On and kun readings in hiragana, without okurigana dots or affix dashes */
  readings: string[];
  meanings: string[];
  meaningWords: Set<string>;
}

const KANA_ONLY = /^[ぁ-ゖァ-ヺー]+$/;
const HAS_KANJI = /[㐀-鿿豈-﫿\u{20000}-\u{2ffff}]/u;

/** "ひと.つ" -> "ひとつ", "-び" -> "び", "ニチ" -> "にち" */
export function normalizeReading(reading: string): string {
  return toHiragana(reading.replace(/[.\-－]/g, '').trim());
}

function splitReadings(field: string): string[] {
  return field
    .split(/[,、]/)
    .map(normalizeReading)
    .filter((r) => r.length > 0);
}

function meaningWords(meaning: string): string[] {
  return meaning.split(/[^a-z0-9']+/).filter((w) => w.length > 0);
}

function index(kanji: Kanji): IndexedKanji {
  const meanings = splitMeanings(kanji.meanings.toLowerCase());
  return {
    kanji,
    readings: [...splitReadings(kanji.onyomi), ...splitReadings(kanji.kunyomi)],
    meanings,
    meaningWords: new Set(meanings.flatMap(meaningWords))
  };
}

function frequencyRank(kanji: Kanji): number {
  return kanji.frequency > 0 ? kanji.frequency : 100000;
}

function gradeRank(kanji: Kanji): number {
  return kanji.grade > 0 ? kanji.grade : 100;
}

/** Frequency-ranked or Jōyō (school grade 1-8) kanji. */
function isCommon(kanji: Kanji): boolean {
  return kanji.frequency > 0 || (kanji.grade >= 1 && kanji.grade <= 8);
}

// Rare kanji whose meaning or reading matches still show up, but after
// common kanji with a looser match (潮 "salt water" before 㴑 "water").
const RARE_KANJI_WEIGHT = 0.4;

/** Common, early-grade, simple kanji first. */
export function compareByCommonness(a: Kanji, b: Kanji): number {
  return (
    frequencyRank(a) - frequencyRank(b) ||
    gradeRank(a) - gradeRank(b) ||
    a.stroke_count - b.stroke_count
  );
}

interface Query {
  text: string;
  lower: string;
  /** The query as hiragana, if it's kana or romaji */
  kana: string | null;
  /** Kanji characters in the query, in order (e.g. a pasted word) */
  characters: string[];
}

function parseQuery(raw: string): Query | null {
  const text = raw.trim();
  if (!text) return null;
  const lower = text.toLowerCase();
  const kana = KANA_ONLY.test(text) ? toHiragana(text) : romajiToHiragana(text);
  const characters = HAS_KANJI.test(text)
    ? Array.from(text).filter((ch) => HAS_KANJI.test(ch))
    : [];
  return { text, lower, kana, characters };
}

/** Relevance of one kanji for the query; 0 means no match. */
function score(entry: IndexedKanji, query: Query): number {
  const { kanji } = entry;
  let best = 0;

  if (query.characters.length > 0) {
    const position = query.characters.indexOf(kanji.character);
    if (position >= 0) {
      // Keep the order of a pasted word: 日本 -> 日, 本
      best = Math.max(best, 1000 - position);
    }
  }

  let textMatch = 0;
  if (query.kana) {
    for (const reading of entry.readings) {
      if (reading === query.kana) textMatch = Math.max(textMatch, 300);
      else if (reading.startsWith(query.kana)) textMatch = Math.max(textMatch, 150);
      else if (reading.includes(query.kana)) textMatch = Math.max(textMatch, 60);
    }
  }
  if (textMatch < 250 && entry.meanings.length > 0) {
    if (entry.meanings.includes(query.lower)) {
      textMatch = Math.max(textMatch, 250);
    } else if (entry.meaningWords.has(query.lower)) {
      textMatch = Math.max(textMatch, 120);
    } else if (entry.meanings.some((m) => m.includes(query.lower))) {
      textMatch = Math.max(textMatch, 40);
    }
  }
  if (!isCommon(kanji)) textMatch *= RARE_KANJI_WEIGHT;

  return Math.max(best, textMatch);
}

/**
 * Builds a search function over `list`. The returned function filters the
 * list to matching kanji, best match first; ties go to the more common
 * kanji.
 */
export function createKanjiSearch(list: Kanji[]): (query: string) => Kanji[] {
  const entries = list.map(index);
  return (rawQuery: string) => {
    const query = parseQuery(rawQuery);
    if (!query) return list;

    const matches: { kanji: Kanji; score: number }[] = [];
    for (const entry of entries) {
      const s = score(entry, query);
      if (s > 0) matches.push({ kanji: entry.kanji, score: s });
    }
    matches.sort((a, b) => b.score - a.score || compareByCommonness(a.kanji, b.kanji));
    return matches.map((m) => m.kanji);
  };
}
