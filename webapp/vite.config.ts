import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

const devBackendOrigin = 'http://127.0.0.1:8080'

// https://vite.dev/config/
export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          'routing-motion': ['framer-motion', 'react-router-dom'],
          i18n: ['i18next', 'react-i18next'],
          radix: [
            '@radix-ui/react-dialog',
            '@radix-ui/react-dropdown-menu',
            '@radix-ui/react-select',
            '@radix-ui/react-separator',
            '@radix-ui/react-slot',
            '@radix-ui/react-tabs',
          ],
        },
      },
    },
  },
  server: {
    proxy: {
      '/download': devBackendOrigin,
      '/healthz': devBackendOrigin,
      '/rpc': devBackendOrigin,
      '/upload': devBackendOrigin,
    },
  },
  plugins: [react(), tailwindcss()],
})
