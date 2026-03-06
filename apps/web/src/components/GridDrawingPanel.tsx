import React, { useState, useCallback, useRef } from 'react';
import type { NonogramStore } from '../hooks/useNonogramStore';
import { computeRowClues, computeColClues } from '../utils/clueComputeUtils';

interface Props {
  store: NonogramStore;
  disabled?: boolean;
}

interface CellProps {
  filled: boolean;
  dragging: boolean;
  onPointerDown: (e: React.PointerEvent) => void;
  onPointerEnter: () => void;
}

const GridCell = React.memo(function GridCell({ filled, dragging, onPointerDown, onPointerEnter }: CellProps) {
  return (
    <div
      className={`grid-cell${filled ? ' grid-cell-filled' : ''}${dragging ? ' grid-cell-dragging' : ''}`}
      onPointerDown={onPointerDown}
      onPointerEnter={onPointerEnter}
    />
  );
});

export function GridDrawingPanel({ store, disabled }: Props) {
  const [isDragging, setIsDragging] = useState(false);
  const dragActionRef = useRef<'fill' | 'erase' | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  const rowClues = computeRowClues(store.gridCells);
  const colClues = computeColClues(store.gridCells);

  const handlePointerDown = useCallback((row: number, col: number, e: React.PointerEvent) => {
    if (disabled) return;
    e.preventDefault();
    const action = store.gridCells[row]?.[col] ? 'erase' : 'fill';
    dragActionRef.current = action;
    setIsDragging(true);
    store.startDrag(row, col, action);
    (e.target as Element).setPointerCapture(e.pointerId);
  }, [store, disabled]);

  const handlePointerEnter = useCallback((row: number, col: number) => {
    if (!isDragging || disabled) return;
    store.dragCell(row, col);
  }, [isDragging, store, disabled]);

  const handlePointerUp = useCallback(() => {
    setIsDragging(false);
    dragActionRef.current = null;
    store.endDrag();
  }, [store]);

  if (store.inputMode === 'clue') return null;

  const maxColClueLen = Math.max(1, ...colClues.map(c => c.length));
  const maxRowClueLen = Math.max(1, ...rowClues.map(r => r.length));

  void maxColClueLen;
  void maxRowClueLen;

  return (
    <div className="grid-panel" ref={containerRef} onPointerUp={handlePointerUp}>
      <div
        className="grid-board"
        style={{
          display: 'grid',
          gridTemplateColumns: `auto repeat(${store.cols}, 1fr)`,
          gridTemplateRows: `auto repeat(${store.rows}, 1fr)`,
        }}
      >
        {/* Top-left corner */}
        <div className="grid-corner" />

        {/* Column clues (top) */}
        {Array.from({ length: store.cols }, (_, c) => (
          <div key={`col-clue-${c}`} className="col-clue">
            {(colClues[c] ?? []).length === 0
              ? <span className="clue-empty">0</span>
              : (colClues[c] ?? []).map((n, i) => <span key={i} className="clue-num">{n}</span>)}
          </div>
        ))}

        {/* Rows with row clues + cells */}
        {Array.from({ length: store.rows }, (_, r) => (
          <React.Fragment key={`row-${r}`}>
            {/* Row clue */}
            <div className="row-clue">
              {(rowClues[r] ?? []).length === 0
                ? <span className="clue-empty">0</span>
                : (rowClues[r] ?? []).map((n, i) => <span key={i} className="clue-num">{n}</span>)}
            </div>
            {/* Cells */}
            {Array.from({ length: store.cols }, (_, c) => (
              <GridCell
                key={`cell-${r}-${c}`}
                filled={store.gridCells[r]?.[c] ?? false}
                dragging={false}
                onPointerDown={e => handlePointerDown(r, c, e)}
                onPointerEnter={() => handlePointerEnter(r, c)}
              />
            ))}
          </React.Fragment>
        ))}
      </div>
    </div>
  );
}
