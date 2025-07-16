/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_API_BASE_URL: string
  readonly VITE_APP_TITLE: string
  readonly VITE_APP_ENV: 'development' | 'production' | 'test'
  readonly VITE_ENABLE_GOOGLE_AUTH: string
  readonly VITE_ENABLE_FILE_UPLOAD: string
  readonly VITE_ENABLE_ANALYTICS: string
  readonly VITE_API_TIMEOUT: string
  readonly VITE_API_RETRIES: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}