# Repository Construction Plan

## 1. Overview

This repository provides nonogram (paint-by-number puzzle) solver implementations and multiple UI applications.

- **Target puzzle type:** Binary (black & white) nonograms only
- **License:** MIT / Apache-2.0 dual license
- **Development methodology:** cc-sdd (Spec-Driven Development)

---

## 2. Workspace Structure

The repository uses a Cargo workspace with the following layout:

```
nonogram-engine/
├── Cargo.toml                  # workspace root
├── crates/
│   ├── nonogram-core/          # solver logic (no format dependency)
│   ├── nonogram-format/        # JSON format management
│   └── nonogram-wasm/          # WebAssembly bindings
├── apps/
│   ├── cli/                    # CLI application
│   ├── desktop/                # Tauri desktop application
│   └── web/                    # Vite + React web application
└── docs/
```

### Crate Dependency Graph

```
nonogram-core ◄─── nonogram-wasm ◄─── apps/web
              ◄─── apps/cli
              ◄─── apps/desktop

nonogram-format ◄─── nonogram-wasm
                ◄─── apps/cli
                ◄─── apps/desktop
```

`nonogram-core` has zero dependency on `nonogram-format`.
Conversion between the JSON format types and the solver's native types is the responsibility of the application/binding layer.

---

## 3. Solver Layer (`nonogram-core`)

### 3.1 Core Data Types

```rust
/// A single cell state
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Unknown,
    Filled,
    Blank,
}

/// A nonogram puzzle
pub struct Puzzle {
    pub row_clues: Vec<Vec<u32>>,
    pub col_clues: Vec<Vec<u32>>,
}

impl Puzzle {
    pub fn width(&self) -> usize  { self.col_clues.len() }
    pub fn height(&self) -> usize { self.row_clues.len() }
}

/// The grid state during solving
pub struct Grid { ... }

/// The result of solving a puzzle
pub enum SolveResult {
    /// No valid solution exists
    NoSolution,
    /// Exactly one solution exists
    UniqueSolution(Grid),
    /// Two or more solutions exist; at least two examples are provided
    MultipleSolutions(Vec<Grid>),
}
```

### 3.2 Solver Trait

```rust
pub trait Solver {
    fn solve(&self, puzzle: &Puzzle) -> SolveResult;
}
```

All solver implementations implement this trait. New solvers can be added independently.

### 3.3 Implemented Solvers

#### Phase 1: Constraint Propagation (`LinePropagator`) — internal component

- Crate-internal component; does NOT implement the `Solver` trait
- Applies constraint propagation row-by-row and column-by-column using dynamic programming
- Complexity: O(k × l) per line, where k = number of blocks, l = line length
- Iterates until a fixpoint is reached
- Cannot solve all puzzles; may return partial state

#### Phase 2: CSP Solver (`CspSolver`)

- Implements the `Solver` trait; the baseline complete solver
- Uses `LinePropagator` for constraint propagation, then exhaustively explores branches with backtracking via the shared `Backtracker` component
- When constraint propagation reaches a fixpoint with undetermined cells, selects a cell to guess using the MRV (Minimum Remaining Values) heuristic
- Backtracks on contradiction
- Solves the vast majority of well-formed nonograms
- Worst case: exponential; practical performance is good due to strong propagation

Shared internal components (`LinePropagator`, `Backtracker`) provide constraint propagation, state snapshot/rollback, grid cloning, and contradiction detection. These are reused across all solvers.

#### Phase 3: Probing Solver (`ProbingSolver`) — optional, advanced

- Three-phase structure: Line Solving → Probing → Backtracking
- Probing temporarily assumes each undetermined cell is FILLED or BLANK, runs line solving, and uses the result either as a forced deduction or as guidance for backtracking
- Based on the LalaFrogKK algorithm (Wu et al., 2011+), the best-known approach in nonogram solving competitions
- See `docs/solver-algorithms-survey.md` for details

#### Out of scope: Dancing Links, SAT encoding

See `docs/solver-algorithms-survey.md` for rationale.

### 3.4 Quality Standards

- All public APIs documented in American English
- Unit tests for each solver covering: trivial puzzles, typical puzzles, no-solution cases, multiple-solution cases, and edge cases (1×1, empty clues, maximum-density clues)

---

## 4. Format Layer (`nonogram-format`)

### 4.1 Problem Format

```json
{
  "row_clues": [[3], [1, 1], [5], [1, 1], [3]],
  "col_clues": [[3], [1, 1], [5], [1, 1], [3]]
}
```

- `row_clues`: array of clues per row; length determines `height`
- `col_clues`: array of clues per column; length determines `width`
- `width` and `height` fields are intentionally omitted; they are derived from the array lengths

### 4.2 Solution Format

```json
{
  "status": "unique",
  "solutions": [
    [
      [true, true, true, false, false],
      [true, false, true, false, false],
      [true, true, true, true, true],
      [true, false, true, false, false],
      [true, true, true, false, false]
    ]
  ]
}
```

- `status`: `"none"` | `"unique"` | `"multiple"`
- `solutions`: array of grids
  - `"none"`: empty array
  - `"unique"`: one grid
  - `"multiple"`: two or more grids (examples)
- Each grid is a 2D array of booleans: `true` = filled, `false` = blank
- Row-major order: `solutions[s][row][col]`

### 4.3 Problem Template Format

A template is a valid problem JSON with all clues set to empty arrays, used as a starting point for puzzle authoring:

```json
{
  "row_clues": [[], [], [], [], []],
  "col_clues": [[], [], [], [], []]
}
```

---

## 5. UI Layer

### 5.1 Common

- All UIs share the JSON format defined in Section 4
- Frontend language: TypeScript
- Frontend framework: React
- Build tool: Vite

### 5.2 Web Application (`apps/web`)

**Stack:** Vite + React + TypeScript + WebAssembly (`nonogram-wasm`)

**Features:**
- Runs entirely in the browser; no backend required
- Problem import/export (JSON)
- Solution export (JSON)
- Problem input:
  - Enter row/column clues sequentially
  - Paint a grid to auto-generate clues
- Solve and display results:
  - No solution → display message
  - Unique solution → display solution with "unique" indicator
  - Multiple solutions → display two example solutions

**WASM integration:**
`nonogram-wasm` exposes `solve(puzzle_json: &str) -> String` (returns solution JSON).
The React frontend calls this function synchronously via the WASM bindings.

### 5.3 Desktop Application (`apps/desktop`)

**Stack:** Tauri + Vite + React + TypeScript

**Features:** Identical to the web application.

**Architecture:**
- The frontend is the same React codebase as the web app (or shares components)
- `nonogram-core` is called from Tauri's Rust backend via Tauri commands
- Alternatively, the same WASM module used in the web app can be reused in the frontend

### 5.4 CLI Application (`apps/cli`)

**Stack:** Rust, `clap` for argument parsing

**Features:**

```
# Solve a puzzle
nonogram-cli solve --input puzzle.json [--solver csp|probing]

# Generate a problem template
nonogram-cli template --rows <N> --cols <M> --output template.json

# Help
nonogram-cli --help
```

- Default solver: `csp`
- Input: problem JSON file (or stdin)
- Output: solution JSON to stdout

---

## 6. Development Methodology (cc-sdd)

Development follows the cc-sdd (Spec-Driven Development) workflow:

1. **Steering** — Document technical stack, conventions, and architecture as project memory (CLAUDE.md / steering docs)
2. **Requirements** — Define requirements in EARS format (WHEN ... THEN ...) before implementation
3. **Design** — Produce and review a design document; get approval before writing code
4. **Implementation** — Implement with full context; verify all requirements are met

cc-sdd is applied to features that touch multiple files or involve non-trivial logic. Small, isolated changes may skip the full flow.

---

## 7. CI/CD

All automation is implemented via GitHub Actions.

### 7.1 Pull Request Checks (`.github/workflows/ci.yml`)

Triggered on every pull request and push to `main`.

| Job | Description |
|-----|-------------|
| `fmt` | `cargo fmt --check` |
| `clippy` | `cargo clippy -- -D warnings` |
| `test` | `cargo test --workspace` |
| `build-wasm` | `wasm-pack build crates/nonogram-wasm` |
| `build-web` | `cd apps/web && bun install && bun run build` |
| `build-cli` | `cargo build -p nonogram-cli --release` |

### 7.2 Release Automation (`.github/workflows/release.yml`)

Triggered on version tags (`v*`).

| Job | Artifact |
|-----|----------|
| `release-cli-linux` | `nonogram-cli` binary for Linux (x86_64) |
| `release-cli-macos` | `nonogram-cli` binary for macOS (x86_64, arm64) |
| `release-cli-windows` | `nonogram-cli` binary for Windows (x86_64) |
| `release-desktop` | Tauri app bundles (.deb, .dmg, .msi) via `tauri-action` |
| `release-web` | Deploy web app to GitHub Pages |

### 7.3 Code Coverage

- Tool: `cargo-llvm-cov`
- Coverage report uploaded to Codecov on every CI run
- Target: `nonogram-core` unit test coverage ≥ 80%

---

## 8. Decision Log

| # | Topic | Decision |
|---|-------|----------|
| 1 | Solver algorithms (simple) | Line solving (DP, O(k×l) per line) |
| 2 | Solver algorithms (advanced) | CSP approach (`CspSolver`); Probing (`ProbingSolver`) as optional Phase 3 |
| 3 | Nonogram type | Binary (black & white) only |
| 4 | JSON format — problem | `row_clues` + `col_clues` only; width/height derived from array lengths |
| 5 | JSON format — solution | `status` (`none`/`unique`/`multiple`) + `solutions` (0, 1, or 2 grids) |
| 6 | Format crate | Independent crate; `nonogram-core` does NOT depend on it |
| 7 | Web framework | Vite + React + TypeScript |
| 8 | Workspace layout | `crates/` (core, format, wasm) + `apps/` (cli, desktop, web) |
| 9 | Development methodology | cc-sdd (Spec-Driven Development) |
| 10 | CI/CD | GitHub Actions: fmt, clippy, test, WASM build, web build, release automation, coverage |
| 11 | Naming conventions | `...Solver` suffix reserved for public `Solver` trait implementors; internal components use descriptive names (`LinePropagator`, `Backtracker`); see `docs/naming-conventions.md` |
| 12 | Cell states | `Unknown` / `Filled` / `Blank` (`Blank` over `Empty` for symmetry with `Filled`) |
| 13 | SolveResult variants | `NoSolution` / `UniqueSolution(Grid)` / `MultipleSolutions(Vec<Grid>)` |
