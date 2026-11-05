// Per-device preferences kept in localStorage.

export const SESSION_LIMIT_OPTIONS = [5, 10, 20, 30, 50, 100];
export const DEFAULT_SESSION_LIMIT = 20;

const DUE_LIMIT_KEY = 'flashcards.dueLimit';
const NEW_LIMIT_KEY = 'flashcards.newLimit';

function readNumber(key: string, fallback: number): number {
  try {
    const value = Number(localStorage.getItem(key));
    return Number.isInteger(value) && value > 0 ? value : fallback;
  } catch {
    return fallback;
  }
}

function writeValue(key: string, value: string) {
  try {
    localStorage.setItem(key, value);
  } catch {
    // Storage unavailable: the choice only lasts for this visit
  }
}

export interface SessionLimits {
  /** Reviews of due cards per study session */
  dueLimit: number;
  /** New cards per study session */
  newLimit: number;
}

export function getSessionLimits(): SessionLimits {
  return {
    dueLimit: readNumber(DUE_LIMIT_KEY, DEFAULT_SESSION_LIMIT),
    newLimit: readNumber(NEW_LIMIT_KEY, DEFAULT_SESSION_LIMIT)
  };
}

export function setSessionLimits(limits: SessionLimits) {
  writeValue(DUE_LIMIT_KEY, String(limits.dueLimit));
  writeValue(NEW_LIMIT_KEY, String(limits.newLimit));
}
