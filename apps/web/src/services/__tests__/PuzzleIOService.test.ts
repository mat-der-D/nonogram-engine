import { describe, it, expect } from 'vitest';
import { PuzzleIOService } from '../PuzzleIOService';
import type { PuzzleIOError } from '../PuzzleIOService';

function makeFile(content: string, name = 'test.json'): File {
  return new File([content], name, { type: 'application/json' });
}

describe('PuzzleIOService.importPuzzle', () => {
  it('正常なJSONをインポートできる', async () => {
    const json = JSON.stringify({ row_clues: [[1, 2], [3]], col_clues: [[1], [2], [3]] });
    const result = await PuzzleIOService.importPuzzle(makeFile(json));
    expect(result.row_clues).toEqual([[1, 2], [3]]);
    expect(result.col_clues).toEqual([[1], [2], [3]]);
  });

  it('空のヒント配列を含むJSONをインポートできる', async () => {
    const json = JSON.stringify({ row_clues: [[], []], col_clues: [[], []] });
    const result = await PuzzleIOService.importPuzzle(makeFile(json));
    expect(result.row_clues).toEqual([[], []]);
  });

  it('JSONパースエラー時はparse種別のエラーをスローする', async () => {
    const file = makeFile('not-json');
    await expect(PuzzleIOService.importPuzzle(file)).rejects.toMatchObject({
      kind: 'parse',
    } as Partial<PuzzleIOError>);
  });

  it('row_cluesフィールドがない場合はschema種別のエラーをスローする', async () => {
    const json = JSON.stringify({ col_clues: [[1]] });
    await expect(PuzzleIOService.importPuzzle(makeFile(json))).rejects.toMatchObject({
      kind: 'schema',
    } as Partial<PuzzleIOError>);
  });

  it('col_cluesフィールドがない場合はschema種別のエラーをスローする', async () => {
    const json = JSON.stringify({ row_clues: [[1]] });
    await expect(PuzzleIOService.importPuzzle(makeFile(json))).rejects.toMatchObject({
      kind: 'schema',
    } as Partial<PuzzleIOError>);
  });

  it('row_cluesが配列でない場合はschema種別のエラーをスローする', async () => {
    const json = JSON.stringify({ row_clues: 'invalid', col_clues: [[1]] });
    await expect(PuzzleIOService.importPuzzle(makeFile(json))).rejects.toMatchObject({
      kind: 'schema',
    } as Partial<PuzzleIOError>);
  });
});
