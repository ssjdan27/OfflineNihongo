// Checking typed answers in flashcard reviews.

import { romajiToHiragana, toHiragana } from './romaji';

const KANA_ONLY = /^[ぁ-ゖァ-ヺー]+$/;

/** Typed romaji or kana as hiragana, or null if it's neither. */
export function typedReadingToKana(input: string): string | null {
  const text = input.trim().replace(/\s+/g, '');
  if (!text) return null;
  return KANA_ONLY.test(text) ? toHiragana(text) : romajiToHiragana(text);
}

/**
 * Forms of a dictionary reading that count as correct, in hiragana.
 * "ひと.つ" accepts the full word ひとつ and the stem ひと; "-び" and "ひと-"
 * drop the affix dash; on'yomi in katakana are converted.
 */
export function acceptedReadingForms(reading: string): string[] {
  const plain = toHiragana(reading.trim().replace(/[-－]/g, ''));
  if (!plain) return [];
  const [stem] = plain.split('.');
  return [...new Set([plain.replace(/\./g, ''), stem])];
}

/** Splits an on/kun reading field such as "ニチ, ジツ" or "ひと-, ひと.つ". */
export function splitReadingField(field: string): string[] {
  return field
    .split(/[,、]/)
    .map((r) => r.trim())
    .filter((r) => r.length > 0);
}

/**
 * The dictionary reading that the typed answer matches, or null if it
 * doesn't match any of them.
 */
export function matchReading(input: string, onyomi: string, kunyomi: string): string | null {
  const kana = typedReadingToKana(input);
  if (!kana) return null;
  for (const reading of [...splitReadingField(onyomi), ...splitReadingField(kunyomi)]) {
    if (acceptedReadingForms(reading).includes(kana)) return reading;
  }
  return null;
}
