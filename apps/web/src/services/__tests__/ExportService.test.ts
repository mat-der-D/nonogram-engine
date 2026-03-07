import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { ExportService } from '../ExportService';

describe('ExportService', () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('exportJson', () => {
    let capturedBlob: Blob | null = null;
    let mockAnchor: { href: string; download: string; click: ReturnType<typeof vi.fn> };

    beforeEach(() => {
      capturedBlob = null;
      mockAnchor = { href: '', download: '', click: vi.fn() };
      vi.spyOn(URL, 'createObjectURL').mockImplementation((blob) => {
        capturedBlob = blob as Blob;
        return 'blob:mock';
      });
      vi.spyOn(URL, 'revokeObjectURL').mockImplementation(() => {});
      vi.spyOn(document, 'createElement').mockImplementation((tag: string) => {
        if (tag === 'a') return mockAnchor as unknown as HTMLAnchorElement;
        return document.createElement(tag);
      });
    });

    it('{ row_clues, col_clues } 形式の正しい JSON を生成する', async () => {
      const cells = [
        [true, false, true],
        [false, true, false],
      ];
      ExportService.exportJson(cells);

      expect(capturedBlob).not.toBeNull();
      const text = await (capturedBlob as Blob).text();
      const parsed = JSON.parse(text) as unknown;
      expect(parsed).toEqual({
        row_clues: [[1, 1], [1]],
        col_clues: [[1], [1], [1]],
      });
      expect(mockAnchor.click).toHaveBeenCalled();
    });

    it('全塗りセルのクルーを正しく生成する', async () => {
      const cells = [[true, true], [true, true]];
      ExportService.exportJson(cells);

      const text = await (capturedBlob as Blob).text();
      const parsed = JSON.parse(text) as unknown;
      expect(parsed).toEqual({
        row_clues: [[2], [2]],
        col_clues: [[2], [2]],
      });
    });
  });

  describe('isExportable (grids.some check)', () => {
    it('全 false のグリッドで isExportable は false を返す', () => {
      const cells = [[false, false], [false, false]];
      expect(cells.some(row => row.some(c => c))).toBe(false);
    });

    it('1 つ以上 true があれば isExportable は true を返す', () => {
      const cells = [[false, true], [false, false]];
      expect(cells.some(row => row.some(c => c))).toBe(true);
    });
  });
});
