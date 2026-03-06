import { useState } from 'react';
import type { NonogramStore } from '../hooks/useNonogramStore';

interface Props {
  store: NonogramStore;
  disabled?: boolean;
}

export function PuzzleSizeInput({ store, disabled }: Props) {
  const [rows, setRows] = useState(String(store.rows));
  const [cols, setCols] = useState(String(store.cols));
  const [error, setError] = useState('');

  const handleConfirm = () => {
    const r = parseInt(rows, 10);
    const c = parseInt(cols, 10);
    if (isNaN(r) || isNaN(c) || r < 1 || r > 50 || c < 1 || c > 50) {
      setError('行数・列数は1〜50の整数で入力してください');
      return;
    }
    setError('');
    store.setDimensions(r, c);
  };

  return (
    <div className="size-input">
      <label>
        行数
        <input
          type="number" min={1} max={50}
          value={rows}
          onChange={e => setRows(e.target.value)}
          disabled={disabled}
          className="size-field"
        />
      </label>
      <span className="size-x">×</span>
      <label>
        列数
        <input
          type="number" min={1} max={50}
          value={cols}
          onChange={e => setCols(e.target.value)}
          disabled={disabled}
          className="size-field"
        />
      </label>
      <button onClick={handleConfirm} disabled={disabled} className="size-confirm-btn">
        確定
      </button>
      {error && <p className="error-msg">{error}</p>}
    </div>
  );
}
