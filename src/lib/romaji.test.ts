import { describe, expect, it } from 'vitest';
import {
  acceptedRomaji,
  isCorrectRomaji,
  romajiToHiragana,
  toHiragana,
  toKatakana
} from './romaji';

describe('romajiToHiragana', () => {
  it.each([
    ['mizu', 'みず'],
    ['sui', 'すい'],
    ['nichi', 'にち'],
    ['hitotsu', 'ひとつ'],
    ['hitotu', 'ひとつ'],
    ['kyou', 'きょう'],
    ['gakkou', 'がっこう'],
    ['kitte', 'きって'],
    ['matcha', 'まっちゃ'],
    ['shinbun', 'しんぶん'],
    ['hon', 'ほん'],
    ['konnichiwa', 'こんにちわ'],
    ['konnbanwa', 'こんばんわ'],
    ["kin'en", 'きんえん'],
    ['kinen', 'きねん'],
    ['jya', 'じゃ'],
    ['sya', 'しゃ'],
    ['tōkyō', 'とうきょう'],
    ['Nihon', 'にほん'],
    ['  sake ', 'さけ'],
  ])('%s -> %s', (input, expected) => {
    expect(romajiToHiragana(input)).toBe(expected);
  });

  it.each(['water', 'xyz', 'hello world', '', '水', 'l'])('rejects %j', (input) => {
    expect(romajiToHiragana(input)).toBeNull();
  });
});

describe('kana conversion', () => {
  it('converts between hiragana and katakana', () => {
    expect(toHiragana('ニチ, ジツ')).toBe('にち, じつ');
    expect(toKatakana('きょう')).toBe('キョウ');
    expect(toHiragana('水 abc')).toBe('水 abc');
  });
});

describe('acceptedRomaji', () => {
  it('accepts common alternative spellings', () => {
    expect(acceptedRomaji('し', 'shi')).toEqual(['shi', 'si']);
    expect(acceptedRomaji('ぢ', 'ji')).toEqual(['ji', 'zi', 'di']);
    expect(acceptedRomaji('ヂョ', 'jo')).toContain('dyo');
    expect(acceptedRomaji('を', 'wo')).toEqual(['wo', 'o']);
    expect(acceptedRomaji('か', 'ka')).toEqual(['ka']);
  });

  it('never accepts a prefix of another accepted spelling', () => {
    // The game advances as soon as the input matches, so a prefix would
    // make the longer spelling impossible to type.
    for (const [character, romaji] of [['し', 'shi'], ['ち', 'chi'], ['ちゃ', 'cha'], ['ぢゃ', 'ja'], ['を', 'wo'], ['ん', 'n']]) {
      const spellings = acceptedRomaji(character, romaji);
      for (const a of spellings) {
        for (const b of spellings) {
          if (a !== b) expect(b.startsWith(a)).toBe(false);
        }
      }
    }
  });

  it('checks answers case- and whitespace-insensitively', () => {
    expect(isCorrectRomaji(' TSU ', 'つ', 'tsu')).toBe(true);
    expect(isCorrectRomaji('tu', 'ツ', 'tsu')).toBe(true);
    expect(isCorrectRomaji('su', 'つ', 'tsu')).toBe(false);
  });
});
