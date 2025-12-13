# AI DEVELOPMENT METHODOLOGY

This document outlines the strict protocol that AI agents must follow when contributing to this project. Unlike standard coding tasks, this project requires a "Systems Engineering" approach to maintain lightness and stability.

## 1. Interaction Protocol: "Think Before You Code"
Before generating any implementation code, the Agent must:
1.  **Restate the Goal:** Summarize the user's request to ensure understanding.
2.  **Check Constraints:** Verify compatibility with the "Zero-Bloat" philosophy (VISION.md).
3.  **Propose a Plan:** List the files to be created or modified.
4.  **Identify Risks:** Flag potential memory leaks (C) or UI thread blocks (Android).

## 2. Testing Strategy: "Deterministic TDD"
We do not accept code without proof of stability.
*   **Low-Level (C/C++):**
    *   Every logic module must have a corresponding unit test (using a lightweight framework like `minunit` or simple `assert` based mains).
    *   Tests must be **deterministic** (same input = same output). Avoid relying on random system states without seeding.
    *   *Prompt Rule:* "Create the test case for this hash function before implementing the function itself."
*   **UI (Kotlin):**
    *   Since UI is declarative (JSON-based), tests should verify the `JSON -> View` parsing logic, not the Android framework itself.

## 3. Documentation Synchronization: "Living Docs"
Code and documentation must never drift apart.
*   **Definition of Done:** A feature is not complete until:
    1.  The code is written.
    2.  The tests pass.
    3.  `ARCHITECTURE.md` is updated if a new module was added.
    4.  Inline comments explain *why*, not *what*.
*   *Prompt Rule:* "If you change the JSON schema, immediately generate the updated documentation section."

## 4. State Management: "The WIP File"
To handle the limited context window of LLMs, we maintain a `WORKINPROGRESS.md` file.
*   **At the start of a session:** The Agent reads this file to know where we left off.
*   **At the end of a session:** The Agent updates this file with:
    *   Current Task Status.
    *   Next Immediate Steps.
    *   Known Bugs introduced during the session.

## 5. Code Style & Quality
*   **C/C++:** C11 standard. No distinct `malloc` without a paired `free`. Prefer stack allocation for small structs.
*   **Kotlin:** No functional streams for simple loops (performance). No heavy dependency injection.
