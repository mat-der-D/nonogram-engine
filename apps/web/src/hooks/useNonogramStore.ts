import { useReducer, useCallback, useRef } from 'react';
import { useWasm } from '../contexts/WasmContext';
import { parseClueString } from '../utils/clueParseUtils';
import { computeRowClues, computeColClues } from '../utils/clueComputeUtils';

export type InputMode = 'clue' | 'grid';

export type SolvePhase =
  | { phase: 'idle' }
  | { phase: 'solving' }
  | { phase: 'done'; result: SolveResult };

export interface SolveResult {
  status: 'none' | 'unique' | 'multiple' | 'error';
  solutions: boolean[][][];
  errorMessage?: string;
}

export interface PuzzleJson {
  row_clues: number[][];
  col_clues: number[][];
}

interface NonogramState {
  rows: number;
  cols: number;
  rowClueInputs: string[];
  colClueInputs: string[];
  clueErrors: Record<string, string>;
  gridCells: boolean[][];
  inputMode: InputMode;
  solvePhase: SolvePhase;
}

type Action =
  | { type: 'SET_DIMENSIONS'; rows: number; cols: number }
  | { type: 'UPDATE_ROW_CLUE'; index: number; raw: string }
  | { type: 'UPDATE_COL_CLUE'; index: number; raw: string }
  | { type: 'TOGGLE_CELL'; row: number; col: number }
  | { type: 'DRAG_CELL'; row: number; col: number; dragAction: 'fill' | 'erase' }
  | { type: 'SET_INPUT_MODE'; mode: InputMode }
  | { type: 'SET_SOLVE_PHASE'; phase: SolvePhase }
  | { type: 'LOAD_PUZZLE'; puzzle: PuzzleJson };

function makeGrid(rows: number, cols: number): boolean[][] {
  return Array.from({ length: rows }, () => Array(cols).fill(false));
}

function cluesFromGrid(cells: boolean[][], rows: number, cols: number): { rowInputs: string[]; colInputs: string[] } {
  const rowClues = computeRowClues(cells);
  const colClues = computeColClues(cells);
  const rowInputs = Array.from({ length: rows }, (_, i) => (rowClues[i] ?? []).join(' '));
  const colInputs = Array.from({ length: cols }, (_, j) => (colClues[j] ?? []).join(' '));
  return { rowInputs, colInputs };
}

function validateClues(rowClueInputs: string[], colClueInputs: string[]): Record<string, string> {
  const errors: Record<string, string> = {};
  rowClueInputs.forEach((raw, i) => {
    const result = parseClueString(raw);
    if (!result.ok) errors[`row-${i}`] = result.error;
  });
  colClueInputs.forEach((raw, j) => {
    const result = parseClueString(raw);
    if (!result.ok) errors[`col-${j}`] = result.error;
  });
  return errors;
}

function reducer(state: NonogramState, action: Action): NonogramState {
  switch (action.type) {
    case 'SET_DIMENSIONS': {
      const { rows, cols } = action;
      const rowClueInputs = Array.from({ length: rows }, (_, i) => state.rowClueInputs[i] ?? '');
      const colClueInputs = Array.from({ length: cols }, (_, j) => state.colClueInputs[j] ?? '');
      const gridCells = Array.from({ length: rows }, (_, i) =>
        Array.from({ length: cols }, (_, j) => state.gridCells[i]?.[j] ?? false)
      );
      const clueErrors = validateClues(rowClueInputs, colClueInputs);
      return { ...state, rows, cols, rowClueInputs, colClueInputs, gridCells, clueErrors };
    }
    case 'UPDATE_ROW_CLUE': {
      const rowClueInputs = [...state.rowClueInputs];
      rowClueInputs[action.index] = action.raw;
      const clueErrors = validateClues(rowClueInputs, state.colClueInputs);
      return { ...state, rowClueInputs, clueErrors };
    }
    case 'UPDATE_COL_CLUE': {
      const colClueInputs = [...state.colClueInputs];
      colClueInputs[action.index] = action.raw;
      const clueErrors = validateClues(state.rowClueInputs, colClueInputs);
      return { ...state, colClueInputs, clueErrors };
    }
    case 'TOGGLE_CELL': {
      const gridCells = state.gridCells.map((row, r) =>
        row.map((cell, c) => (r === action.row && c === action.col ? !cell : cell))
      );
      const { rowInputs, colInputs } = cluesFromGrid(gridCells, state.rows, state.cols);
      return { ...state, gridCells, rowClueInputs: rowInputs, colClueInputs: colInputs, clueErrors: {} };
    }
    case 'DRAG_CELL': {
      const target = action.dragAction === 'fill';
      const gridCells = state.gridCells.map((row, r) =>
        row.map((cell, c) => (r === action.row && c === action.col ? target : cell))
      );
      const { rowInputs, colInputs } = cluesFromGrid(gridCells, state.rows, state.cols);
      return { ...state, gridCells, rowClueInputs: rowInputs, colClueInputs: colInputs, clueErrors: {} };
    }
    case 'SET_INPUT_MODE':
      return { ...state, inputMode: action.mode };
    case 'SET_SOLVE_PHASE':
      return { ...state, solvePhase: action.phase };
    case 'LOAD_PUZZLE': {
      const { puzzle } = action;
      const rows = puzzle.row_clues.length;
      const cols = puzzle.col_clues.length;
      const rowClueInputs = puzzle.row_clues.map(clues => clues.join(' '));
      const colClueInputs = puzzle.col_clues.map(clues => clues.join(' '));
      const gridCells = makeGrid(rows, cols);
      const clueErrors = validateClues(rowClueInputs, colClueInputs);
      return { ...state, rows, cols, rowClueInputs, colClueInputs, gridCells, clueErrors, solvePhase: { phase: 'idle' } };
    }
    default:
      return state;
  }
}

const DEFAULT_ROWS = 5;
const DEFAULT_COLS = 5;

function makeInitialState(): NonogramState {
  return {
    rows: DEFAULT_ROWS,
    cols: DEFAULT_COLS,
    rowClueInputs: Array(DEFAULT_ROWS).fill(''),
    colClueInputs: Array(DEFAULT_COLS).fill(''),
    clueErrors: {},
    gridCells: makeGrid(DEFAULT_ROWS, DEFAULT_COLS),
    inputMode: 'clue',
    solvePhase: { phase: 'idle' },
  };
}

export interface NonogramStore {
  rows: number;
  cols: number;
  rowClueInputs: string[];
  colClueInputs: string[];
  clueErrors: Record<string, string>;
  gridCells: boolean[][];
  inputMode: InputMode;
  solvePhase: SolvePhase;
  setDimensions(rows: number, cols: number): void;
  updateRowClue(index: number, raw: string): void;
  updateColClue(index: number, raw: string): void;
  toggleCell(row: number, col: number): void;
  startDrag(row: number, col: number, action: 'fill' | 'erase'): void;
  dragCell(row: number, col: number): void;
  endDrag(): void;
  setInputMode(mode: InputMode): void;
  solve(): Promise<void>;
  loadPuzzle(puzzle: PuzzleJson): void;
  getPuzzleJson(): PuzzleJson;
}

export function useNonogramStore(): NonogramStore {
  const [state, dispatch] = useReducer(reducer, undefined, makeInitialState);
  const wasm = useWasm();
  const dragActionRef = useRef<'fill' | 'erase' | null>(null);

  const setDimensions = useCallback((rows: number, cols: number) => {
    dispatch({ type: 'SET_DIMENSIONS', rows, cols });
  }, []);

  const updateRowClue = useCallback((index: number, raw: string) => {
    dispatch({ type: 'UPDATE_ROW_CLUE', index, raw });
  }, []);

  const updateColClue = useCallback((index: number, raw: string) => {
    dispatch({ type: 'UPDATE_COL_CLUE', index, raw });
  }, []);

  const toggleCell = useCallback((row: number, col: number) => {
    dispatch({ type: 'TOGGLE_CELL', row, col });
  }, []);

  const startDrag = useCallback((row: number, col: number, action: 'fill' | 'erase') => {
    dragActionRef.current = action;
    dispatch({ type: 'DRAG_CELL', row, col, dragAction: action });
  }, []);

  const dragCell = useCallback((row: number, col: number) => {
    if (dragActionRef.current !== null) {
      dispatch({ type: 'DRAG_CELL', row, col, dragAction: dragActionRef.current });
    }
  }, []);

  const endDrag = useCallback(() => {
    dragActionRef.current = null;
  }, []);

  const setInputMode = useCallback((mode: InputMode) => {
    dispatch({ type: 'SET_INPUT_MODE', mode });
  }, []);

  const solve = useCallback(async () => {
    dispatch({ type: 'SET_SOLVE_PHASE', phase: { phase: 'solving' } });
    await new Promise(r => setTimeout(r, 0));

    try {
      const rowClues = state.rowClueInputs.map(raw => {
        const result = parseClueString(raw);
        return result.ok ? result.clues : [];
      });
      const colClues = state.colClueInputs.map(raw => {
        const result = parseClueString(raw);
        return result.ok ? result.clues : [];
      });
      const puzzleJson = JSON.stringify({ row_clues: rowClues, col_clues: colClues });
      const resultJson = wasm.solve(puzzleJson);
      const result = JSON.parse(resultJson) as SolveResult;
      dispatch({ type: 'SET_SOLVE_PHASE', phase: { phase: 'done', result } });
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      dispatch({
        type: 'SET_SOLVE_PHASE',
        phase: { phase: 'done', result: { status: 'error', solutions: [], errorMessage: message } },
      });
    }
  }, [state.rowClueInputs, state.colClueInputs, wasm]);

  const loadPuzzle = useCallback((puzzle: PuzzleJson) => {
    dispatch({ type: 'LOAD_PUZZLE', puzzle });
  }, []);

  const getPuzzleJson = useCallback((): PuzzleJson => {
    const row_clues = state.rowClueInputs.map(raw => {
      const result = parseClueString(raw);
      return result.ok ? result.clues : [];
    });
    const col_clues = state.colClueInputs.map(raw => {
      const result = parseClueString(raw);
      return result.ok ? result.clues : [];
    });
    return { row_clues, col_clues };
  }, [state.rowClueInputs, state.colClueInputs]);

  return {
    ...state,
    setDimensions,
    updateRowClue,
    updateColClue,
    toggleCell,
    startDrag,
    dragCell,
    endDrag,
    setInputMode,
    solve,
    loadPuzzle,
    getPuzzleJson,
  };
}

// Exported for unit testing
export { reducer as _reducer, makeInitialState as _makeInitialState };
export type { NonogramState as _NonogramState, Action as _Action };
