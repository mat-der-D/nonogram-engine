import React, { createContext, useContext, useEffect, useState } from 'react';

export type WasmLoadStatus =
  | { phase: 'loading' }
  | { phase: 'error'; message: string }
  | { phase: 'ready' };

export interface WasmContextValue {
  status: WasmLoadStatus;
  solve: (puzzleJson: string) => string;
}

const noop = () => JSON.stringify({ status: 'error', message: 'WASM not ready' });

const WasmContext = createContext<WasmContextValue>({
  status: { phase: 'loading' },
  solve: noop,
});

export function WasmProvider({ children }: { children: React.ReactNode }): React.JSX.Element {
  const [status, setStatus] = useState<WasmLoadStatus>({ phase: 'loading' });
  const [solveRef, setSolveRef] = useState<(puzzleJson: string) => string>(() => noop);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const wasm = await import('nonogram-wasm');
        await wasm.default();
        if (!cancelled) {
          setSolveRef(() => (puzzleJson: string) => wasm.solve(puzzleJson));
          setStatus({ phase: 'ready' });
        }
      } catch (e) {
        if (!cancelled) {
          const message = e instanceof Error ? e.message : String(e);
          console.error('WASM initialization failed:', e);
          setStatus({ phase: 'error', message });
        }
      }
    })();
    return () => { cancelled = true; };
  }, []);

  return (
    <WasmContext.Provider value={{ status, solve: status.phase === 'ready' ? solveRef : noop }}>
      {children}
    </WasmContext.Provider>
  );
}

export function useWasm(): WasmContextValue {
  return useContext(WasmContext);
}
