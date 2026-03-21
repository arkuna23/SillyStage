# SillyStage Website

SillyStage 的文档与博客站点，基于 `Rspress` 构建。

## Setup

Install dependencies with `pnpm`:

```bash
pnpm install
```

## Development

Run the docs site locally:

```bash
pnpm dev
```

Run Biome lint checks:

```bash
pnpm lint
```

Format the workspace:

```bash
pnpm format
```

Build for production:

```bash
pnpm build
```

Preview the built site:

```bash
pnpm preview
```

## Deployment

Pushes to `master` that change `website/` trigger GitHub Actions to build and deploy the site to GitHub Pages.
