import { describe, expect, it } from 'vitest';
import { speakableReading, speakableReadings } from './speech';

describe('speakable readings', () => {
  it('drops okurigana dots and affix dashes', () => {
    expect(speakableReading('ひと.つ')).toBe('ひとつ');
    expect(speakableReading('-び')).toBe('び');
  });

  it('joins a reading field with pauses', () => {
    expect(speakableReadings('ニチ, ジツ')).toBe('ニチ、ジツ');
    expect(speakableReadings('ひと-, ひと.つ')).toBe('ひと、ひとつ');
    expect(speakableReadings('')).toBe('');
  });
});
