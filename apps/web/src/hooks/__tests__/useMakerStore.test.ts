import { describe, it, expect } from 'vitest';
import { reducer, makeInitialState } from '../useMakerStore';
import type { MakerState, MakerAction } from '../useMakerStore';

function applyActions(state: MakerState, actions: MakerAction[]): MakerState {
  return actions.reduce((s, a) => reducer(s, a), state);
}

describe('useMakerStore reducer', () => {
  // --- Task 3.4 テスト ---

  it('COMMIT_HISTORY + TOGGLE_CELL で 1 undo ステップになる', () => {
    const initial = makeInitialState();
    // セルをトグル（COMMIT_HISTORY → TOGGLE_CELL の順）
    const state = applyActions(initial, [
      { type: 'COMMIT_HISTORY' },
      { type: 'TOGGLE_CELL', row: 0, col: 0 },
    ]);
    expect(state.cells[0][0]).toBe(true);
    expect(state.history.length).toBe(1);
    expect(state.future.length).toBe(0);

    // UNDO で 1 ステップ戻る
    const undone = reducer(state, { type: 'UNDO' });
    expect(undone.cells[0][0]).toBe(false);
    expect(undone.history.length).toBe(0);
    expect(undone.future.length).toBe(1);
  });

  it('複数ドラッグ操作（COMMIT_HISTORY 1 回 + 複数 DRAG_CELL）が 1 undo ステップになる', () => {
    const initial = makeInitialState();
    const state = applyActions(initial, [
      { type: 'COMMIT_HISTORY' },
      { type: 'DRAG_CELL', row: 0, col: 0, action: 'fill' },
      { type: 'DRAG_CELL', row: 0, col: 1, action: 'fill' },
      { type: 'DRAG_CELL', row: 0, col: 2, action: 'fill' },
    ]);
    expect(state.cells[0][0]).toBe(true);
    expect(state.cells[0][1]).toBe(true);
    expect(state.cells[0][2]).toBe(true);
    expect(state.history.length).toBe(1); // 履歴は 1 ステップ

    const undone = reducer(state, { type: 'UNDO' });
    expect(undone.cells[0][0]).toBe(false);
    expect(undone.cells[0][1]).toBe(false);
    expect(undone.history.length).toBe(0);
  });

  it('UNDO/REDO で history/future スタックが正しく操作される', () => {
    const initial = makeInitialState();

    // 2 ステップ操作
    const s1 = applyActions(initial, [
      { type: 'COMMIT_HISTORY' },
      { type: 'TOGGLE_CELL', row: 0, col: 0 },
    ]);
    const s2 = applyActions(s1, [
      { type: 'COMMIT_HISTORY' },
      { type: 'TOGGLE_CELL', row: 1, col: 1 },
    ]);

    expect(s2.cells[0][0]).toBe(true);
    expect(s2.cells[1][1]).toBe(true);
    expect(s2.history.length).toBe(2);

    // UNDO × 1
    const u1 = reducer(s2, { type: 'UNDO' });
    expect(u1.cells[1][1]).toBe(false);
    expect(u1.history.length).toBe(1);
    expect(u1.future.length).toBe(1);

    // REDO × 1
    const r1 = reducer(u1, { type: 'REDO' });
    expect(r1.cells[1][1]).toBe(true);
    expect(r1.history.length).toBe(2);
    expect(r1.future.length).toBe(0);
  });

  it('新しい操作で future がクリアされる', () => {
    const initial = makeInitialState();
    const withHistory = applyActions(initial, [
      { type: 'COMMIT_HISTORY' },
      { type: 'TOGGLE_CELL', row: 0, col: 0 },
    ]);
    const undone = reducer(withHistory, { type: 'UNDO' });
    expect(undone.future.length).toBe(1);

    // COMMIT_HISTORY で future がクリアされる
    const cleared = reducer(undone, { type: 'COMMIT_HISTORY' });
    expect(cleared.future.length).toBe(0);
  });

  it('RESET_GRID で cells が全 false になり history にスナップショットが追加される', () => {
    const initial = makeInitialState();
    const withCell = applyActions(initial, [
      { type: 'COMMIT_HISTORY' },
      { type: 'TOGGLE_CELL', row: 0, col: 0 },
    ]);
    expect(withCell.cells[0][0]).toBe(true);

    const reset = reducer(withCell, { type: 'RESET_GRID' });
    const allBlank = reset.cells.every(row => row.every(c => !c));
    expect(allBlank).toBe(true);
    // history には COMMIT_HISTORY のスナップショット + RESET_GRID のスナップショット
    expect(reset.history.length).toBe(2);
    expect(reset.future.length).toBe(0);
  });

  it('SET_DIMENSIONS でグリッドリサイズ後も既存 cells が保持される', () => {
    const initial = makeInitialState();
    const withCell = applyActions(initial, [
      { type: 'COMMIT_HISTORY' },
      { type: 'TOGGLE_CELL', row: 0, col: 0 },
    ]);
    expect(withCell.cells[0][0]).toBe(true);

    // 縮小
    const smaller = reducer(withCell, { type: 'SET_DIMENSIONS', width: 3, height: 3 });
    expect(smaller.gridWidth).toBe(3);
    expect(smaller.gridHeight).toBe(3);
    expect(smaller.cells[0][0]).toBe(true); // 保持される
    expect(smaller.cells.length).toBe(3);
    expect(smaller.cells[0].length).toBe(3);

    // 拡大
    const larger = reducer(smaller, { type: 'SET_DIMENSIONS', width: 5, height: 5 });
    expect(larger.gridWidth).toBe(5);
    expect(larger.cells[0][0]).toBe(true); // 引き続き保持
    expect(larger.cells[0][4]).toBe(false); // 新規セルは false
  });

  it('SET_DIMENSIONS でトリミングされたセルは false になる', () => {
    const initial = makeInitialState();
    // col 19 (最後の列) にセルをセット
    const withCell = applyActions(initial, [
      { type: 'COMMIT_HISTORY' },
      { type: 'TOGGLE_CELL', row: 0, col: 19 },
    ]);
    expect(withCell.cells[0][19]).toBe(true);

    // 幅を 5 に縮小 → col 19 は消える
    const smaller = reducer(withCell, { type: 'SET_DIMENSIONS', width: 5, height: 5 });
    expect(smaller.cells[0].length).toBe(5);
    // col 19 のデータは切り捨て（アクセス不可）
  });

  it('history が 100 件を超えると古いエントリが破棄される', () => {
    let state = makeInitialState();
    for (let i = 0; i < 105; i++) {
      state = reducer(state, { type: 'COMMIT_HISTORY' });
    }
    expect(state.history.length).toBe(100);
  });
});
