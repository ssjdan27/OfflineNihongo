import { describe, expect, it } from 'vitest';
import { buildHeatmap, daysStudied, longestStreak, type ReviewHistory } from './history';

describe('buildHeatmap', () => {
  // Thursday, Nov 5 2026
  const today = new Date(2026, 10, 5, 20, 30);
  const history: ReviewHistory = {
    '2026-11-05': { reviews: 12, correct: 10 },
    '2026-11-01': { reviews: 3, correct: 3 },
    '2026-05-01': { reviews: 99, correct: 90 } // older than 26 weeks
  };

  it('ends with the current week, Sunday first', () => {
    const weeks = buildHeatmap(history, today);
    expect(weeks).toHaveLength(26);
    expect(weeks.every((week) => week.length === 7)).toBe(true);

    const lastWeek = weeks[25];
    expect(lastWeek[0].date).toBe('2026-11-01'); // Sunday
    expect(lastWeek[4].date).toBe('2026-11-05'); // today
    expect(lastWeek[4].future).toBe(false);
    expect(lastWeek[5].future).toBe(true);
    expect(weeks[0][0].date).toBe('2026-05-10');
  });

  it('scales activity levels with the number of reviews', () => {
    const lastWeek = buildHeatmap(history, today)[25];
    expect(lastWeek[0]).toMatchObject({ reviews: 3, level: 1 });
    expect(lastWeek[4]).toMatchObject({ reviews: 12, correct: 10, level: 2 });
    expect(lastWeek[1]).toMatchObject({ reviews: 0, level: 0 });
  });
});

describe('streaks', () => {
  it('finds the longest run of consecutive days, across month ends', () => {
    const history: ReviewHistory = {
      '2026-10-30': { reviews: 1, correct: 1 },
      '2026-10-31': { reviews: 5, correct: 4 },
      '2026-11-01': { reviews: 2, correct: 2 },
      '2026-11-03': { reviews: 1, correct: 0 },
      '2026-11-04': { reviews: 0, correct: 0 }
    };
    expect(longestStreak(history)).toBe(3);
    expect(daysStudied(history)).toBe(4);
    expect(longestStreak({})).toBe(0);
  });
});
