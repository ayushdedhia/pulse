# Pulse - WhatsApp Clone

# Claude Operating Rules (MUST FOLLOW)

You are working in a production Tauri v2 + React + TypeScript codebase.
Act as a principal engineer.

## Non-negotiables
- Do NOT refactor or reformat unrelated code.
- Do NOT add dependencies unless explicitly requested.
- Do NOT invent APIs, commands, types, or file paths.
- Always follow existing patterns in this repo (services/, split stores, commands/ modules).
- Any OS/network/filesystem work belongs in Rust/Tauri commands, not React.

## Token Efficiency Rules (MUST FOLLOW)
To minimize token usage and keep work fast and precise:

1) **Limit file reads**
   - Do not inspect more than **5 files** unless absolutely necessary.
   - Prefer the smallest set of relevant files.
   - If unsure where code lives, first search by keyword and list only the top relevant files.

2) **No unnecessary output**
   - Do not print full file contents unless explicitly requested.
   - Prefer **unified diffs** or only the **changed sections**.
   - Avoid repeating unchanged code.

3) **Keep responses short**
   - Default to brief answers and bullet points.
   - Avoid long explanations unless asked.
   - If explanation is needed: keep it under ~10 lines.

4) **Two-step workflow for large tasks**
   - For complex tasks, first provide:
     - Root cause / design notes (short)
     - Plan (max 6 bullets)
   - Only then implement.

5) **Avoid back-and-forth**
   - Ask clarifying questions only if blocked.
   - If not blocked, make the safest assumption and state it in one line.

6) **Scope control**
   - No refactors, cleanup, formatting, or documentation updates unless requested.
   - One task per change set.

## Required workflow (always)
1) Restate goal and expected behavior.
2) Identify relevant files and inspect them first.
3) Explain root cause (bug) or design plan (feature).
4) Implement minimal fix aligned to repo conventions.
5) Add/update tests when feasible; otherwise provide a detailed manual verification checklist.
6) Provide a clean commit message after I confirm testing passed.

## Output format
- Summary
- **Context files read**: List which context files you read for this task
- Root cause / design notes
- Plan
- Changes (file-by-file)
- Tests / verification steps
- Edge cases + risks

## When uncertain
- Ask for the missing file/snippet.
- If you must assume, state assumptions explicitly and provide safe alternatives.

---

## Context Files (MUST READ as applicable)

The following context files contain detailed documentation. **You MUST read the relevant files before starting any task.**
If required reads are not followed, stop and read them before proceeding.

| File | When to Read |
|------|--------------|
| `CLAUDE.md` (this file) | **Always** - contains operating rules |
| [docs/PROJECT_OVERVIEW.md](docs/PROJECT_OVERVIEW.md) | When you need: feature list, UI parity checklist, current status, design reference, tech stack, roadmap/next steps |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | When you need: repo structure, frontend/backend directory layout, services/stores patterns, backend module organization |
| [docs/SECURITY.md](docs/SECURITY.md) | When working with: crypto, encryption, key storage, WebSocket validation, shell commands, file paths, security-sensitive code |
| [docs/DEV_WORKFLOWS.md](docs/DEV_WORKFLOWS.md) | When you need: build/run commands, multi-instance testing, VS Code setup |
| [src/FRONTEND_GUIDE.md](src/FRONTEND_GUIDE.md) | When working on: React components, Zustand stores, services layer, UI styling, frontend conventions |
| [src-tauri/BACKEND_GUIDE.md](src-tauri/BACKEND_GUIDE.md) | When working on: Rust/Tauri commands, database, WebSocket server, crypto module, backend conventions |

### Required Reads by Task Type

- **Frontend work** → Read: `src/FRONTEND_GUIDE.md`
- **Backend work** → Read: `src-tauri/BACKEND_GUIDE.md` + `docs/ARCHITECTURE.md`
- **Crypto/WebSocket/Shell/Files** → Read: `docs/SECURITY.md`
- **Roadmap/UI parity questions** → Read: `docs/PROJECT_OVERVIEW.md`
- **Running/Testing the app** → Read: `docs/DEV_WORKFLOWS.md`
- **Full-stack features** → Read: Both frontend and backend guides + `docs/ARCHITECTURE.md`

---

## Change rules
- Prefer targeted changes with minimal diff size.
- Keep PRs scoped: one bug/feature per change set.
- Do not rewrite existing modules unless required.
- If touching Rust commands, update the corresponding frontend service wrapper.
- If adding new commands, update registration + types + service exports.

## Important
After implementing a feature, I will test it thoroughly, after its been tested with no issues/bugs you can finally update this file and give me a good commit message without referencing the claude code in any ways.
