import { describe, it, expect } from 'vitest';
import { _reducer, _makeInitialState } from '../useNonogramStore';

const reducer = _reducer;
const makeInitialState = _makeInitialState;

describe('useNonogramStore reducer', () => {
  describe('SET_DIMENSIONS', () => {
    it('setDimensions後に配列長が正しいことを確認する', () => {
      const state = makeInitialState();
      const next = reducer(state, { type: 'SET_DIMENSIONS', rows: 3, cols: 7 });
      expect(next.rows).toBe(3);
      expect(next.cols).toBe(7);
      expect(next.rowClueInputs).toHaveLength(3);
      expect(next.colClueInputs).toHaveLength(7);
      expect(next.gridCells).toHaveLength(3);
      next.gridCells.forEach(row => expect(row).toHaveLength(7));
    });

    it('既存ヒント入力を保持する', () => {
      const state = makeInitialState();
      const s1 = reducer(state, { type: 'UPDATE_ROW_CLUE', index: 0, raw: '1 2' });
      const s2 = reducer(s1, { type: 'SET_DIMENSIONS', rows: 3, cols: 3 });
      expect(s2.rowClueInputs[0]).toBe('1 2');
    });
  });

  describe('TOGGLE_CELL', () => {
    it('セルのトグル動作を確認する', () => {
      const state = makeInitialState();
      expect(state.gridCells[0][0]).toBe(false);

      const s1 = reducer(state, { type: 'TOGGLE_CELL', row: 0, col: 0 });
      expect(s1.gridCells[0][0]).toBe(true);

      const s2 = reducer(s1, { type: 'TOGGLE_CELL', row: 0, col: 0 });
      expect(s2.gridCells[0][0]).toBe(false);
    });

    it('他のセルは変更されない', () => {
      const state = makeInitialState();
      const next = reducer(state, { type: 'TOGGLE_CELL', row: 0, col: 0 });
      expect(next.gridCells[0][1]).toBe(false);
      expect(next.gridCells[1][0]).toBe(false);
    });

    it('セルトグル後にヒントが自動更新される', () => {
      const state = reducer(makeInitialState(), { type: 'SET_DIMENSIONS', rows: 2, cols: 3 });
      const s1 = reducer(state, { type: 'TOGGLE_CELL', row: 0, col: 0 });
      const s2 = reducer(s1, { type: 'TOGGLE_CELL', row: 0, col: 1 });
      expect(s2.rowClueInputs[0]).toBe('2');
    });
  });

  describe('SET_SOLVE_PHASE', () => {
    it('idle → solving のフェーズ遷移', () => {
      const state = makeInitialState();
      expect(state.solvePhase.phase).toBe('idle');

      const solving = reducer(state, { type: 'SET_SOLVE_PHASE', phase: { phase: 'solving' } });
      expect(solving.solvePhase.phase).toBe('solving');
    });

    it('solving → done のフェーズ遷移', () => {
      const state = makeInitialState();
      const solving = reducer(state, { type: 'SET_SOLVE_PHASE', phase: { phase: 'solving' } });
      const result = { status: 'unique' as const, solutions: [[[true, false]]] };
      const done = reducer(solving, { type: 'SET_SOLVE_PHASE', phase: { phase: 'done', result } });

      expect(done.solvePhase.phase).toBe('done');
      if (done.solvePhase.phase === 'done') {
        expect(done.solvePhase.result.status).toBe('unique');
      }
    });
  });

  describe('UPDATE_ROW_CLUE バリデーション', () => {
    it('不正なヒントはclueErrorsに記録される', () => {
      const state = makeInitialState();
      const next = reducer(state, { type: 'UPDATE_ROW_CLUE', index: 0, raw: 'abc' });
      expect(next.clueErrors['row-0']).toBeTruthy();
    });

    it('空ヒントはエラーなし', () => {
      const state = makeInitialState();
      const next = reducer(state, { type: 'UPDATE_ROW_CLUE', index: 0, raw: '' });
      expect(next.clueErrors['row-0']).toBeUndefined();
    });
  });

  describe('LOAD_PUZZLE', () => {
    it('パズルロード後にヒント入力が更新される', () => {
      const state = makeInitialState();
      const puzzle = { row_clues: [[1, 2], [3]], col_clues: [[1], [2], [4]] };
      const next = reducer(state, { type: 'LOAD_PUZZLE', puzzle });
      expect(next.rows).toBe(2);
      expect(next.cols).toBe(3);
      expect(next.rowClueInputs[0]).toBe('1 2');
      expect(next.rowClueInputs[1]).toBe('3');
      expect(next.colClueInputs[2]).toBe('4');
    });
  });
});
