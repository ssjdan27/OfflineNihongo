// Flashcard (SRS) types and display helpers shared by the pages.

export interface SrsCard {
  id: string;
  character: string;
  level: number;
  interval: number;
  ease_factor: number;
  next_review: string; // RFC 3339
  total_reviews: number;
  correct_reviews: number;
  created_at: string;
  last_reviewed: string | null;
  streak: number;
}

/** Matches srs::MATURE_LEVEL in the backend. */
export const MATURE_LEVEL = 4;

export type CardStage = 'new' | 'learning' | 'mature';

export function cardStage(card: SrsCard): CardStage {
  if (card.total_reviews === 0) return 'new';
  return card.level >= MATURE_LEVEL ? 'mature' : 'learning';
}

export const STAGE_LABELS: Record<CardStage, string> = {
  new: 'New',
  learning: 'Learning',
  mature: 'Mature'
};

function startOfDay(date: Date): number {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate()).getTime();
}

/** "Due today", "Due tomorrow", "Due in 5 days", "Due Mar 3, 2027" */
export function formatDue(nextReview: string, now: Date = new Date()): string {
  const due = new Date(nextReview);
  if (Number.isNaN(due.getTime())) return 'Due now';
  const days = Math.round((startOfDay(due) - startOfDay(now)) / 86_400_000);
  if (days <= 0) return 'Due today';
  if (days === 1) return 'Due tomorrow';
  if (days < 30) return `Due in ${days} days`;
  return `Due ${due.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })}`;
}

export type ReviewType = 'meaning' | 'reading' | 'recall';

/**
 * Picks what to ask about a card: its meaning, its reading, or (when recall
 * reviews are on) which kanji matches a meaning. Only asks for things the
 * dictionary has an answer for. `random` is in [0, 1).
 */
export function chooseReviewType(
  options: { hasMeanings: boolean; hasReadings: boolean; recall: boolean },
  random: number = Math.random()
): ReviewType {
  const weights: [ReviewType, number][] = [];
  if (options.hasMeanings) weights.push(['meaning', 0.6]);
  if (options.hasReadings) weights.push(['reading', 0.4]);
  if (options.recall && options.hasMeanings) weights.push(['recall', 0.35]);
  if (weights.length === 0) return 'meaning';

  const total = weights.reduce((sum, [, weight]) => sum + weight, 0);
  let remaining = random * total;
  for (const [type, weight] of weights) {
    if (remaining < weight) return type;
    remaining -= weight;
  }
  return weights[weights.length - 1][0];
}

export type DeckSort = 'due' | 'missed' | 'recent';

function missedCount(card: SrsCard): number {
  return card.total_reviews - card.correct_reviews;
}

function dueTime(card: SrsCard): number {
  // Never-reviewed cards aren't due yet; they go after scheduled reviews
  if (card.total_reviews === 0) return Number.POSITIVE_INFINITY;
  const time = new Date(card.next_review).getTime();
  return Number.isNaN(time) ? Number.NEGATIVE_INFINITY : time;
}

/** Compares two cards for sorting the deck (missing cards sort last). */
export function compareCards(sort: DeckSort, a: SrsCard | undefined, b: SrsCard | undefined): number {
  if (!a || !b) return a ? -1 : b ? 1 : 0;
  switch (sort) {
    case 'due':
      return dueTime(a) - dueTime(b) || 0;
    case 'missed':
      return missedCount(b) - missedCount(a) || b.total_reviews - a.total_reviews;
    case 'recent':
      return b.created_at.localeCompare(a.created_at);
  }
}

/** A card that keeps being forgotten: 4+ misses and under half correct. */
export function isTroubleCard(card: SrsCard): boolean {
  return missedCount(card) >= 4 && card.correct_reviews / card.total_reviews < 0.5;
}
