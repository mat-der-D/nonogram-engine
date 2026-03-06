import { describe, it, expect } from 'vitest';
import { parseClueString } from '../clueParseUtils';

describe('parseClueString', () => {
  it('空文字列は空配列を返す', () => {
    expect(parseClueString('')).toEqual({ ok: true, clues: [] });
  });

  it('スペースのみは空配列を返す', () => {
    expect(parseClueString('   ')).toEqual({ ok: true, clues: [] });
  });

  it('単一の整数をパースする', () => {
    expect(parseClueString('3')).toEqual({ ok: true, clues: [3] });
  });

  it('カンマ区切りをパースする', () => {
    expect(parseClueString('1,2,3')).toEqual({ ok: true, clues: [1, 2, 3] });
  });

  it('スペース区切りをパースする', () => {
    expect(parseClueString('1 2 3')).toEqual({ ok: true, clues: [1, 2, 3] });
  });

  it('混在区切りをパースする', () => {
    expect(parseClueString('1, 2  3')).toEqual({ ok: true, clues: [1, 2, 3] });
  });

  it('アルファベットを含む場合はエラーを返す', () => {
    const result = parseClueString('abc');
    expect(result.ok).toBe(false);
  });

  it('小数を含む場合はエラーを返す', () => {
    const result = parseClueString('1.5');
    expect(result.ok).toBe(false);
  });

  it('負数を含む場合はエラーを返す', () => {
    const result = parseClueString('-1');
    expect(result.ok).toBe(false);
  });

  it('ゼロはエラーを返す', () => {
    const result = parseClueString('0');
    expect(result.ok).toBe(false);
  });

  it('混在した不正値はエラーを返す', () => {
    const result = parseClueString('1 a 3');
    expect(result.ok).toBe(false);
  });
});
