export type FuzzyMatch = { score: number; positions: number[] };

const WORD_START_BONUS = 8;
const CONSECUTIVE_BONUS = 5;
const PREFIX_BONUS = 12;
const GAP_PENALTY = 1;
const MID_WORD_JUMPS_ALLOWED = 1;

export function fuzzyMatch(query: string, text: string): FuzzyMatch | null {
  const needle = fold(query.replace(/\s+/g, ""));
  if (needle === "") {
    return { score: 0, positions: [] };
  }
  const haystack = fold(text);
  const positions: number[] = [];
  let score = 0;
  let from = 0;
  let midWordJumps = 0;
  for (const character of needle) {
    const found = bestPosition(haystack, character, from, positions.at(-1));
    if (found === -1) {
      return null;
    }
    const previous = positions.at(-1);
    if (previous !== undefined && found === previous + 1) {
      score += CONSECUTIVE_BONUS;
    } else if (previous !== undefined) {
      score -= GAP_PENALTY * Math.min(found - previous - 1, 10);
      if (!isWordStart(haystack, found)) {
        midWordJumps += 1;
      }
      if (midWordJumps > MID_WORD_JUMPS_ALLOWED) {
        return null;
      }
    }
    if (isWordStart(haystack, found)) {
      score += WORD_START_BONUS;
    }
    positions.push(found);
    from = found + 1;
  }
  if (positions[0] === 0) {
    score += PREFIX_BONUS;
  }
  return { score: score - haystack.length * 0.01, positions };
}

function bestPosition(
  haystack: string,
  character: string,
  from: number,
  previous: number | undefined,
): number {
  const first = haystack.indexOf(character, from);
  if (first === -1 || (previous !== undefined && first === previous + 1)) {
    return first;
  }
  for (
    let index = first;
    index !== -1;
    index = haystack.indexOf(character, index + 1)
  ) {
    if (isWordStart(haystack, index)) {
      return index;
    }
  }
  return first;
}

function isWordStart(text: string, index: number): boolean {
  return index === 0 || /[\s/\-_.()]/.test(text[index - 1] ?? "");
}

function fold(text: string): string {
  return text
    .normalize("NFD")
    .replace(/\p{Diacritic}/gu, "")
    .toLocaleLowerCase();
}
