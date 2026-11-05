// Japanese pronunciation through the system's text-to-speech voices.
//
// Uses the Web Speech API. Whether it works depends on the platform and on
// a Japanese voice being installed (macOS ships one, Windows needs the
// Japanese language pack, Linux webviews often have none), so callers
// should only show speech controls when hasJapaneseVoice() resolves true.

let voicePromise: Promise<SpeechSynthesisVoice | null> | null = null;

function synth(): SpeechSynthesis | null {
  return typeof window !== 'undefined' && 'speechSynthesis' in window ? window.speechSynthesis : null;
}

function pickJapaneseVoice(voices: SpeechSynthesisVoice[]): SpeechSynthesisVoice | null {
  const japanese = voices.filter((v) => v.lang.toLowerCase().replace('_', '-').startsWith('ja'));
  // Prefer voices that don't need a network connection
  return japanese.find((v) => v.localService) ?? japanese[0] ?? null;
}

function loadVoice(): Promise<SpeechSynthesisVoice | null> {
  const speech = synth();
  if (!speech) return Promise.resolve(null);

  const immediate = pickJapaneseVoice(speech.getVoices());
  if (immediate) return Promise.resolve(immediate);

  // Voices are often loaded asynchronously; wait briefly for them.
  return new Promise((resolve) => {
    const done = () => {
      speech.removeEventListener('voiceschanged', done);
      clearTimeout(timer);
      resolve(pickJapaneseVoice(speech.getVoices()));
    };
    const timer = setTimeout(done, 1500);
    speech.addEventListener('voiceschanged', done);
  });
}

function japaneseVoice(): Promise<SpeechSynthesisVoice | null> {
  if (!voicePromise) {
    const pending = loadVoice();
    voicePromise = pending;
    // Only remember a voice that was found: voices can finish loading
    // after the first check, so look again next time.
    pending.then((voice) => {
      if (!voice && voicePromise === pending) voicePromise = null;
    });
  }
  return voicePromise;
}

/** Whether Japanese text can be spoken on this system. */
export async function hasJapaneseVoice(): Promise<boolean> {
  return (await japaneseVoice()) !== null;
}

/**
 * Text to speak for a dictionary reading: drops the okurigana dot and affix
 * dashes ("ひと.つ" -> "ひとつ", "-び" -> "び").
 */
export function speakableReading(reading: string): string {
  return reading.replace(/[.\-－]/g, '').trim();
}

/** Speaks `text` with the Japanese voice, interrupting anything already playing. */
export async function speakJapanese(text: string): Promise<void> {
  const speech = synth();
  const voice = await japaneseVoice();
  if (!speech || !voice || !text.trim()) return;

  speech.cancel();
  const utterance = new SpeechSynthesisUtterance(text);
  utterance.voice = voice;
  utterance.lang = voice.lang;
  utterance.rate = 0.9;
  speech.speak(utterance);
}

/** All readings of an on/kun field ("ニチ, ジツ") as one phrase with pauses. */
export function speakableReadings(field: string): string {
  return field
    .split(/[,、]/)
    .map(speakableReading)
    .filter((r) => r.length > 0)
    .join('、');
}
