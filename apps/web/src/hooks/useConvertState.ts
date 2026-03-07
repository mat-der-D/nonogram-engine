import { useState, useCallback, useRef, useEffect } from 'react';
import { useWasm } from '../contexts/WasmContext';

export interface ConvertParams {
  gridWidth: number;
  gridHeight: number;
  smoothStrength: number;
  threshold: number;
  edgeStrength: number;
  noiseRemoval: number;
}

const DEFAULT_PARAMS: ConvertParams = {
  gridWidth: 20,
  gridHeight: 20,
  smoothStrength: 1.0,
  threshold: 128,
  edgeStrength: 0.3,
  noiseRemoval: 0,
};

export interface ConvertStateAndActions {
  params: ConvertParams;
  previewGrid: boolean[][] | null;
  isConverting: boolean;
  imageError: string | null;
  originalPreviewUrl: string | null;
  loadImage(file: File): Promise<void>;
  updateParam<K extends keyof ConvertParams>(key: K, value: ConvertParams[K]): void;
  reset(): void;
}

export function useConvertState(): ConvertStateAndActions {
  const wasm = useWasm();

  const [params, setParams] = useState<ConvertParams>(DEFAULT_PARAMS);
  const [previewGrid, setPreviewGrid] = useState<boolean[][] | null>(null);
  const [isConverting, setIsConverting] = useState(false);
  const [imageError, setImageError] = useState<string | null>(null);
  const [originalPreviewUrl, setOriginalPreviewUrl] = useState<string | null>(null);

  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const resizedBytesRef = useRef<Uint8Array | null>(null);
  const paramsRef = useRef<ConvertParams>(params);
  paramsRef.current = params;
  const originalPreviewUrlRef = useRef<string | null>(null);

  const triggerConvert = useCallback((bytes: Uint8Array, p: ConvertParams) => {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => {
      setIsConverting(true);
      try {
        const resultJson = wasm.image_to_grid(
          bytes,
          p.gridWidth, p.gridHeight,
          p.smoothStrength, p.threshold, p.edgeStrength, p.noiseRemoval,
        );
        const result = JSON.parse(resultJson) as {
          status: string;
          grid?: { rows: number; cols: number; cells: boolean[][] };
          message?: string;
        };
        if (result.status === 'ok' && result.grid) {
          setPreviewGrid(result.grid.cells);
          setImageError(null);
        } else {
          setImageError(result.message ?? '変換エラーが発生しました');
          setPreviewGrid(null);
        }
      } catch (e) {
        setImageError(e instanceof Error ? e.message : '変換エラーが発生しました');
        setPreviewGrid(null);
      } finally {
        setIsConverting(false);
      }
    }, 100);
  }, [wasm]);

  const loadImage = useCallback(async (file: File) => {
    setImageError(null);
    setPreviewGrid(null);

    // Revoke old URL using ref to avoid stale closure
    if (originalPreviewUrlRef.current) URL.revokeObjectURL(originalPreviewUrlRef.current);
    const objectUrl = URL.createObjectURL(file);
    originalPreviewUrlRef.current = objectUrl;
    setOriginalPreviewUrl(objectUrl);

    try {
      const img = new Image();
      await new Promise<void>((resolve, reject) => {
        img.onload = () => resolve();
        img.onerror = () => reject(new Error('画像の読み込みに失敗しました'));
        img.src = objectUrl;
      });

      // Resize to max 384×384, maintaining aspect ratio, no upscale
      const MAX = 384;
      let w = img.naturalWidth;
      let h = img.naturalHeight;
      if (w > MAX || h > MAX) {
        const scale = Math.min(MAX / w, MAX / h);
        w = Math.floor(w * scale);
        h = Math.floor(h * scale);
      }

      const canvas = document.createElement('canvas');
      canvas.width = w;
      canvas.height = h;
      const ctx = canvas.getContext('2d')!;
      ctx.drawImage(img, 0, 0, w, h);

      const bytes = await new Promise<Uint8Array>((resolve, reject) => {
        canvas.toBlob(blob => {
          if (!blob) { reject(new Error('画像のリサイズに失敗しました')); return; }
          blob.arrayBuffer().then(buf => resolve(new Uint8Array(buf)), reject);
        }, 'image/png');
      });

      // Limit grid dims for tiny images (<50px)
      const maxDimW = w < 50 ? w : 50;
      const maxDimH = h < 50 ? h : 50;
      const newParams = {
        ...paramsRef.current,
        gridWidth: Math.min(paramsRef.current.gridWidth, maxDimW),
        gridHeight: Math.min(paramsRef.current.gridHeight, maxDimH),
      };
      setParams(newParams);
      resizedBytesRef.current = bytes;
      triggerConvert(bytes, newParams);
    } catch (e) {
      setImageError(e instanceof Error ? e.message : '画像の読み込みに失敗しました');
      resizedBytesRef.current = null;
    }
  }, [triggerConvert]);

  const updateParam = useCallback(<K extends keyof ConvertParams>(key: K, value: ConvertParams[K]) => {
    const next = { ...paramsRef.current, [key]: value };
    paramsRef.current = next;
    setParams(next);
    if (resizedBytesRef.current) {
      triggerConvert(resizedBytesRef.current, next);
    }
  }, [triggerConvert]);

  const reset = useCallback(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    resizedBytesRef.current = null;
    setParams(DEFAULT_PARAMS);
    paramsRef.current = DEFAULT_PARAMS;
    setPreviewGrid(null);
    setIsConverting(false);
    setImageError(null);
    if (originalPreviewUrlRef.current) URL.revokeObjectURL(originalPreviewUrlRef.current);
    originalPreviewUrlRef.current = null;
    setOriginalPreviewUrl(null);
  }, []);

  useEffect(() => {
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
      if (originalPreviewUrlRef.current) URL.revokeObjectURL(originalPreviewUrlRef.current);
    };
  }, []);

  return {
    params,
    previewGrid,
    isConverting,
    imageError,
    originalPreviewUrl,
    loadImage,
    updateParam,
    reset,
  };
}
