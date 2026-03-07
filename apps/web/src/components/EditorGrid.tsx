import React, { useEffect, useRef } from 'react';
import type { MakerStore } from '../hooks/useMakerStore';

interface Props {
  store: MakerStore;
}

export function EditorGrid({ store }: Props) {
  const { cells, rowClues, colClues, startDrag, dragCell, endDrag, undo, redo } = store;
  const isDragging = useRef(false);

  useEffect(() => {
    const handleMouseUp = () => {
      if (isDragging.current) {
        endDrag();
        isDragging.current = false;
      }
    };

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.shiftKey && e.key === 'z') {
        e.preventDefault();
        redo();
      } else if (e.ctrlKey && e.key === 'y') {
        e.preventDefault();
        redo();
      } else if (e.ctrlKey && e.key === 'z') {
        e.preventDefault();
        undo();
      }
    };

    window.addEventListener('mouseup', handleMouseUp);
    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('mouseup', handleMouseUp);
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [endDrag, undo, redo]);

  const rows = cells.length;
  const cols = cells[0]?.length ?? 0;

  const maxRowClueLen = Math.ceil(cols / 2);
  const maxColClueLen = Math.ceil(rows / 2);

  return (
    <div className="editor-grid-wrapper">
      <div
        className="editor-grid"
        style={{
          gridTemplateColumns: `calc(var(--cell-size) * ${maxRowClueLen}) repeat(${cols}, var(--cell-size))`,
          gridTemplateRows: `calc(var(--cell-size) * ${maxColClueLen}) repeat(${rows}, var(--cell-size))`,
        }}
      >
        {/* Corner */}
        <div className="editor-corner" />

        {/* Column clues */}
        {colClues.map((clue, c) => (
          <div key={`cc-${c}`} className="editor-col-clue">
            {clue.length > 0
              ? clue.map((n, i) => <span key={i} className="clue-num">{n}</span>)
              : <span className="clue-empty">0</span>
            }
          </div>
        ))}

        {/* Rows: row clue + cells */}
        {cells.map((row, r) => (
          <React.Fragment key={`row-${r}`}>
            <div className="editor-row-clue">
              {rowClues[r] && rowClues[r].length > 0
                ? rowClues[r].map((n, i) => <span key={i} className="clue-num">{n}</span>)
                : <span className="clue-empty">0</span>
              }
            </div>
            {row.map((filled, c) => (
              <div
                key={`cell-${r}-${c}`}
                className={`editor-cell${filled ? ' editor-cell-filled' : ''}`}
                onMouseDown={e => {
                  e.preventDefault();
                  isDragging.current = true;
                  startDrag(r, c, filled ? 'erase' : 'fill');
                }}
                onMouseEnter={() => {
                  if (isDragging.current) {
                    dragCell(r, c);
                  }
                }}
              />
            ))}
          </React.Fragment>
        ))}
      </div>
    </div>
  );
}
