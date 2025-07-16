export const isLocal = () =>
  window.location.hostname === 'localhost' ||
  window.location.hostname === '127.0.0.1'

export const isDevelopment = () => import.meta.env.DEV
export const isProduction = () => import.meta.env.PROD

export const getApiUrl = () => {
  if (isLocal()) {
    return import.meta.env.VITE_API_BASE_URL || 'https://dochelp.pro/api'
  }
  return 'https://dochelp.pro/api' // Use relative URL for production
}