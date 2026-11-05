// Review history: calendar heatmap and streak helpers.

export interface DayStats {
  reviews: number;
  correct: number;
}

/** Reviews per local calendar day, keyed "YYYY-MM-DD" (from get_review_history). */
export type ReviewHistory = Record<string, DayStats>;

export interface HeatmapDay {
  date: string; // YYYY-MM-DD
  reviews: number;
  correct: number;
  /** 0 = no reviews, 1-4 = increasing activity */
  level: number;
  /** After today: shown as an empty cell */
  future: boolean;
}

export function dayKey(date: Date): string {
  return [
    date.getFullYear(),
    String(date.getMonth() + 1).padStart(2, '0'),
    String(date.getDate()).padStart(2, '0')
  ].join('-');
}

function addDays(date: Date, days: number): Date {
  const result = new Date(date.getFullYear(), date.getMonth(), date.getDate());
  result.setDate(result.getDate() + days);
  return result;
}

function activityLevel(reviews: number): number {
  if (reviews <= 0) return 0;
  if (reviews < 10) return 1;
  if (reviews < 25) return 2;
  if (reviews < 50) return 3;
  return 4;
}

/**
 * The last `weeks` weeks as columns of seven days (Sunday first), ending
 * with the week that contains `today`.
 */
export function buildHeatmap(history: ReviewHistory, today: Date, weeks = 26): HeatmapDay[][] {
  const todayStart = addDays(today, 0);
  const start = addDays(todayStart, -todayStart.getDay() - (weeks - 1) * 7);
  const columns: HeatmapDay[][] = [];
  for (let week = 0; week < weeks; week++) {
    const column: HeatmapDay[] = [];
    for (let weekday = 0; weekday < 7; weekday++) {
      const date = addDays(start, week * 7 + weekday);
      const key = dayKey(date);
      const stats = history[key] ?? { reviews: 0, correct: 0 };
      column.push({
        date: key,
        reviews: stats.reviews,
        correct: stats.correct,
        level: activityLevel(stats.reviews),
        future: date.getTime() > todayStart.getTime()
      });
    }
    columns.push(column);
  }
  return columns;
}

/** The most consecutive days with at least one review. */
export function longestStreak(history: ReviewHistory): number {
  const days = Object.entries(history)
    .filter(([, stats]) => stats.reviews > 0)
    .map(([key]) => key)
    .sort();
  let longest = 0;
  let current = 0;
  let previous: Date | null = null;
  for (const key of days) {
    const [year, month, day] = key.split('-').map(Number);
    const date = new Date(year, month - 1, day);
    current = previous && dayKey(addDays(previous, 1)) === key ? current + 1 : 1;
    longest = Math.max(longest, current);
    previous = date;
  }
  return longest;
}

/** Number of days with at least one review. */
export function daysStudied(history: ReviewHistory): number {
  return Object.values(history).filter((stats) => stats.reviews > 0).length;
}
