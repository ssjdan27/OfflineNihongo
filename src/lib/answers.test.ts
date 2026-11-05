import { describe, expect, it } from 'vitest';
import { acceptedReadingForms, matchReading, typedReadingToKana } from './answers';

describe('typedReadingToKana', () => {
  it('accepts romaji and either kana script', () => {
    expect(typedReadingToKana('mizu')).toBe('みず');
    expect(typedReadingToKana('スイ')).toBe('すい');
    expect(typedReadingToKana(' hito tsu ')).toBe('ひとつ');
    expect(typedReadingToKana('water')).toBeNull();
    expect(typedReadingToKana('')).toBeNull();
  });
});

describe('acceptedReadingForms', () => {
  it('accepts the whole word and the stem of okurigana readings', () => {
    expect(acceptedReadingForms('ひと.つ')).toEqual(['ひとつ', 'ひと']);
    expect(acceptedReadingForms('-び')).toEqual(['び']);
    expect(acceptedReadingForms('ニチ')).toEqual(['にち']);
  });
});

describe('matchReading', () => {
  const onyomi = 'イチ, イツ';
  const kunyomi = 'ひと-, ひと.つ';

  it('returns the matched dictionary reading', () => {
    expect(matchReading('ichi', onyomi, kunyomi)).toBe('イチ');
    expect(matchReading('hitotsu', onyomi, kunyomi)).toBe('ひと.つ');
    expect(matchReading('ひと', onyomi, kunyomi)).toBe('ひと-');
    expect(matchReading('いつ', onyomi, kunyomi)).toBe('イツ');
  });

  it('rejects wrong or unreadable answers', () => {
    expect(matchReading('ni', onyomi, kunyomi)).toBeNull();
    expect(matchReading('one', onyomi, kunyomi)).toBeNull();
    expect(matchReading('', onyomi, kunyomi)).toBeNull();
  });
});
