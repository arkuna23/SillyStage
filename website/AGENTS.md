# Website Guide

This folder contains the documentation and blog website for the project. It is separate from the application frontend under `webapp/`.

## Purpose

- Use `website/` for docs-site pages, guides, API docs presentation, blog content, and site-level navigation.
- Do not put product application UI here; application features belong in `webapp/`.
- Keep website copy and structure aligned with the backend/docs sources in the repo.

## Tooling

- This site uses `Rspress`.
- Use `pnpm` for dependency management and scripts.
- Do not introduce `npm`/`yarn` commands, lockfiles, or alternative frontend toolchains.

## Content Layout

- Primary content lives under `website/docs/`.
- Shared static assets for the docs site should stay under `website/docs/public/` unless the existing site structure clearly requires another location.
- Keep navigation config in sync with content structure when adding, deleting, or renaming pages.

## Working Rules

- Prefer editing existing docs structure and config instead of inventing parallel content systems.
- Keep docs and blog writing concise, factual, and easy to scan.
- If a task also changes backend APIs or behavior, update the canonical API docs directly under `website/docs/en/api/` and `website/docs/zh/api/`.
- Preserve the existing site framework and conventions unless the task explicitly asks for a website-level refactor.
