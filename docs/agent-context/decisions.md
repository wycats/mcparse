# Decision Log

This file tracks key architectural and design decisions made throughout the project. It serves as a reference to understand _why_ things are the way they are and prevents re-litigating settled issues without new information.

## Format

### [Date] Title of Decision

**Context**: What was the problem or situation?
**Decision**: What did we decide to do?
**Rationale**: Why did we choose this path? What alternatives were considered?

### [2025-11-25] Expression Continuation Model

**Context**: We needed a way to handle operator precedence and expression parsing that is flexible enough for macros to extend.
**Decision**: Adopted an "Expression Continuation" model where the parser parses a "head" term and then repeatedly checks if the next token (e.g., an operator) wants to "continue" the expression.
**Rationale**: This allows macros to participate in the expression parsing loop dynamically, enabling powerful DSLs and custom operators without a rigid, pre-computed precedence table.

### [2025-11-25] Atomic Whitespace

**Context**: We needed to handle whitespace in a way that supports both "loose" parsing (ignoring whitespace) and "tight" parsing (e.g., method calls `obj.method` vs `obj . method`).
**Decision**: Treat whitespace as a distinct `Atom` during the initial lexing phase, but provide Shape Algebra primitives (`term`, `enter`) that implicitly skip it, while offering `adjacent` for explicit whitespace checks.
**Rationale**: This preserves whitespace information for high-fidelity tools (formatters, highlighters) while keeping the common case of parsing (ignoring whitespace) ergonomic for the grammar writer.
