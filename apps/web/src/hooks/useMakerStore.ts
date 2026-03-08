import { useReducer, useCallback, useRef, useMemo } from 'react';
import { computeRowClues, computeColClues } from '../utils/clueComputeUtils';
import { PuzzleIOService } from '../services/PuzzleIOService';

export type SolvePhase =
  | { phase: 'idle' }
  | { phase: 'solving' }
  | { phase: 'done'; result: SolveResult }
  | { phase: 'cancelled' };

export interface SolveResult {
  status: 'none' | 'unique' | 'multiple' | 'error';
  solutions: boolean[][][];
  errorMessage?: string;
}

export type ImportPhase =
  | { phase: 'idle' }
  | { phase: 'solving' }
  | { phase: 'done'; status: 'success-puzzle' | 'success-solution' | 'non-unique' | 'no-solution' | 'error'; message?: string }
  | { phase: 'cancelled' };

export interface MakerState {
  gridWidth: number;
  gridHeight: number;
  cells: boolean[][];
  history: boolean[][][][];
  future: boolean[][][][];
  solvePhase: SolvePhase;
  isConvertOpen: boolean;
  isSolverOpen: boolean;
  isImportOpen: boolean;
  importPhase: ImportPhase;
}

export type MakerAction =
  | { type: 'SET_DIMENSIONS'; width: number; height: number }
  | { type: 'TOGGLE_CELL'; row: number; col: number }
  | { type: 'COMMIT_HISTORY' }
  | { type: 'DRAG_CELL'; row: number; col: number; action: 'fill' | 'erase' }
  | { type: 'RESET_GRID' }
  | { type: 'UNDO' }
  | { type: 'REDO' }
  | { type: 'LOAD_GRID'; cells: boolean[][] }
  | { type: 'SET_SOLVE_PHASE'; phase: SolvePhase }
  | { type: 'SET_CONVERT_OPEN'; open: boolean }
  | { type: 'SET_SOLVER_OPEN'; open: boolean }
  | { type: 'SET_IMPORT_OPEN'; open: boolean }
  | { type: 'SET_IMPORT_PHASE'; phase: ImportPhase };

const MAX_HISTORY = 100;

function pushHistory(history: boolean[][][][], snapshot: boolean[][]): boolean[][][][] {
  if (history.length < MAX_HISTORY) {
    return [...history, snapshot];
  }
  return [...history.slice(1), snapshot];
}

function makeGrid(height: number, width: number): boolean[][] {
  return Array.from({ length: height }, () => Array<boolean>(width).fill(false));
}

export function reducer(state: MakerState, action: MakerAction): MakerState {
  switch (action.type) {
    case 'SET_DIMENSIONS': {
      const { width, height } = action;
      const cells = Array.from({ length: height }, (_, r) =>
        Array.from({ length: width }, (_, c) => state.cells[r]?.[c] ?? false)
      );
      return { ...state, gridWidth: width, gridHeight: height, cells };
    }
    case 'TOGGLE_CELL': {
      const cells = state.cells.map((row, r) =>
        r === action.row ? row.map((cell, c) => (c === action.col ? !cell : cell)) : row
      );
      return { ...state, cells };
    }
    case 'COMMIT_HISTORY':
      return { ...state, history: pushHistory(state.history, state.cells), future: [] };
    case 'DRAG_CELL': {
      const target = action.action === 'fill';
      const cells = state.cells.map((row, r) =>
        r === action.row ? row.map((cell, c) => (c === action.col ? target : cell)) : row
      );
      return { ...state, cells };
    }
    case 'RESET_GRID':
      return { ...state, cells: makeGrid(state.gridHeight, state.gridWidth), history: pushHistory(state.history, state.cells), future: [] };
    case 'UNDO': {
      if (state.history.length === 0) return state;
      const history = [...state.history];
      const prev = history.pop()!;
      const future = [state.cells, ...state.future];
      return { ...state, cells: prev, history, future };
    }
    case 'REDO': {
      if (state.future.length === 0) return state;
      const future = [...state.future];
      const next = future.shift()!;
      const history = [...state.history, state.cells];
      return { ...state, cells: next, history, future };
    }
    case 'LOAD_GRID': {
      const gridHeight = action.cells.length;
      const gridWidth = action.cells[0]?.length ?? 0;
      return { ...state, cells: action.cells, gridWidth, gridHeight, history: [], future: [] };
    }
    case 'SET_SOLVE_PHASE':
      return { ...state, solvePhase: action.phase };
    case 'SET_CONVERT_OPEN':
      return { ...state, isConvertOpen: action.open };
    case 'SET_SOLVER_OPEN':
      return { ...state, isSolverOpen: action.open };
    case 'SET_IMPORT_OPEN':
      return { ...state, isImportOpen: action.open };
    case 'SET_IMPORT_PHASE':
      return { ...state, importPhase: action.phase };
    default:
      return state;
  }
}

const DEFAULT_WIDTH = 20;
const DEFAULT_HEIGHT = 20;

export function makeInitialState(): MakerState {
  return {
    gridWidth: DEFAULT_WIDTH,
    gridHeight: DEFAULT_HEIGHT,
    cells: makeGrid(DEFAULT_HEIGHT, DEFAULT_WIDTH),
    history: [],
    future: [],
    solvePhase: { phase: 'idle' },
    isConvertOpen: false,
    isSolverOpen: false,
    isImportOpen: false,
    importPhase: { phase: 'idle' },
  };
}

export interface MakerStore extends MakerState {
  rowClues: number[][];
  colClues: number[][];
  canUndo: boolean;
  canRedo: boolean;
  isExportable: boolean;
  setDimensions(width: number, height: number): void;
  toggleCell(row: number, col: number): void;
  startDrag(row: number, col: number, action: 'fill' | 'erase'): void;
  dragCell(row: number, col: number): void;
  endDrag(): void;
  resetGrid(): void;
  undo(): void;
  redo(): void;
  loadGrid(cells: boolean[][]): void;
  solve(): Promise<void>;
  cancelSolve(): void;
  setConvertOpen(open: boolean): void;
  setSolverOpen(open: boolean): void;
  importPuzzleJson(file: File): void;
  importSolutionJson(file: File): void;
  cancelImportSolve(): void;
  setImportOpen(open: boolean): void;
}

export function useMakerStore(): MakerStore {
  const [state, dispatch] = useReducer(reducer, undefined, makeInitialState);
  const dragActionRef = useRef<'fill' | 'erase' | null>(null);
  const workerRef = useRef<Worker | null>(null);
  const importWorkerRef = useRef<Worker | null>(null);

  const rowClues = useMemo(() => computeRowClues(state.cells), [state.cells]);
  const colClues = useMemo(() => computeColClues(state.cells), [state.cells]);
  const rowCluesRef = useRef(rowClues);
  rowCluesRef.current = rowClues;
  const colCluesRef = useRef(colClues);
  colCluesRef.current = colClues;
  const canUndo = state.history.length > 0;
  const canRedo = state.future.length > 0;
  const isExportable = state.cells.some(row => row.some(c => c));

  const setDimensions = useCallback((width: number, height: number) => {
    dispatch({ type: 'SET_DIMENSIONS', width, height });
  }, []);

  const toggleCell = useCallback((row: number, col: number) => {
    dispatch({ type: 'COMMIT_HISTORY' });
    dispatch({ type: 'TOGGLE_CELL', row, col });
  }, []);

  const startDrag = useCallback((row: number, col: number, action: 'fill' | 'erase') => {
    dispatch({ type: 'COMMIT_HISTORY' });
    dragActionRef.current = action;
    dispatch({ type: 'DRAG_CELL', row, col, action });
  }, []);

  const dragCell = useCallback((row: number, col: number) => {
    if (dragActionRef.current !== null) {
      dispatch({ type: 'DRAG_CELL', row, col, action: dragActionRef.current });
    }
  }, []);

  const endDrag = useCallback(() => {
    dragActionRef.current = null;
  }, []);

  const resetGrid = useCallback(() => {
    dispatch({ type: 'RESET_GRID' });
  }, []);

  const undo = useCallback(() => {
    dispatch({ type: 'UNDO' });
  }, []);

  const redo = useCallback(() => {
    dispatch({ type: 'REDO' });
  }, []);

  const loadGrid = useCallback((cells: boolean[][]) => {
    dispatch({ type: 'LOAD_GRID', cells });
  }, []);

  const solve = useCallback(async () => {
    dispatch({ type: 'SET_SOLVER_OPEN', open: true });
    dispatch({ type: 'SET_SOLVE_PHASE', phase: { phase: 'solving' } });
    await new Promise(r => setTimeout(r, 0));

    const puzzleJson = JSON.stringify({
      row_clues: rowCluesRef.current,
      col_clues: colCluesRef.current,
    });

    const worker = new Worker(
      new URL('../workers/solveWorker.ts', import.meta.url),
      { type: 'module' }
    );
    workerRef.current = worker;

    worker.onmessage = (event) => {
      workerRef.current = null;
      worker.terminate();
      const msg = event.data;
      if (msg.type === 'result') {
        const result = JSON.parse(msg.resultJson) as SolveResult;
        dispatch({ type: 'SET_SOLVE_PHASE', phase: { phase: 'done', result } });
      } else {
        dispatch({ type: 'SET_SOLVE_PHASE', phase: { phase: 'done', result: { status: 'error', solutions: [], errorMessage: msg.message } } });
      }
    };

    worker.onerror = (event) => {
      workerRef.current = null;
      worker.terminate();
      dispatch({ type: 'SET_SOLVE_PHASE', phase: { phase: 'done', result: { status: 'error', solutions: [], errorMessage: event.message ?? 'Worker error' } } });
    };

    worker.postMessage({ type: 'solve', puzzleJson });
  }, []);

  const cancelSolve = useCallback(() => {
    if (workerRef.current) {
      workerRef.current.terminate();
      workerRef.current = null;
    }
    dispatch({ type: 'SET_SOLVE_PHASE', phase: { phase: 'cancelled' } });
  }, []);

  const setConvertOpen = useCallback((open: boolean) => {
    dispatch({ type: 'SET_CONVERT_OPEN', open });
  }, []);

  const setSolverOpen = useCallback((open: boolean) => {
    dispatch({ type: 'SET_SOLVER_OPEN', open });
  }, []);

  const setImportOpen = useCallback((open: boolean) => {
    dispatch({ type: 'SET_IMPORT_OPEN', open });
  }, []);

  const importPuzzleJson = useCallback((file: File) => {
    dispatch({ type: 'SET_IMPORT_OPEN', open: true });
    dispatch({ type: 'SET_IMPORT_PHASE', phase: { phase: 'solving' } });

    PuzzleIOService.importPuzzle(file).then((puzzleJson) => {
      const jsonStr = JSON.stringify({
        row_clues: puzzleJson.row_clues,
        col_clues: puzzleJson.col_clues,
      });

      const worker = new Worker(
        new URL('../workers/solveWorker.ts', import.meta.url),
        { type: 'module' }
      );
      importWorkerRef.current = worker;

      worker.onmessage = (event) => {
        importWorkerRef.current = null;
        worker.terminate();
        const msg = event.data;
        if (msg.type === 'result') {
          const result = JSON.parse(msg.resultJson) as SolveResult;
          if (result.status === 'unique') {
            dispatch({ type: 'LOAD_GRID', cells: result.solutions[0] });
            dispatch({ type: 'SET_IMPORT_PHASE', phase: { phase: 'done', status: 'success-puzzle' } });
          } else if (result.status === 'multiple') {
            dispatch({ type: 'SET_IMPORT_PHASE', phase: { phase: 'done', status: 'non-unique' } });
          } else {
            dispatch({ type: 'SET_IMPORT_PHASE', phase: { phase: 'done', status: 'no-solution' } });
          }
        } else {
          dispatch({ type: 'SET_IMPORT_PHASE', phase: { phase: 'done', status: 'error', message: msg.message ?? '不明なエラーが発生しました' } });
        }
      };

      worker.onerror = (event) => {
        importWorkerRef.current = null;
        worker.terminate();
        dispatch({ type: 'SET_IMPORT_PHASE', phase: { phase: 'done', status: 'error', message: event.message ?? 'Worker error' } });
      };

      worker.postMessage({ type: 'solve', puzzleJson: jsonStr });
    }).catch((err: { message?: string }) => {
      dispatch({ type: 'SET_IMPORT_PHASE', phase: { phase: 'done', status: 'error', message: err?.message ?? 'ファイルの読み込みに失敗しました' } });
    });
  }, []);

  const importSolutionJson = useCallback((file: File) => {
    dispatch({ type: 'SET_IMPORT_OPEN', open: true });

    PuzzleIOService.importSolutionGrid(file).then((cells) => {
      dispatch({ type: 'LOAD_GRID', cells });
      dispatch({ type: 'SET_IMPORT_PHASE', phase: { phase: 'done', status: 'success-solution' } });
    }).catch((err: { message?: string }) => {
      dispatch({ type: 'SET_IMPORT_PHASE', phase: { phase: 'done', status: 'error', message: err?.message ?? 'ファイルの読み込みに失敗しました' } });
    });
  }, []);

  const cancelImportSolve = useCallback(() => {
    if (importWorkerRef.current) {
      importWorkerRef.current.terminate();
      importWorkerRef.current = null;
    }
    dispatch({ type: 'SET_IMPORT_PHASE', phase: { phase: 'cancelled' } });
  }, []);

  return {
    ...state,
    rowClues,
    colClues,
    canUndo,
    canRedo,
    isExportable,
    setDimensions,
    toggleCell,
    startDrag,
    dragCell,
    endDrag,
    resetGrid,
    undo,
    redo,
    loadGrid,
    solve,
    cancelSolve,
    setConvertOpen,
    setSolverOpen,
    importPuzzleJson,
    importSolutionJson,
    cancelImportSolve,
    setImportOpen,
  };
}
