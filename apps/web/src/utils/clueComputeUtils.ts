export function computeRowClues(cells: boolean[][]): number[][] {
  return cells.map(row => computeLineClues(row));
}

export function computeColClues(cells: boolean[][]): number[][] {
  if (cells.length === 0) return [];
  const cols = cells[0].length;
  return Array.from({ length: cols }, (_, c) =>
    computeLineClues(cells.map(row => row[c]))
  );
}

function computeLineClues(line: boolean[]): number[] {
  const clues: number[] = [];
  let count = 0;
  for (const cell of line) {
    if (cell) {
      count++;
    } else if (count > 0) {
      clues.push(count);
      count = 0;
    }
  }
  if (count > 0) clues.push(count);
  return clues;
}
