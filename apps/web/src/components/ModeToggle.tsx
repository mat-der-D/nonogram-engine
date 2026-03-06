import type { NonogramStore } from '../hooks/useNonogramStore';

interface Props {
  store: NonogramStore;
  disabled?: boolean;
}

export function ModeToggle({ store, disabled }: Props) {
  return (
    <div className="mode-toggle">
      <button
        className={`mode-btn${store.inputMode === 'clue' ? ' mode-btn-active' : ''}`}
        onClick={() => store.setInputMode('clue')}
        disabled={disabled}
      >
        ヒント入力
      </button>
      <button
        className={`mode-btn${store.inputMode === 'grid' ? ' mode-btn-active' : ''}`}
        onClick={() => store.setInputMode('grid')}
        disabled={disabled}
      >
        グリッド描画
      </button>
    </div>
  );
}
