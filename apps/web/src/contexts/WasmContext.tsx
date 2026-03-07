import React, { createContext, useContext, useEffect, useState } from 'react';

export type WasmLoadStatus =
  | { phase: 'loading' }
  | { phase: 'error'; message: string }
  | { phase: 'ready' };

export interface WasmContextValue {
  status: WasmLoadStatus;
  solve: (puzzleJson: string) => string;
  image_to_grid: (
    imageBytes: Uint8Array,
    gridWidth: number,
    gridHeight: number,
    smoothStrength: number,
    threshold: number,
    edgeStrength: number,
    noiseRemoval: number,
  ) => string;
}

const noop = () => JSON.stringify({ status: 'error', message: 'WASM not ready' });

const WasmContext = createContext<WasmContextValue>({
  status: { phase: 'loading' },
  solve: noop,
  image_to_grid: noop,
});

export function WasmProvider({ children }: { children: React.ReactNode }): React.JSX.Element {
  const [status, setStatus] = useState<WasmLoadStatus>({ phase: 'loading' });
  const [solveRef, setSolveRef] = useState<(puzzleJson: string) => string>(() => noop);
  const [imageToGridRef, setImageToGridRef] = useState<(
    imageBytes: Uint8Array,
    gridWidth: number,
    gridHeight: number,
    smoothStrength: number,
    threshold: number,
    edgeStrength: number,
    noiseRemoval: number,
  ) => string>(() => noop);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const wasm = await import('nonogram-wasm');
        if (!cancelled) {
          setSolveRef(() => (puzzleJson: string) => wasm.solve(puzzleJson));
          setImageToGridRef(() => (bytes: Uint8Array, w: number, h: number, smooth: number, thr: number, edge: number, noise: number) =>
            wasm.image_to_grid(bytes, w, h, smooth, thr, edge, noise)
          );
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
    <WasmContext.Provider value={{
      status,
      solve: status.phase === 'ready' ? solveRef : noop,
      image_to_grid: status.phase === 'ready' ? imageToGridRef : noop,
    }}>
      {children}
    </WasmContext.Provider>
  );
}

export function useWasm(): WasmContextValue {
  return useContext(WasmContext);
}
