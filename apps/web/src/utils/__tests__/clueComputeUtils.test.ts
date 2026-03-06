import { describe, it, expect } from 'vitest';
import { computeRowClues, computeColClues } from '../clueComputeUtils';

describe('computeRowClues', () => {
  it('全空のグリッドは全行空配列を返す', () => {
    const cells = [
      [false, false, false],
      [false, false, false],
    ];
    expect(computeRowClues(cells)).toEqual([[], []]);
  });

  it('全塗りつぶしの行はその長さを返す', () => {
    const cells = [
      [true, true, true],
      [false, false, false],
    ];
    expect(computeRowClues(cells)).toEqual([[3], []]);
  });

  it('連続セルのヒントを正しく計算する', () => {
    const cells = [[true, true, false, true]];
    expect(computeRowClues(cells)).toEqual([[2, 1]]);
  });

  it('飛び飛びセルのヒントを正しく計算する', () => {
    const cells = [[true, false, true, false, true]];
    expect(computeRowClues(cells)).toEqual([[1, 1, 1]]);
  });
});

describe('computeColClues', () => {
  it('全空のグリッドは全列空配列を返す', () => {
    const cells = [
      [false, false],
      [false, false],
    ];
    expect(computeColClues(cells)).toEqual([[], []]);
  });

  it('連続セルの列ヒントを正しく計算する', () => {
    const cells = [
      [true, false],
      [true, false],
      [false, true],
    ];
    expect(computeColClues(cells)).toEqual([[2], [1]]);
  });

  it('飛び飛びセルの列ヒントを正しく計算する', () => {
    const cells = [
      [true],
      [false],
      [true],
    ];
    expect(computeColClues(cells)).toEqual([[1, 1]]);
  });

  it('空のグリッドは空配列を返す', () => {
    expect(computeColClues([])).toEqual([]);
  });
});
