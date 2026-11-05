// Shared kanji types and display helpers.

export interface Kanji {
  character: string;
  stroke_count: number;
  grade: number;
  jlpt_level: number;
  frequency: number;
  onyomi: string;
  kunyomi: string;
  /** Meanings separated by "; " (a single meaning can contain commas). */
  meanings: string;
  nanori: string;
}

export interface KanjiLookup {
  character: string;
  stroke_count: number;
  grade: number;
  jlpt_level: number;
  frequency: number;
  onyomi: string;
  kunyomi: string;
  meanings: string[];
  nanori: string[];
}

export function splitMeanings(meanings: string): string[] {
  return meanings
    .split(';')
    .map((m) => m.trim())
    .filter((m) => m.length > 0);
}

export function getGradeLabel(grade: number): string {
  if (grade >= 1 && grade <= 6) return `Grade ${grade}`;
  if (grade === 8) return 'Jr. High';
  if (grade === 9) return 'Jinmeiyou';
  if (grade === 10) return 'Jinmeiyou Variant';
  return 'Unknown';
}

// KANJIDIC2 records the pre-2010 JLPT levels (1 = hardest, 4 = easiest),
// which don't map one-to-one onto today's N1-N5.
const JLPT_APPROXIMATE_N_LEVEL: Record<number, string> = {
  1: 'N1',
  2: 'N2–N3',
  3: 'N4',
  4: 'N5',
};

/** Short label for an old JLPT level, e.g. "JLPT 4 (≈N5)". */
export function getJlptLabel(level: number): string {
  const approx = JLPT_APPROXIMATE_N_LEVEL[level];
  return approx ? `JLPT ${level} (≈${approx})` : '';
}

export const JLPT_LEVEL_HELP =
  'Pre-2010 JLPT level from KANJIDIC2 (4 = easiest). The N-level is an approximation.';
