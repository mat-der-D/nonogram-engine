interface Props {
  grid: boolean[][];
  label?: string;
}

const MAX_PX = 240;
const GAP = 1;

export function SolutionGrid({ grid, label }: Props) {
  if (!grid || grid.length === 0) return null;
  const rows = grid.length;
  const cols = grid[0]?.length ?? 0;

  const cellByWidth = Math.floor((MAX_PX - GAP * (cols - 1)) / cols);
  const cellByHeight = Math.floor((MAX_PX - GAP * (rows - 1)) / rows);
  const cellSize = Math.max(2, Math.min(cellByWidth, cellByHeight));

  return (
    <div className="solution-grid-wrapper">
      {label && <p className="solution-label">{label}</p>}
      <div
        className="solution-grid"
        style={{
          gridTemplateColumns: `repeat(${cols}, ${cellSize}px)`,
          gridTemplateRows: `repeat(${rows}, ${cellSize}px)`,
        }}
      >
        {grid.map((row, r) =>
          row.map((filled, c) => (
            <div
              key={`${r}-${c}`}
              className={`solution-cell${filled ? ' solution-cell-filled' : ''}`}
            />
          ))
        )}
      </div>
    </div>
  );
}
