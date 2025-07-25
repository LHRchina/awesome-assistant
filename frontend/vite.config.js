import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  server: {
    port: 5173,
    host: true,
    open: false,
    allowedHosts: ['dochelp.pro', 'localhost', '127.0.0.1'],
    hmr: {
      port: 5173,
      host: 'localhost'
    }
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks: undefined
      }
    }
  },
  define: {
    __VUE_PROD_DEVTOOLS__: false
  }
})