interface Props {
  grid: boolean[][];
  label?: string;
}

export function SolutionGrid({ grid, label }: Props) {
  if (!grid || grid.length === 0) return null;
  const cols = grid[0]?.length ?? 0;

  return (
    <div className="solution-grid-wrapper">
      {label && <p className="solution-label">{label}</p>}
      <div
        className="solution-grid"
        style={{ gridTemplateColumns: `repeat(${cols}, 1fr)` }}
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
