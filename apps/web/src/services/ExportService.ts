import { computeRowClues, computeColClues } from '../utils/clueComputeUtils';

function triggerDownload(url: string, filename: string): void {
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

export const ExportService = {
  exportJson(cells: boolean[][], filename = 'nonogram.json'): void {
    const rowClues = computeRowClues(cells);
    const colClues = computeColClues(cells);
    const json = JSON.stringify({ row_clues: rowClues, col_clues: colClues });
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    triggerDownload(url, filename);
  },

  async exportPuzzleOnlyPng(
    cells: boolean[][],
    rowClues: number[][],
    colClues: number[][],
    filename = 'nonogram-puzzle.png'
  ): Promise<void> {
    const blankCells = cells.map(row => row.map(() => false));
    return this.exportPng(blankCells, rowClues, colClues, filename);
  },

  async exportPng(
    cells: boolean[][],
    rowClues: number[][],
    colClues: number[][],
    filename = 'nonogram.png'
  ): Promise<void> {
    const rows = cells.length;
    const cols = cells[0]?.length ?? 0;
    const CELL_SIZE = Math.max(10, Math.min(30, Math.floor(400 / Math.max(rows, cols, 1))));
    const FONT_SIZE = Math.max(8, Math.floor(CELL_SIZE * 0.6));

    const maxRowClueLen = Math.max(1, ...rowClues.map(c => c.length));
    const maxColClueLen = Math.max(1, ...colClues.map(c => c.length));
    const clueColWidth = maxRowClueLen * CELL_SIZE;
    const clueRowHeight = maxColClueLen * CELL_SIZE;

    const canvas = document.createElement('canvas');
    canvas.width = clueColWidth + cols * CELL_SIZE;
    canvas.height = clueRowHeight + rows * CELL_SIZE;
    const ctx = canvas.getContext('2d')!;

    // Background
    ctx.fillStyle = '#ffffff';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Row clue background
    ctx.fillStyle = '#f1f3f5';
    ctx.fillRect(0, clueRowHeight, clueColWidth, rows * CELL_SIZE);

    // Col clue background
    ctx.fillStyle = '#f1f3f5';
    ctx.fillRect(clueColWidth, 0, cols * CELL_SIZE, clueRowHeight);

    // Corner
    ctx.fillStyle = '#f8f9fa';
    ctx.fillRect(0, 0, clueColWidth, clueRowHeight);

    // Draw cells
    ctx.strokeStyle = '#dee2e6';
    ctx.lineWidth = 0.5;
    for (let r = 0; r < rows; r++) {
      for (let c = 0; c < cols; c++) {
        const x = clueColWidth + c * CELL_SIZE;
        const y = clueRowHeight + r * CELL_SIZE;
        ctx.fillStyle = cells[r][c] ? '#111111' : '#ffffff';
        ctx.fillRect(x, y, CELL_SIZE, CELL_SIZE);
        ctx.strokeRect(x + 0.25, y + 0.25, CELL_SIZE - 0.5, CELL_SIZE - 0.5);
      }
    }

    // Draw row clues
    ctx.fillStyle = '#495057';
    ctx.font = `${FONT_SIZE}px sans-serif`;
    ctx.textAlign = 'right';
    ctx.textBaseline = 'middle';
    for (let r = 0; r < rows; r++) {
      const clue = rowClues[r] ?? [];
      const y = clueRowHeight + r * CELL_SIZE + CELL_SIZE / 2;
      const text = clue.length > 0 ? clue.join(' ') : '0';
      ctx.fillText(text, clueColWidth - 4, y);
    }

    // Draw col clues
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    for (let c = 0; c < cols; c++) {
      const clue = colClues[c] ?? [];
      const x = clueColWidth + c * CELL_SIZE + CELL_SIZE / 2;
      const nums = clue.length > 0 ? clue : [0];
      nums.forEach((n, i) => {
        const y = clueRowHeight - (nums.length - i) * CELL_SIZE + CELL_SIZE / 2;
        ctx.fillText(String(n), x, y);
      });
    }

    // Outer border
    ctx.strokeStyle = '#adb5bd';
    ctx.lineWidth = 1;
    ctx.strokeRect(clueColWidth, clueRowHeight, cols * CELL_SIZE, rows * CELL_SIZE);

    await new Promise<void>(resolve => {
      canvas.toBlob(blob => {
        if (blob) {
          const url = URL.createObjectURL(blob);
          triggerDownload(url, filename);
        }
        resolve();
      }, 'image/png');
    });
  },
};
