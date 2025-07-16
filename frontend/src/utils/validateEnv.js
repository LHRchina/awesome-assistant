const requiredEnvVars = [
  'VITE_API_BASE_URL'
]

const optionalEnvVars = {
  VITE_APP_TITLE: 'Awesome Assistant',
  VITE_APP_ENV: 'development'
}

export function validateEnvironment() {
  const missing = requiredEnvVars.filter(key => !import.meta.env[key])

  if (missing.length > 0) {
    console.error('Missing required environment variables:', missing)
    throw new Error(`Missing required environment variables: ${missing.join(', ')}`)
  }

  // Set defaults for optional variables
  Object.entries(optionalEnvVars).forEach(([key, defaultValue]) => {
    if (!import.meta.env[key]) {
      console.warn(`Using default value for ${key}: ${defaultValue}`)
    }
  })
}