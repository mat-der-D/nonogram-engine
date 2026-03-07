import { useRef } from 'react';
import type { MakerStore } from '../hooks/useMakerStore';
import { useConvertState } from '../hooks/useConvertState';
import type { ConvertParams } from '../hooks/useConvertState';

interface ConvertModalProps {
  store: MakerStore;
}

// ImageUploader sub-component
function ImageUploader({
  originalPreviewUrl,
  imageError,
  onFileSelect,
}: {
  originalPreviewUrl: string | null;
  imageError: string | null;
  onFileSelect: (file: File) => void;
}) {
  const inputRef = useRef<HTMLInputElement>(null);
  return (
    <div className="convert-preview-item">
      <p className="convert-preview-label">元画像</p>
      <div
        className="image-uploader-area"
        onClick={() => inputRef.current?.click()}
        onDragOver={e => e.preventDefault()}
        onDrop={e => {
          e.preventDefault();
          const file = e.dataTransfer.files[0];
          if (file) onFileSelect(file);
        }}
      >
        {originalPreviewUrl ? (
          <img src={originalPreviewUrl} alt="プレビュー" className="image-preview" />
        ) : (
          <p className="image-uploader-text">クリックまたはドロップで画像を選択<br />(PNG / JPEG / WebP / GIF)</p>
        )}
        <input
          ref={inputRef}
          type="file"
          accept="image/png,image/jpeg,image/webp,image/gif"
          className="image-uploader-input"
          onChange={e => {
            const file = e.target.files?.[0];
            if (file) onFileSelect(file);
            e.target.value = '';
          }}
        />
      </div>
      {imageError && <p className="image-error">{imageError}</p>}
    </div>
  );
}

// PreviewComparison sub-component
function PreviewComparison({ previewGrid }: { previewGrid: boolean[][] | null }) {
  return (
    <div className="convert-preview-item">
      <p className="convert-preview-label">グリッドプレビュー</p>
      {previewGrid ? (
        <div
          className="preview-dot-grid"
          style={(() => {
            const cols = previewGrid[0]?.length ?? 1;
            const rows = previewGrid.length;
            const GAP = 1;
            const MAX_PX = 220;
            const cellByWidth = Math.floor((MAX_PX - GAP * (cols - 1)) / cols);
            const cellByHeight = Math.floor((MAX_PX - GAP * (rows - 1)) / rows);
            const cellSize = Math.max(2, Math.min(cellByWidth, cellByHeight));
            return {
              gridTemplateColumns: `repeat(${cols}, ${cellSize}px)`,
              gridTemplateRows: `repeat(${rows}, ${cellSize}px)`,
            };
          })()}
        >
          {previewGrid.map((row, r) =>
            row.map((filled, c) => (
              <div
                key={`${r}-${c}`}
                className={`preview-dot-cell${filled ? ' preview-dot-cell-filled' : ''}`}
              />
            ))
          )}
        </div>
      ) : (
        <div className="preview-placeholder">画像を選択してください</div>
      )}
    </div>
  );
}

// ParamSliders sub-component
function ParamSliders({
  params,
  isConverting,
  updateParam,
}: {
  params: ConvertParams;
  isConverting: boolean;
  updateParam: <K extends keyof ConvertParams>(key: K, value: ConvertParams[K]) => void;
}) {
  const sliders: Array<{
    key: keyof ConvertParams;
    label: string;
    min: number;
    max: number;
    step: number;
  }> = [
    { key: 'gridWidth', label: 'グリッド幅', min: 5, max: 50, step: 1 },
    { key: 'gridHeight', label: 'グリッド高', min: 5, max: 50, step: 1 },
    { key: 'smoothStrength', label: 'ブラー強度', min: 0, max: 5, step: 0.1 },
    { key: 'threshold', label: 'しきい値', min: 0, max: 255, step: 1 },
    { key: 'edgeStrength', label: 'エッジ強度', min: 0, max: 1, step: 0.05 },
    { key: 'noiseRemoval', label: 'ノイズ除去', min: 0, max: 20, step: 1 },
  ];

  return (
    <div className="param-sliders">
      {isConverting && (
        <div className="convert-spinner-row">
          <div className="convert-spinner" />
          <span>変換中...</span>
        </div>
      )}
      {sliders.map(({ key, label, min, max, step }) => (
        <div key={key} className="param-row">
          <label className="param-label" htmlFor={`param-${key}`}>{label}</label>
          <input
            id={`param-${key}`}
            type="range"
            className="param-slider"
            min={min}
            max={max}
            step={step}
            value={params[key]}
            disabled={isConverting}
            onChange={e => updateParam(key, Number(e.target.value) as ConvertParams[typeof key])}
          />
          <span className="param-value">{params[key]}</span>
        </div>
      ))}
    </div>
  );
}

// Main ConvertModal
export function ConvertModal({ store }: ConvertModalProps) {
  const convertState = useConvertState();
  const { originalPreviewUrl, imageError, previewGrid, isConverting, params, loadImage, updateParam, reset } = convertState;

  const handleApply = () => {
    if (previewGrid) {
      store.loadGrid(previewGrid);
    }
    reset();
    store.setConvertOpen(false);
  };

  const handleClose = () => {
    reset();
    store.setConvertOpen(false);
  };

  return (
    <div className="modal-overlay" onClick={handleClose}>
      <div className="modal-panel" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h2 className="modal-title">画像からグリッドに変換</h2>
          <button className="modal-close-btn" onClick={handleClose}>×</button>
        </div>

        <div className="modal-body convert-body">
          <div className="convert-preview-row">
            <ImageUploader
              originalPreviewUrl={originalPreviewUrl}
              imageError={imageError}
              onFileSelect={loadImage}
            />
            <PreviewComparison previewGrid={previewGrid} />
          </div>
          <ParamSliders params={params} isConverting={isConverting} updateParam={updateParam} />
        </div>

        <div className="modal-footer">
          <button className="toolbar-btn" onClick={handleClose}>キャンセル</button>
          <button
            className="toolbar-btn toolbar-btn-primary"
            onClick={handleApply}
            disabled={!previewGrid}
          >
            適用
          </button>
        </div>
      </div>
    </div>
  );
}
