// Romaji helpers shared by the kana game and the kanji search.

// Other common spellings for Hepburn romaji (Kunrei/Nihon-shiki and the
// spellings people type into an IME). No variant is a prefix of another
// accepted spelling for the same kana, so the game can advance as soon as
// the input matches.
const HEPBURN_VARIANTS: Record<string, string[]> = {
  shi: ['si'],
  chi: ['ti'],
  tsu: ['tu'],
  fu: ['hu'],
  ji: ['zi'],
  sha: ['sya'],
  shu: ['syu'],
  sho: ['syo'],
  cha: ['tya', 'cya'],
  chu: ['tyu', 'cyu'],
  cho: ['tyo', 'cyo'],
  ja: ['zya', 'jya'],
  ju: ['zyu', 'jyu'],
  jo: ['zyo', 'jyo'],
  wo: ['o'],
};

// ぢ/づ and their yōon are also typed with "d" (Nihon-shiki / IME input).
const D_ROW_VARIANTS: Record<string, string[]> = {
  'ぢ': ['di'],
  'づ': ['du'],
  'ぢゃ': ['dya'],
  'ぢゅ': ['dyu'],
  'ぢょ': ['dyo'],
};

/** Converts katakana to hiragana; other characters are left unchanged. */
export function toHiragana(text: string): string {
  return text.replace(/[ァ-ヶ]/g, (ch) => String.fromCharCode(ch.charCodeAt(0) - 0x60));
}

/** Converts hiragana to katakana; other characters are left unchanged. */
export function toKatakana(text: string): string {
  return text.replace(/[ぁ-ゖ]/g, (ch) => String.fromCharCode(ch.charCodeAt(0) + 0x60));
}

/** Every romaji spelling accepted for `character`, whose main spelling is `romaji`. */
export function acceptedRomaji(character: string, romaji: string): string[] {
  const main = romaji.toLowerCase();
  const variants = [
    main,
    ...(HEPBURN_VARIANTS[main] ?? []),
    ...(D_ROW_VARIANTS[toHiragana(character)] ?? []),
  ];
  return [...new Set(variants)];
}

/** Whether `input` is an accepted romaji spelling for the kana. */
export function isCorrectRomaji(input: string, character: string, romaji: string): boolean {
  return acceptedRomaji(character, romaji).includes(input.trim().toLowerCase());
}
