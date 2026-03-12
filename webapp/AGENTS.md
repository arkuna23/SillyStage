# Webapp Agent Guide

`webapp/` is reserved for the future frontend. Do not initialize the frontend in tasks that only ask for preparation or planning.

## Default Stack

- React
- TypeScript
- Vite
- Tailwind CSS
- Radix UI primitives for lightweight, composable UI building blocks
- `pnpm` for package management

If a later task asks to initialize the frontend and does not override these choices, use this stack.

## Frontend Principles

- Keep the frontend separate from the Rust backend crates.
- Treat `ss-protocol` and the documentation under `docs/` as the source of truth for backend communication and product flow.
- Before implementing frontend features, check `docs/` first instead of relying on hardcoded per-file guidance here.
- Use the existing HTTP transport shape:
  - `POST /rpc` for request/response
  - SSE for streaming responses such as turn execution
- Do not invent a second API layer or frontend-only wire format unless explicitly requested.

## UI Direction

- Prefer modern, lightweight, composable primitives over heavy all-in-one admin UI kits.
- Tailwind should be the default styling layer.
- Build local components on top of primitives instead of depending on a rigid template system.
- Start with small, purpose-built state management. Do not introduce a heavy global state library by default.
- Use `pnpm` for dependency installation, script execution, and lockfile management.
- Do not mix package managers. Do not add `package-lock.json` or `yarn.lock`.

## Copy Style

- When writing Simplified Chinese UI copy, default to natural product Chinese rather than literal translation from English.
- Keep a light stage / roleplay tone when it helps the product voice, but do not let copy become theatrical at the expense of clarity.
- Prefer mixed Chinese/English terminology for established technical terms such as `RPC`, `SSE`, `prompt`, and `Radix` when that reads more naturally.
- Avoid translationese, direct English sentence structure, and half-translated engineering jargon in user-facing Chinese copy.

## Current Constraint

- This folder is intentionally empty except for this guide.
- Do not add `package.json`, Vite config, Tailwind config, or source files until a later task explicitly asks for frontend initialization.
