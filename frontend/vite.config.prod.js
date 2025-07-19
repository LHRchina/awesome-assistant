import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  server: {
    allowedHosts: ['dochelp.pro'],
    hmr: false
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks: undefined
      }
    },
    minify: 'esbuild',
    sourcemap: false
  },
  define: {
    __VUE_PROD_DEVTOOLS__: false,
    'process.env.NODE_ENV': '"production"'
  }
})