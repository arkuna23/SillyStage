import * as path from 'node:path'
import { defineConfig } from '@rspress/core'

export default defineConfig({
  root: path.join(__dirname, 'docs'),
  base: process.env.DOCS_BASE_PATH ?? '/',
  lang: 'en',
  locales: [
    {
      lang: 'en',
      label: 'English',
      title: 'SillyStage',
      description: 'SillyStage project documentation',
    },
    {
      lang: 'zh',
      label: '中文',
      title: 'SillyStage',
      description: 'SillyStage 项目文档',
    },
  ],
  title: 'SillyStage',
  description: 'Documentation for the SillyStage Rust interactive storytelling engine.',
  themeConfig: {
    localeRedirect: 'only-default-lang',
    locales: [
      {
        lang: 'en',
        outlineTitle: 'ON THIS PAGE',
      },
      {
        lang: 'zh',
        outlineTitle: '大纲',
      },
    ],
  },
})
