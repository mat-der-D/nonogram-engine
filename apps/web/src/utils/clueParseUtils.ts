export type ParseResult =
  | { ok: true; clues: number[] }
  | { ok: false; error: string };

export function parseClueString(input: string): ParseResult {
  const trimmed = input.trim();
  if (trimmed === '') return { ok: true, clues: [] };

  const parts = trimmed.split(/[\s,]+/).filter(s => s.length > 0);
  const clues: number[] = [];

  for (const part of parts) {
    if (!/^\d+$/.test(part)) {
      return { ok: false, error: 'ヒントには正の整数を入力してください' };
    }
    const n = parseInt(part, 10);
    if (n <= 0) {
      return { ok: false, error: 'ヒントには正の整数を入力してください' };
    }
    clues.push(n);
  }

  return { ok: true, clues };
}
