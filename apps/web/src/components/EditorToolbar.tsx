import { useEffect, useRef, useState } from 'react';
import type { MakerStore } from '../hooks/useMakerStore';
import { ExportService } from '../services/ExportService';
import { PuzzleIOService } from '../services/PuzzleIOService';

interface Props {
  store: MakerStore;
}

export function EditorToolbar({ store }: Props) {
  const [widthInput, setWidthInput] = useState(String(store.gridWidth));
  const [heightInput, setHeightInput] = useState(String(store.gridHeight));
  const [exportOpen, setExportOpen] = useState(false);
  const [importOpen, setImportOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const importDropdownRef = useRef<HTMLDivElement>(null);
  const importPuzzleInputRef = useRef<HTMLInputElement>(null);
  const importSolutionInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    setWidthInput(String(store.gridWidth));
  }, [store.gridWidth]);

  useEffect(() => {
    setHeightInput(String(store.gridHeight));
  }, [store.gridHeight]);

  useEffect(() => {
    if (!exportOpen) return;
    const handleClickOutside = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setExportOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [exportOpen]);

  useEffect(() => {
    if (!importOpen) return;
    const handleClickOutside = (e: MouseEvent) => {
      if (importDropdownRef.current && !importDropdownRef.current.contains(e.target as Node)) {
        setImportOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [importOpen]);

  const applyDimensions = () => {
    const w = Math.min(50, Math.max(1, parseInt(widthInput, 10) || store.gridWidth));
    const h = Math.min(50, Math.max(1, parseInt(heightInput, 10) || store.gridHeight));
    store.setDimensions(w, h);
    setWidthInput(String(w));
    setHeightInput(String(h));
  };

  const handleExportJson = () => {
    setExportOpen(false);
    ExportService.exportJson(store.cells);
  };

  const handleExportSolutionJson = () => {
    setExportOpen(false);
    PuzzleIOService.exportSolutionGrid(store.cells);
  };

  const handleExportPng = () => {
    setExportOpen(false);
    void ExportService.exportPng(store.cells, store.rowClues, store.colClues);
  };

  const handleExportPuzzleOnlyPng = () => {
    setExportOpen(false);
    void ExportService.exportPuzzleOnlyPng(store.cells, store.rowClues, store.colClues);
  };

  const handleImportPuzzle = () => {
    setImportOpen(false);
    importPuzzleInputRef.current?.click();
  };

  const handleImportSolution = () => {
    setImportOpen(false);
    importSolutionInputRef.current?.click();
  };

  return (
    <div className="editor-toolbar">
      {/* Canvas settings */}
      <div className="toolbar-group">
        <label className="toolbar-label">
          <span className="toolbar-label-text">幅</span>
          <input
            className="toolbar-num-input"
            type="number"
            min={1}
            max={50}
            value={widthInput}
            onChange={e => setWidthInput(e.target.value)}
            onBlur={applyDimensions}
            onKeyDown={e => { if (e.key === 'Enter') applyDimensions(); }}
          />
        </label>
        <label className="toolbar-label">
          <span className="toolbar-label-text">高</span>
          <input
            className="toolbar-num-input"
            type="number"
            min={1}
            max={50}
            value={heightInput}
            onChange={e => setHeightInput(e.target.value)}
            onBlur={applyDimensions}
            onKeyDown={e => { if (e.key === 'Enter') applyDimensions(); }}
          />
        </label>
        <button
          className="toolbar-btn"
          onClick={() => store.setConvertOpen(true)}
          title="画像から変換"
        >
          <span className="toolbar-icon">🖼</span>
          <span className="toolbar-btn-text"> Convert</span>
        </button>
        <div className="toolbar-dropdown" ref={importDropdownRef}>
          <button
            className="toolbar-btn"
            onClick={() => setImportOpen(v => !v)}
            title="インポート"
          >
            <span className="toolbar-icon">⬆</span>
            <span className="toolbar-btn-text"> Import ▼</span>
          </button>
          {importOpen && (
            <div className="toolbar-dropdown-menu">
              <button className="toolbar-dropdown-item" onClick={handleImportPuzzle}>問題 JSON</button>
              <button className="toolbar-dropdown-item" onClick={handleImportSolution}>解答 JSON</button>
            </div>
          )}
        </div>
      </div>

      <div className="toolbar-divider" />

      {/* Edit history */}
      <div className="toolbar-group">
        <button
          className="toolbar-btn"
          onClick={store.undo}
          disabled={!store.canUndo}
          title="元に戻す (Ctrl+Z)"
        >
          <span className="toolbar-icon">↩</span>
          <span className="toolbar-btn-text"> Undo</span>
        </button>
        <button
          className="toolbar-btn"
          onClick={store.redo}
          disabled={!store.canRedo}
          title="やり直す (Ctrl+Shift+Z)"
        >
          <span className="toolbar-icon">↪</span>
          <span className="toolbar-btn-text"> Redo</span>
        </button>
        <button
          className="toolbar-btn"
          onClick={store.resetGrid}
          title="グリッドをリセット"
        >
          <span className="toolbar-icon">⊗</span>
          <span className="toolbar-btn-text"> リセット</span>
        </button>
      </div>

      <div className="toolbar-divider" />

      {/* Validate and export */}
      <div className="toolbar-group">
        <button
          className="toolbar-btn toolbar-btn-primary"
          onClick={() => void store.solve()}
          disabled={!store.isExportable}
          title="唯一解を検証"
        >
          <span className="toolbar-icon">✓</span>
          <span className="toolbar-btn-text"> 検証</span>
        </button>
        <div className="toolbar-dropdown" ref={dropdownRef}>
          <button
            className="toolbar-btn toolbar-btn-primary"
            onClick={() => setExportOpen(v => !v)}
            disabled={!store.isExportable}
            title="エクスポート"
          >
            <span className="toolbar-icon">⬇</span>
            <span className="toolbar-btn-text"> Export ▼</span>
          </button>
          {exportOpen && store.isExportable && (
            <div className="toolbar-dropdown-menu">
              <button className="toolbar-dropdown-item" onClick={handleExportJson}>問題 JSON</button>
              <button className="toolbar-dropdown-item" onClick={handleExportSolutionJson}>解答 JSON</button>
              <button className="toolbar-dropdown-item" onClick={handleExportPng}>問題+解答 PNG</button>
              <button className="toolbar-dropdown-item" onClick={handleExportPuzzleOnlyPng}>問題のみ PNG</button>
            </div>
          )}
        </div>
      </div>

      {/* Hidden file inputs for import */}
      <input
        ref={importPuzzleInputRef}
        type="file"
        accept=".json,application/json"
        style={{ display: 'none' }}
        onChange={e => {
          const file = e.target.files?.[0];
          if (file) store.importPuzzleJson(file);
          e.target.value = '';
        }}
      />
      <input
        ref={importSolutionInputRef}
        type="file"
        accept=".json,application/json"
        style={{ display: 'none' }}
        onChange={e => {
          const file = e.target.files?.[0];
          if (file) store.importSolutionJson(file);
          e.target.value = '';
        }}
      />
    </div>
  );
}
