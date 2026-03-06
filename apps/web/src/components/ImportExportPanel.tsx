import { useRef, useState } from 'react';
import type { NonogramStore } from '../hooks/useNonogramStore';
import { PuzzleIOService } from '../services/PuzzleIOService';
import type { PuzzleIOError } from '../services/PuzzleIOService';

interface Props {
  store: NonogramStore;
}

export function ImportExportPanel({ store }: Props) {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [importError, setImportError] = useState('');

  const handleExportPuzzle = () => {
    const { row_clues, col_clues } = store.getPuzzleJson();
    PuzzleIOService.exportPuzzle(row_clues, col_clues);
  };

  const handleExportTemplate = () => {
    PuzzleIOService.exportTemplate(store.rows, store.cols);
  };

  const handleImport = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    setImportError('');
    try {
      const puzzle = await PuzzleIOService.importPuzzle(file);
      store.loadPuzzle(puzzle);
    } catch (err) {
      const error = err as PuzzleIOError;
      setImportError(error.message ?? 'インポートに失敗しました');
    }
    if (fileInputRef.current) fileInputRef.current.value = '';
  };

  return (
    <div className="io-panel">
      <button className="io-btn" onClick={handleExportPuzzle}>
        問題エクスポート
      </button>
      <button className="io-btn" onClick={handleExportTemplate}>
        テンプレート
      </button>
      <button className="io-btn" onClick={() => fileInputRef.current?.click()}>
        問題インポート
      </button>
      <input
        ref={fileInputRef}
        type="file"
        accept=".json"
        style={{ display: 'none' }}
        onChange={handleImport}
      />
      {importError && <p className="error-msg io-error">{importError}</p>}
    </div>
  );
}
