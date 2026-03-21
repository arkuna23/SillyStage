import * as path from 'node:path'
import { defineConfig } from '@rspress/core'

export default defineConfig({
  root: path.join(__dirname, 'docs'),
  base: process.env.DOCS_BASE_PATH ?? '/',
  title: 'SillyStage',
  description: 'Documentation for the SillyStage Rust interactive storytelling engine.',
  themeConfig: {
    socialLinks: [
      {
        icon: 'github',
        mode: 'link',
        content: 'https://github.com/arkuna23/SillyStage',
      },
    ],
  },
})
