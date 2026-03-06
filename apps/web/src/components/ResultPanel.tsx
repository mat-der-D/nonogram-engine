import type { NonogramStore } from '../hooks/useNonogramStore';
import { PuzzleIOService } from '../services/PuzzleIOService';
import { SolutionGrid } from './SolutionGrid';

interface Props {
  store: NonogramStore;
}

export function ResultPanel({ store }: Props) {
  const { solvePhase } = store;

  const canExport =
    solvePhase.phase === 'done' &&
    (solvePhase.result.status === 'unique' || solvePhase.result.status === 'multiple');

  const handleExportSolution = () => {
    if (solvePhase.phase === 'done') {
      PuzzleIOService.exportSolution(solvePhase.result);
    }
  };

  return (
    <div className="result-panel">
      {solvePhase.phase === 'idle' && (
        <p className="result-hint">パズルを入力して「解く」を押してください</p>
      )}

      {solvePhase.phase === 'solving' && (
        <p className="result-hint result-solving">求解中...</p>
      )}

      {solvePhase.phase === 'done' && solvePhase.result.status === 'none' && (
        <div className="result-card result-none">
          <p className="result-status">✗ 解が存在しないパズルです</p>
          <p className="result-sub">ヒントの組み合わせを確認してください。</p>
        </div>
      )}

      {solvePhase.phase === 'done' && solvePhase.result.status === 'unique' && (
        <div className="result-card result-unique">
          <p className="result-status">✓ 唯一解</p>
          <SolutionGrid grid={solvePhase.result.solutions[0]} label="唯一解" />
        </div>
      )}

      {solvePhase.phase === 'done' && solvePhase.result.status === 'multiple' && (
        <div className="result-card result-multiple">
          <p className="result-status">⚠ 複数の解が存在します（例を2つ表示）</p>
          <div className="solution-grid-row">
            <SolutionGrid grid={solvePhase.result.solutions[0]} label="解例 1" />
            <SolutionGrid grid={solvePhase.result.solutions[1]} label="解例 2" />
          </div>
          <p className="result-sub">唯一解にするにはグリッドを描き直してください。</p>
        </div>
      )}

      {solvePhase.phase === 'done' && solvePhase.result.status === 'error' && (
        <div className="result-card result-error">
          <p className="result-status">✗ 求解エラー</p>
          <p className="result-sub">{solvePhase.result.errorMessage}</p>
        </div>
      )}

      <div className="result-export">
        <button
          className="io-btn"
          onClick={handleExportSolution}
          disabled={!canExport}
        >
          解答エクスポート
        </button>
      </div>
    </div>
  );
}
