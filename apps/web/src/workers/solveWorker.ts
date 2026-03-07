/// <reference lib="webworker" />
export {};

addEventListener('message', async (event: MessageEvent) => {
  if (event.data.type !== 'solve') return;
  try {
    const wasmModule = await import('nonogram-wasm');
    const resultJson = wasmModule.solve(event.data.puzzleJson as string);
    postMessage({ type: 'result', resultJson });
  } catch (e) {
    const message = e instanceof Error ? e.message : String(e);
    postMessage({ type: 'error', message });
  }
});
