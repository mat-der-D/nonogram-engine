import type { MakerStore } from '../hooks/useMakerStore';

interface Props {
  store: MakerStore;
}

export function ImportModal({ store }: Props) {
  if (!store.isImportOpen) return null;

  const { importPhase, setImportOpen, cancelImportSolve } = store;
  const isClosable = importPhase.phase !== 'solving';

  return (
    <div
      className="modal-overlay"
      onClick={isClosable ? () => setImportOpen(false) : undefined}
    >
      <div className="modal-panel" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h2 className="modal-title">インポート</h2>
          <button
            className="modal-close-btn"
            onClick={() => setImportOpen(false)}
            disabled={!isClosable}
          >
            ×
          </button>
        </div>

        <div className="modal-body">
          {importPhase.phase === 'solving' && (
            <div className="solver-solving">
              <div className="spinner" />
              <p>解析中...</p>
              <button className="toolbar-btn" onClick={() => cancelImportSolve()}>
                中止
              </button>
            </div>
          )}

          {importPhase.phase === 'idle' && (
            <p className="solver-message">ファイルを選択してください。</p>
          )}

          {importPhase.phase === 'cancelled' && (
            <p className="solver-message">解析を中止しました。</p>
          )}

          {importPhase.phase === 'done' && (
            <div className="solver-result">
              {importPhase.status === 'success-puzzle' && (
                <div className="solver-badge solver-badge-unique">唯一解が確認されました。グリッドに反映しました。</div>
              )}
              {importPhase.status === 'success-solution' && (
                <div className="solver-badge solver-badge-unique">解答グリッドを反映しました。</div>
              )}
              {importPhase.status === 'non-unique' && (
                <>
                  <div className="solver-badge solver-badge-multiple">インポートできません</div>
                  <p className="solver-message">複数解が存在するため、インポートできません。</p>
                </>
              )}
              {importPhase.status === 'no-solution' && (
                <>
                  <div className="solver-badge solver-badge-none">インポートできません</div>
                  <p className="solver-message">解がありません: インポートできません。</p>
                </>
              )}
              {importPhase.status === 'error' && (
                <>
                  <div className="solver-badge solver-badge-error">エラー</div>
                  <p className="solver-message">
                    {importPhase.message ?? '不明なエラーが発生しました。'}
                  </p>
                </>
              )}
            </div>
          )}
        </div>

        <div className="modal-footer">
          <button
            className="toolbar-btn toolbar-btn-primary"
            onClick={() => setImportOpen(false)}
            disabled={!isClosable}
          >
            閉じる
          </button>
        </div>
      </div>
    </div>
  );
}
