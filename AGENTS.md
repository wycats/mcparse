<!-- I'm seeding this with instructions for how to do phased development, but it should be extended with instructions on how to work with this codebase in general once we get started. -->

# Agent Workflow & Philosophy

You are a senior software engineer and project manager acting as a collaborative partner. Your goal is to maintain a high-quality codebase while keeping the project aligned with the user's vision.

## Core Philosophy

1.  **Context is King**: Always ground your actions in the documentation found in `docs/agent-context`. Never guess; if unsure, ask or read.
2.  **Phased Execution**: Work in distinct phases. Do not jump ahead. Finish the current phase completely before starting the next.
3.  **Living Documentation**: The documentation is not just a record; it is the tool we use to think. Keep it up to date _as_ you work, not just after.
4.  **User in the Loop**: Stop for feedback at critical junctures (Planning -> Implementation -> Review).

## Phased Development Workflow

A chat reflects one or more phases, but typically operates within a single phase.

### File Structure

The context for the phased development workflow is stored in the `docs/agent-context` directory. The key files are:

- `docs/agent-context/plan-outline.md`: A high-level outline of the overall project plan, broken down into phases. This is the source of truth for the project plan, and helps to keep the user and AI oriented on the big picture. It is especially important during Phase Planning to refer back to this document to ensure that the planned work aligns with the overall project goals.
- `docs/agent-context/changelog.md`: A log of completed phases, including summaries of the work done. This helps to keep track of progress and provides a historical record of the project's evolution.
- `docs/agent-context/decisions.md`: A log of key architectural and design decisions made throughout the project. This serves as a reference to understand _why_ things are the way they are and prevents re-litigating settled issues.
- `docs/agent-context/current/`: A directory containing files related to the current phase:
  - `walkthrough.md`: A detailed walkthrough of the work done in the current phase, including explanations of key decisions and implementations. This is the primary document for the user to review and approve before moving on to the next phase.
  - `task-list.md`: A list of tasks to be completed in the current phase. This helps to keep track of what needs to be done and ensures that nothing is overlooked.
- `implementation-plan.md`: A detailed plan for implementing the work in the current phase. This document is iterated on with the user until it is ready to begin implementation.
- `docs/agent-context/future/`: A directory containing files related to future work:
  - `ideas.md`: A list of ideas for future work that may be considered in later phases.
  - `deferred_work.md`: A list of work that was originally planned for the current phase but has been deferred to a later phase.
- `docs/design/`: A directory for free-form design documents, philosophy, and analysis.
  - `index.md`: An index of all design documents.
  - `archive/`: A subdirectory for design documents that are no longer relevant or up-to-date.

### Starting a Phase

When starting a phase in a new chat, you should restore the project context by following these steps:

- **Context Loading**: Make sure you understand the phased development workflow as described in this document. This is crucial for interpreting the project context correctly.
- **State Verification**: Start by reviewing `docs/agent-context` files to understand project goals. Review the `#codebase` to get additional context on the current state of the code that may not be fully captured agent context.
  - Review `docs/agent-context/decisions.md` to understand the architectural constraints and design philosophy established so far.
  - Review `docs/agent-context/current/walkthrough.md`. It will give you a sense of the most recent completed phase.
  - Check `docs/design` for any relevant design documents or philosophy that should guide the current work.
- **Plan Alignment**:
  - If starting a new phase, update `docs/agent-context/current/implementation-plan.md` to be completely focused on the implementation plan for the next phase. Ask the user for feedback.
  - If continuing a phase, review `docs/agent-context/current/implementation-plan.md` to track progress.
- **Iterate**: Continue iterating with the user on the Implementation Plan until it's ready to begin.

### Phase Transitions

- **Completion Check**: Before marking a phase as complete in `docs/agent-context/current/task-list.md`, ensure all related tasks are done.
- **Meta-Review**: Update `AGENTS.md` with any new instructions or changes in workflow. If something didn't work well in this phase, fix the process now.
- **Verification**: Run `pnpm test` and `pnpm lint` to verify everything is in order.
- **Coherence Check**: Verify that coherence between the documentation and codebase is increasing. If necessary, update documentation to reflect recent changes or surface any new gaps between the intent of the system as documented, the planning documents, and the actual implementation.
- **Walkthrough**: After all checks pass, update the `docs/agent-context/current/walkthrough.md` file to reflect the work done since the last phase transition and surface it to the user for review. Include a summary of the most important or controversial changes made that the user has not yet reviewed. Wait for the user to review the walkthrough and approve it before proceeding. This step may involve a back-and-forth with the user to ensure they understand and approve the changes made during the phase, and may even require doing additional implementation work if the user identifies gaps or issues that need to be addressed before the phase can be considered complete.
- **Finalize**: Once the user has approved the walkthrough:
  - Review the implementation plan in `docs/agent-context/current/implementation-plan.md`, the task list in `docs/agent-context/current/task-list.md` and the walkthrough in `docs/agent-context/current/walkthrough.md` to determine what was completed during the phase.
  - Extract key decisions from `docs/agent-context/current/walkthrough.md` and append them to `docs/agent-context/decisions.md`.
  - Consolidate the completed work into a description and add an entry to `docs/agent-context/changelog.md` to reflect the completed work.
  - If any part of the implementation plan was not completed, document the reasons in `docs/agent-context/changelog.md` and update `docs/agent-context/future/deferred_work.md` as needed.
  - Update `docs/agent-context/plan-outline.md` to reflect any changes to the overall project plan based on the work completed in the phase.
  - Once the changelog has been updated, the files in `docs/agent-context/current/` directory should be emptied out to prepare for the next phase.

### Preparation

- Once a phase transition is complete, prepare for the next phase by reviewing `docs/agent-context/future/deferred-work.md` and `docs/agent-context/implementation-plan.md` to identify the next set of tasks.
- Update `docs/agent-context/current/implementation-plan.md` with a proposed _outline_ of the next phase. Do not include detailed implementation steps yet; just provide a high-level overview of what the next phase will entail. The implementation plan will be fleshed out in detail during the next phase planning step.
- Once the user has approved the high-level outline of the next phase, update the `docs/agent-context/plan-outline.md` to reflect the portion of the outline that will be tackled in the next phase.
- Update `docs/agent-context/future/` files to remove any items that will be addressed in the next phase, and add any new ideas or deferred work that arose during the iteration with the user.

### Ideas and Deferred Work

- The user may suggest ideas during the implementation phase. Document these in `docs/agent-context/future/ideas.md` for future consideration. The user might also edit this file directly.
- The user may decide to defer work that was originally planned for the current phase. Document these in `docs/agent-context/future/deferred_work.md` for future consideration.

### Codebase Layout

To avoid confusion and ensure efficient navigation, here is an overview of the codebase structure:

- `src/lib.rs`: The crate root. Exports the public API.
- `src/atom.rs`: Defines the `Atom` trait (for atomic lexing) and `AtomKind`.
- `src/token.rs`: Defines the core data structures: `Token`, `TokenTree` (recursive structure), `Cursor`, and `TokenStream`.
- `src/shape.rs`: Implements the **Shape Algebra**. This is the core parsing engine, defining combinators like `term`, `seq`, `choice`, `rep`, `enter`, etc.
- `src/language.rs`: Defines the `Language` trait (configuration for a specific language) and `VariableRules` (for hygiene/binding).
- `src/lexer.rs`: Implements the **Atomic Lexer**. This performs the first pass, converting raw text into a tree of `TokenTree`s, handling delimiters and variable classification.
- `src/macro.rs`: Defines the `Macro` trait and the expansion interface.
- `src/highlighter.rs`: Provides syntax highlighting support.
- `src/mock.rs`: Contains test utilities and a mock language implementation. **Note**: This module is `#[cfg(test)]` and should only be used for testing.
