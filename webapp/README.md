# SillyStage Webapp

Frontend scaffold for the future SillyStage UI.

## Stack

- React 19
- TypeScript
- Vite
- Tailwind CSS v4 via the Vite plugin
- Radix UI primitives
- pnpm

## Current Capabilities

- Bilingual UI text with `en` and `zh-CN`
- Browser-language detection with local persistence
- Theater-inspired black-gold theme tokens
- A first reusable UI layer for button, badge, card, input, textarea, select, and section header
- A homepage shell that demonstrates the current transport surfaces and the initial component system
- SPA routing for the current root-mounted frontend

## Commands

From the repository root:

```bash
just frontend
just backend
just dev
just package-linux
just package-windows
```

`just dev` starts `ss-app` on `127.0.0.1:8080` and runs Vite with a development proxy for `/rpc` and `/healthz`, so browser requests no longer hit the Vite server directly.

`just package-linux` and `just package-windows` build the frontend, build `ss-app`, and assemble a self-contained release under `dist/`. The packaged binary auto-discovers the sibling `ss-app.toml` and bundled `webapp/` assets.

Inside `webapp/` directly:

```bash
pnpm install
pnpm dev
pnpm check
pnpm lint
pnpm build
```

## Backend Contract

- `POST /rpc` for request and response style calls
- SSE for streaming turn execution updates
- `GET /healthz` for health checks

## Frontend Pathing

- The current frontend assumes it is served from `/`
- API calls target root-scoped backend endpoints like `/rpc` and `/healthz`
- During `pnpm dev`, Vite proxies `/rpc` and `/healthz` to `http://127.0.0.1:8080`

Treat `ss-protocol` and the API docs under `../docs/zh/api/` and `../docs/en/api/` as the source of truth when the frontend starts consuming backend payloads.
