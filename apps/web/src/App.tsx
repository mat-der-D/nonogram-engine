import { useWasm, WasmProvider } from './contexts/WasmContext';
import { useNonogramStore } from './hooks/useNonogramStore';
import { PuzzleSizeInput } from './components/PuzzleSizeInput';
import { ModeToggle } from './components/ModeToggle';
import { ClueInputPanel } from './components/ClueInputPanel';
import { GridDrawingPanel } from './components/GridDrawingPanel';
import { SolveButton } from './components/SolveButton';
import { ResultPanel } from './components/ResultPanel';
import { ImportExportPanel } from './components/ImportExportPanel';
import './App.css';

function NonogramApp() {
  const wasm = useWasm();
  const store = useNonogramStore();
  const isDisabled = store.solvePhase.phase === 'solving';

  if (wasm.status.phase === 'loading') {
    return (
      <div className="wasm-loading">
        <div className="spinner" />
        <p>WASM 読み込み中...</p>
      </div>
    );
  }

  return (
    <div className="app-root">
      <header className="app-header">
        <h1 className="app-title">Nonogram Solver</h1>
        {wasm.status.phase === 'error' && (
          <div className="wasm-error-banner">
            WASM初期化エラー: {wasm.status.message}（求解機能は利用できません）
          </div>
        )}
        <ImportExportPanel store={store} />
      </header>

      <main className="app-main">
        <section className="input-panel">
          <PuzzleSizeInput store={store} disabled={isDisabled} />
          <ModeToggle store={store} disabled={isDisabled} />
          <ClueInputPanel store={store} disabled={isDisabled} />
          <GridDrawingPanel store={store} disabled={isDisabled} />
          <div className="solve-area">
            <SolveButton store={store} />
          </div>
        </section>

        <section className="result-panel-section">
          <ResultPanel store={store} />
        </section>
      </main>
    </div>
  );
}

function App() {
  return (
    <WasmProvider>
      <NonogramApp />
    </WasmProvider>
  );
}

export default App;
