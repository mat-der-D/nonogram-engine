import type { NonogramStore } from '../hooks/useNonogramStore';

interface Props {
  store: NonogramStore;
  disabled?: boolean;
}

export function ClueInputPanel({ store, disabled }: Props) {
  if (store.inputMode === 'grid') return null;

  return (
    <div className="clue-panel">
      <div className="clue-section">
        <h3 className="clue-section-title">行ヒント</h3>
        {store.rowClueInputs.map((val, i) => (
          <div key={i} className="clue-row">
            <label className="clue-label">行 {i + 1}</label>
            <input
              type="text"
              value={val}
              onChange={e => store.updateRowClue(i, e.target.value)}
              disabled={disabled}
              placeholder="例: 1 2 3"
              className={`clue-input${store.clueErrors[`row-${i}`] ? ' clue-input-error' : ''}`}
            />
            {store.clueErrors[`row-${i}`] && (
              <p className="error-msg">{store.clueErrors[`row-${i}`]}</p>
            )}
          </div>
        ))}
      </div>
      <div className="clue-section">
        <h3 className="clue-section-title">列ヒント</h3>
        {store.colClueInputs.map((val, j) => (
          <div key={j} className="clue-row">
            <label className="clue-label">列 {j + 1}</label>
            <input
              type="text"
              value={val}
              onChange={e => store.updateColClue(j, e.target.value)}
              disabled={disabled}
              placeholder="例: 1 2 3"
              className={`clue-input${store.clueErrors[`col-${j}`] ? ' clue-input-error' : ''}`}
            />
            {store.clueErrors[`col-${j}`] && (
              <p className="error-msg">{store.clueErrors[`col-${j}`]}</p>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
