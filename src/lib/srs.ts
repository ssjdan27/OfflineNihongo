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
