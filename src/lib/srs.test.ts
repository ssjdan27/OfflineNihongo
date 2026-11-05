import { describe, expect, it } from 'vitest';
import { cardStage, chooseReviewType, formatDue, type SrsCard } from './srs';

function card(overrides: Partial<SrsCard>): SrsCard {
  return {
    id: 'x',
    character: '日',
    level: 0,
    interval: 1,
    ease_factor: 2.5,
    next_review: '2026-11-05T09:00:00+00:00',
    total_reviews: 0,
    correct_reviews: 0,
    created_at: '',
    last_reviewed: null,
    streak: 0,
    ...overrides
  };
}

describe('cardStage', () => {
  it('distinguishes new, learning and mature cards', () => {
    expect(cardStage(card({}))).toBe('new');
    expect(cardStage(card({ total_reviews: 1, level: 0 }))).toBe('learning');
    expect(cardStage(card({ total_reviews: 5, level: 4 }))).toBe('mature');
  });
});

describe('formatDue', () => {
  const now = new Date(2026, 10, 5, 20, 0); // Nov 5, 8pm local

  it('describes due dates relative to today', () => {
    expect(formatDue(new Date(2026, 10, 1, 9).toISOString(), now)).toBe('Due today');
    expect(formatDue(new Date(2026, 10, 5, 23).toISOString(), now)).toBe('Due today');
    expect(formatDue(new Date(2026, 10, 6, 1).toISOString(), now)).toBe('Due tomorrow');
    expect(formatDue(new Date(2026, 10, 10, 1).toISOString(), now)).toBe('Due in 5 days');
    expect(formatDue(new Date(2027, 2, 3, 12).toISOString(), now)).toBe('Due Mar 3, 2027');
    expect(formatDue('garbage', now)).toBe('Due now');
  });
});

describe('chooseReviewType', () => {
  const both = { hasMeanings: true, hasReadings: true, recall: false };

  it('asks about meanings and readings 60/40', () => {
    expect(chooseReviewType(both, 0)).toBe('meaning');
    expect(chooseReviewType(both, 0.59)).toBe('meaning');
    expect(chooseReviewType(both, 0.61)).toBe('reading');
    expect(chooseReviewType(both, 0.999)).toBe('reading');
  });

  it('adds recall reviews when enabled', () => {
    const withRecall = { ...both, recall: true };
    const counts = { meaning: 0, reading: 0, recall: 0 };
    for (let i = 0; i < 1000; i++) counts[chooseReviewType(withRecall, i / 1000)]++;
    expect(counts.recall).toBeGreaterThan(200);
    expect(counts.meaning).toBeGreaterThan(counts.reading);
  });

  it('only asks for what the dictionary has', () => {
    expect(chooseReviewType({ hasMeanings: false, hasReadings: true, recall: true }, 0)).toBe('reading');
    expect(chooseReviewType({ hasMeanings: true, hasReadings: false, recall: false }, 0.9)).toBe('meaning');
    expect(chooseReviewType({ hasMeanings: false, hasReadings: false, recall: true }, 0.5)).toBe('meaning');
  });
});
