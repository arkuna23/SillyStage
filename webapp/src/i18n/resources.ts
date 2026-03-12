import en from './locales/en'
import zhCN from './locales/zh-CN'

export const defaultNS = 'translation'

export const resources = {
  en: {
    translation: en,
  },
  'zh-CN': {
    translation: zhCN,
  },
} as const

