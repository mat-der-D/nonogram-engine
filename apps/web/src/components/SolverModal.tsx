import type { MakerStore } from '../hooks/useMakerStore';
import { SolutionGrid } from './SolutionGrid';

interface Props {
  store: MakerStore;
}

export function SolverModal({ store }: Props) {
  if (!store.isSolverOpen) return null;

  const { solvePhase, setSolverOpen } = store;
  const isClosable = solvePhase.phase !== 'solving';

  return (
    <div
      className="modal-overlay"
      onClick={isClosable ? () => setSolverOpen(false) : undefined}
    >
      <div className="modal-panel" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h2 className="modal-title">検証結果</h2>
          <button
            className="modal-close-btn"
            onClick={() => setSolverOpen(false)}
            disabled={!isClosable}
          >
            ×
          </button>
        </div>

        <div className="modal-body">
          {solvePhase.phase === 'solving' && (
            <div className="solver-solving">
              <div className="spinner" />
              <p>解析中...</p>
            </div>
          )}

          {solvePhase.phase === 'idle' && (
            <p className="solver-message">検証を実行してください。</p>
          )}

          {solvePhase.phase === 'done' && (
            <div className="solver-result">
              {solvePhase.result.status === 'unique' && (
                <>
                  <div className="solver-badge solver-badge-unique">唯一解</div>
                  <div className="solver-grids">
                    <SolutionGrid grid={solvePhase.result.solutions[0]} />
                  </div>
                </>
              )}
              {solvePhase.result.status === 'multiple' && (
                <>
                  <div className="solver-badge solver-badge-multiple">複数解</div>
                  <div className="solver-grids">
                    {solvePhase.result.solutions.slice(0, 2).map((grid, i) => (
                      <SolutionGrid key={i} grid={grid} label={`解 ${i + 1}`} />
                    ))}
                  </div>
                </>
              )}
              {solvePhase.result.status === 'none' && (
                <>
                  <div className="solver-badge solver-badge-none">解なし</div>
                  <p className="solver-message">この問題には解がありません。</p>
                </>
              )}
              {solvePhase.result.status === 'error' && (
                <>
                  <div className="solver-badge solver-badge-error">エラー</div>
                  <p className="solver-message">
                    {solvePhase.result.errorMessage ?? '不明なエラーが発生しました。'}
                  </p>
                </>
              )}
            </div>
          )}
        </div>

        <div className="modal-footer">
          <button
            className="toolbar-btn toolbar-btn-primary"
            onClick={() => setSolverOpen(false)}
            disabled={!isClosable}
          >
            閉じる
          </button>
        </div>
      </div>
    </div>
  );
}
