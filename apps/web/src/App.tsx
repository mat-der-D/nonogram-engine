import { useWasm, WasmProvider } from './contexts/WasmContext';
import { useMakerStore } from './hooks/useMakerStore';
import { EditorToolbar } from './components/EditorToolbar';
import { EditorGrid } from './components/EditorGrid';
import { SolverModal } from './components/SolverModal';
import { ConvertModal } from './components/ConvertModal';
import './App.css';

function MakerApp() {
  const wasm = useWasm();
  const store = useMakerStore();

  if (wasm.status.phase === 'loading') {
    return (
      <div className="wasm-loading">
        <div className="spinner" />
        <p>読み込み中...</p>
      </div>
    );
  }

  return (
    <div className="app-root">
      <header className="app-header">
        <h1 className="app-title">Nonogram Maker</h1>
        {wasm.status.phase === 'error' && (
          <div className="wasm-error-banner">
            WASM初期化エラー: {wasm.status.message}（求解・変換機能は利用できません）
          </div>
        )}
      </header>
      <EditorToolbar store={store} />
      <main className="maker-main">
        <EditorGrid store={store} />
      </main>
      <SolverModal store={store} />
      {store.isConvertOpen && <ConvertModal store={store} />}
    </div>
  );
}

function App() {
  return (
    <WasmProvider>
      <MakerApp />
    </WasmProvider>
  );
}

export default App;
