import { describe, expect, it } from 'vitest';
import type { Kanji } from './kanji';
import { createKanjiSearch, normalizeReading } from './search';

function kanji(
  character: string,
  meanings: string,
  onyomi: string,
  kunyomi: string,
  frequency = 0,
  grade = 0
): Kanji {
  return {
    character,
    stroke_count: 4,
    grade,
    jlpt_level: 0,
    frequency,
    onyomi,
    kunyomi,
    meanings,
    nanori: ''
  };
}

const LIST: Kanji[] = [
  kanji('汁', 'soup; juice; broth; sap; gravy; pus', 'ジュウ', 'しる, -しる', 1817),
  kanji('水', 'water; water radical (no.85)', 'スイ', 'みず, みず-', 223, 1),
  kanji('滝', 'waterfall; rapids; cascade', 'ロウ, ソウ', 'たき', 1528),
  kanji('日', 'day; sun; Japan; counter for days', 'ニチ, ジツ', 'ひ, -び, -か', 1, 1),
  kanji('本', 'book; present; main; true; real; counter for long cylindrical things', 'ホン', 'もと', 10, 1),
  kanji('一', 'one; one radical (no.1)', 'イチ, イツ', 'ひと-, ひと.つ', 2, 1),
  kanji('瑞', 'congratulations', 'ズイ, スイ', 'みず, しるし', 0, 9)
];

describe('normalizeReading', () => {
  it('drops okurigana dots and affix dashes and converts katakana', () => {
    expect(normalizeReading('ひと.つ')).toBe('ひとつ');
    expect(normalizeReading('-び')).toBe('び');
    expect(normalizeReading('ニチ')).toBe('にち');
  });
});

describe('createKanjiSearch', () => {
  const search = createKanjiSearch(LIST);
  const chars = (query: string) => search(query).map((k) => k.character);

  it('returns everything for an empty query', () => {
    expect(search('   ')).toBe(LIST);
  });

  it('ranks exact English meanings above partial ones', () => {
    expect(chars('water')).toEqual(['水', '滝']);
  });

  it('finds kanji by romaji readings, common kanji first', () => {
    expect(chars('mizu')).toEqual(['水', '瑞']);
    expect(chars('sui')).toEqual(['水', '瑞']);
    expect(chars('hitotsu')).toEqual(['一']);
  });

  it('matches on and kun readings typed in either kana script', () => {
    expect(chars('すい')).toEqual(['水', '瑞']);
    expect(chars('ニチ')).toEqual(['日']);
  });

  it('finds every kanji of a pasted word, in order', () => {
    expect(chars('日本')).toEqual(['日', '本']);
  });

  it('does not treat English words as romaji', () => {
    // "sun" is valid romaji (すん) but should still find 日 by meaning
    expect(chars('sun')[0]).toBe('日');
    expect(chars('juice')).toEqual(['汁']);
  });
});

describe('rare kanji', () => {
  it('rank below common kanji with a looser match', () => {
    const search = createKanjiSearch([
      kanji('㴑', 'water', '', '', 0, 0),
      kanji('潮', 'tide; salt water; opportunity', 'チョウ', 'しお', 1010, 6)
    ]);
    expect(search('water').map((k) => k.character)).toEqual(['潮', '㴑']);
  });
});
