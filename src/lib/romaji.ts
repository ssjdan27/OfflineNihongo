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

// Romaji -> hiragana, Hepburn plus the common Kunrei/Nihon-shiki spellings.
const ROMAJI_TO_HIRAGANA: Record<string, string> = (() => {
  const table: Record<string, string> = {
    a: 'あ', i: 'い', u: 'う', e: 'え', o: 'お',
    ka: 'か', ki: 'き', ku: 'く', ke: 'け', ko: 'こ',
    ga: 'が', gi: 'ぎ', gu: 'ぐ', ge: 'げ', go: 'ご',
    sa: 'さ', shi: 'し', si: 'し', su: 'す', se: 'せ', so: 'そ',
    za: 'ざ', ji: 'じ', zi: 'じ', zu: 'ず', ze: 'ぜ', zo: 'ぞ',
    ta: 'た', chi: 'ち', ti: 'ち', tsu: 'つ', tu: 'つ', te: 'て', to: 'と',
    da: 'だ', di: 'ぢ', du: 'づ', dzu: 'づ', de: 'で', do: 'ど',
    na: 'な', ni: 'に', nu: 'ぬ', ne: 'ね', no: 'の',
    ha: 'は', hi: 'ひ', fu: 'ふ', hu: 'ふ', he: 'へ', ho: 'ほ',
    ba: 'ば', bi: 'び', bu: 'ぶ', be: 'べ', bo: 'ぼ',
    pa: 'ぱ', pi: 'ぴ', pu: 'ぷ', pe: 'ぺ', po: 'ぽ',
    ma: 'ま', mi: 'み', mu: 'む', me: 'め', mo: 'も',
    ya: 'や', yu: 'ゆ', yo: 'よ',
    ra: 'ら', ri: 'り', ru: 'る', re: 'れ', ro: 'ろ',
    wa: 'わ', wo: 'を',
  };
  // Yōon: consonant + small ya/yu/yo
  const yoon: Record<string, string> = {
    ky: 'き', gy: 'ぎ', sh: 'し', sy: 'し', j: 'じ', zy: 'じ', jy: 'じ',
    ch: 'ち', ty: 'ち', cy: 'ち', dy: 'ぢ', ny: 'に', hy: 'ひ', by: 'び',
    py: 'ぴ', my: 'み', ry: 'り',
  };
  const small: Record<string, string> = { a: 'ゃ', u: 'ゅ', o: 'ょ' };
  for (const [prefix, kana] of Object.entries(yoon)) {
    for (const [vowel, smallKana] of Object.entries(small)) {
      table[prefix + vowel] = kana + smallKana;
    }
  }
  return table;
})();

const VOWELS = new Set(['a', 'i', 'u', 'e', 'o']);

// Hepburn long vowels (tōkyō, kūki) and their circumflex variants
const LONG_VOWELS: Record<string, string> = {
  'ā': 'aa', 'ī': 'ii', 'ū': 'uu', 'ē': 'ei', 'ō': 'ou',
  'â': 'aa', 'î': 'ii', 'û': 'uu', 'ê': 'ei', 'ô': 'ou',
};

/**
 * Converts romaji (e.g. "mizu", "kyou", "gakkou", "shinbun") to hiragana.
 * Returns null if the text isn't entirely romaji, so English search terms
 * like "water" aren't mistaken for readings.
 */
export function romajiToHiragana(input: string): string | null {
  const text = input
    .trim()
    .toLowerCase()
    .replace(/[āīūēōâîûêô]/g, (vowel) => LONG_VOWELS[vowel]);
  if (!text) return null;

  let result = '';
  let i = 0;
  while (i < text.length) {
    const ch = text[i];

    // Doubled consonant -> small tsu (gakkou, kitte, matcha)
    const next = text[i + 1];
    if (
      next !== undefined &&
      ch !== 'n' &&
      /[bcdfghjkmprstvwz]/.test(ch) &&
      (ch === next || (ch === 't' && next === 'c'))
    ) {
      result += 'っ';
      i += 1;
      continue;
    }

    // ん: "n'" (kin'en), "nn" (konnichiwa = ko-n-ni-chi-wa, or IME-style
    // "konnbanwa"), and n before a consonant or at the end (shinbun, hon)
    if (ch === 'n') {
      const afterNext = text[i + 2];
      if (next === "'") {
        result += 'ん';
        i += 2;
        continue;
      }
      if (next === 'n') {
        result += 'ん';
        // keep the second n if it starts the next syllable (konnichiwa)
        i += afterNext !== undefined && (VOWELS.has(afterNext) || afterNext === 'y') ? 1 : 2;
        continue;
      }
      if (next === undefined || (!VOWELS.has(next) && next !== 'y')) {
        result += 'ん';
        i += 1;
        continue;
      }
    }

    let matched = false;
    for (const length of [3, 2, 1]) {
      const chunk = text.slice(i, i + length);
      const kana = ROMAJI_TO_HIRAGANA[chunk];
      if (chunk.length === length && kana) {
        result += kana;
        i += length;
        matched = true;
        break;
      }
    }
    if (!matched) return null;
  }
  return result;
}
