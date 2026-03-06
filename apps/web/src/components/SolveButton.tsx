import type { NonogramStore } from '../hooks/useNonogramStore';
import { useWasm } from '../contexts/WasmContext';

interface Props {
  store: NonogramStore;
}

export function SolveButton({ store }: Props) {
  const wasm = useWasm();
  const isSolving = store.solvePhase.phase === 'solving';
  const hasErrors = Object.keys(store.clueErrors).length > 0;
  const isDisabled = isSolving || wasm.status.phase !== 'ready' || hasErrors;

  return (
    <button
      className={`solve-btn${isSolving ? ' solve-btn-loading' : ''}`}
      onClick={() => store.solve()}
      disabled={isDisabled}
    >
      {isSolving ? '求解中...' : '解く'}
    </button>
  );
}
