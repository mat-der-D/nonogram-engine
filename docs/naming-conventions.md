# Naming Conventions: Solver Layer

## Background

This document records the naming decisions for `nonogram-core` solver components.
The decisions were made considering the following principles:

- **Algorithm-descriptive naming**: Names should describe what a component does, not how it is positioned or treated (e.g., `StandardSolver` or `BasicSolver` are anti-patterns).
- **Visibility distinction**: The `...Solver` suffix is reserved exclusively for public types that implement the `Solver` trait. Internal components must not use this suffix.
- **Extensibility**: Names should remain balanced and consistent as new solver algorithms are added.
- **Shared infrastructure awareness**: Constraint propagation (CP) and backtracking are shared internal components used by all complete solvers. Individual solver names should reflect what is unique to each, not the shared infrastructure.

---

## Internal Components (not public API, do not implement `Solver`)

| Name | Role |
|---|---|
| `LinePropagator` | Constraint propagation engine. Computes the intersection of all valid arrangements for a single line (row or column) and iterates to a fixpoint across the entire grid. |
| `Backtracker` | Backtracking search engine. Provides state snapshot/rollback, cell hypothesis, and contradiction detection. Shared by all complete solvers as the final search phase. |

These components do **not** use the `...Solver` suffix because they do not implement the `Solver` trait and are not exposed as public API.

---

## Public Solvers (implement `Solver` trait)

| Name | Algorithm | Distinguishing Feature |
|---|---|---|
| `CspSolver` | CP + Backtracking (CSP approach) | No additional technique beyond the shared infrastructure. "CSP" (Constraint Satisfaction Problem) is the established academic term for the paradigm of constraint propagation combined with backtracking search. |
| `ProbingSolver` | CP + Probing + Backtracking | Adds a probing phase between constraint propagation and backtracking. Based on the LalaFrogKK approach. |
| `DlxSolver` (future) | Dancing Links (Algorithm X) | Encodes the puzzle as an exact cover problem and solves with DLX. |
| `SatSolver` (future) | SAT Encoding | Encodes constraints as a CNF formula and delegates to a SAT solver. |

### Naming Rationale

- **`CspSolver`**: The simplest complete solver combines only the two shared components (CP + backtracking). Since it has no unique technique to name itself after, it is named after the paradigm: CSP (Constraint Satisfaction Problem). This is the formal academic name for the "constraint propagation + backtracking search" combination.
- **`ProbingSolver`**: Named after probing, the technique it adds on top of the shared CP + backtracking infrastructure.
- **`DlxSolver` / `SatSolver`**: Named after their fundamentally different algorithmic approaches (Dancing Links, SAT encoding). These do not share the CP/backtracking infrastructure in the same way, so no `Csp` or `Cp` prefix is needed.

---

## Cell State Enum

The three cell states are named `Unknown`, `Filled`, and `Blank`.

- `Blank` was chosen over `Empty` because `Blank` (white/unfilled) is a more symmetric antonym to `Filled` (black/painted), and `Empty` could be confused with `Unknown` (undetermined).

---

## SolveResult Variants

| Variant | Meaning |
|---|---|
| `NoSolution` | No valid solution exists. |
| `UniqueSolution(Grid)` | Exactly one solution exists. |
| `MultipleSolutions(Vec<Grid>)` | Two or more solutions exist; representative examples are provided. |

- `Vec<Grid>` is used for `MultipleSolutions` instead of a fixed tuple, as a deliberate design decision for forward compatibility.
- All three variant names use consistent `...Solution(s)` grammar. `Solved` was rejected as grammatically inconsistent with the other variants.
