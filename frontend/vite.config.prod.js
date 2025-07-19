import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  build: {
    rollupOptions: {
      output: {
        manualChunks: undefined
      }
    },
    minify: 'terser',
    sourcemap: false
  },
  define: {
    __VUE_PROD_DEVTOOLS__: false,
    'process.env.NODE_ENV': '"production"'
  },
  // Disable HMR for production
  server: {
    hmr: false
  }
})