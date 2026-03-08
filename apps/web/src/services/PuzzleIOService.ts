import type { PuzzleJson, SolveResult } from '../hooks/useNonogramStore';

export type { PuzzleJson, SolveResult };

export type PuzzleIOError =
  | { kind: 'parse'; message: string }
  | { kind: 'schema'; message: string };

function downloadJson(data: unknown, filename: string): void {
  const json = JSON.stringify(data, null, 2);
  const blob = new Blob([json], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

export const PuzzleIOService = {
  exportPuzzle(rowClues: number[][], colClues: number[][]): void {
    downloadJson({ row_clues: rowClues, col_clues: colClues }, 'puzzle.json');
  },

  exportTemplate(rows: number, cols: number): void {
    const row_clues = Array.from({ length: rows }, () => [] as number[]);
    const col_clues = Array.from({ length: cols }, () => [] as number[]);
    downloadJson({ row_clues, col_clues }, 'puzzle-template.json');
  },

  exportSolution(result: SolveResult): void {
    downloadJson(
      { status: result.status, solutions: result.solutions },
      'solution.json'
    );
  },

  exportSolutionGrid(cells: boolean[][]): void {
    downloadJson({ cells }, 'solution-grid.json');
  },

  importSolutionGrid(file: File): Promise<boolean[][]> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = (e) => {
        const text = e.target?.result as string;
        let parsed: unknown;
        try {
          parsed = JSON.parse(text);
        } catch {
          reject({ kind: 'parse', message: 'JSONのパースに失敗しました' } as PuzzleIOError);
          return;
        }
        if (
          typeof parsed !== 'object' ||
          parsed === null ||
          !Array.isArray((parsed as Record<string, unknown>).cells)
        ) {
          reject({ kind: 'schema', message: 'cells フィールドがありません' } as PuzzleIOError);
          return;
        }
        const p = parsed as Record<string, unknown>;
        const isBoolArray = (arr: unknown): arr is boolean[] =>
          Array.isArray(arr) && arr.every(x => typeof x === 'boolean');
        const isBoolArrayArray = (arr: unknown): arr is boolean[][] =>
          Array.isArray(arr) && arr.every(isBoolArray);
        if (!isBoolArrayArray(p.cells)) {
          reject({ kind: 'schema', message: 'cells の形式が不正です（boolean[][] が必要です）' } as PuzzleIOError);
          return;
        }
        resolve(p.cells as boolean[][]);
      };
      reader.onerror = () => {
        reject({ kind: 'parse', message: 'ファイルの読み込みに失敗しました' } as PuzzleIOError);
      };
      reader.readAsText(file);
    });
  },

  importPuzzle(file: File): Promise<PuzzleJson> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = (e) => {
        const text = e.target?.result as string;
        let parsed: unknown;
        try {
          parsed = JSON.parse(text);
        } catch {
          reject({ kind: 'parse', message: 'JSONのパースに失敗しました' } as PuzzleIOError);
          return;
        }
        if (
          typeof parsed !== 'object' ||
          parsed === null ||
          !Array.isArray((parsed as Record<string, unknown>).row_clues) ||
          !Array.isArray((parsed as Record<string, unknown>).col_clues)
        ) {
          reject({ kind: 'schema', message: 'row_clues または col_clues フィールドがありません' } as PuzzleIOError);
          return;
        }
        const p = parsed as Record<string, unknown>;
        const isNumberArray = (arr: unknown): arr is number[] =>
          Array.isArray(arr) && arr.every(x => typeof x === 'number' && Number.isInteger(x) && x >= 0);
        const isNumberArrayArray = (arr: unknown): arr is number[][] =>
          Array.isArray(arr) && arr.every(isNumberArray);
        if (!isNumberArrayArray(p.row_clues) || !isNumberArrayArray(p.col_clues)) {
          reject({ kind: 'schema', message: 'row_clues または col_clues の形式が不正です' } as PuzzleIOError);
          return;
        }
        resolve({ row_clues: p.row_clues as number[][], col_clues: p.col_clues as number[][] });
      };
      reader.onerror = () => {
        reject({ kind: 'parse', message: 'ファイルの読み込みに失敗しました' } as PuzzleIOError);
      };
      reader.readAsText(file);
    });
  },
};
