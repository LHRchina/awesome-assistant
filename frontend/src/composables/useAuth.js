import { ref, reactive } from 'vue'
import axios from 'axios'

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'https://dochelp.pro/api'

// Global auth state
const authToken = ref(localStorage.getItem('authToken'))
const currentUser = ref(null)
const isAuthenticated = ref(false)
const isLoading = ref(false)
const authError = ref('')

// Configure axios interceptor to include auth token
axios.interceptors.request.use((config) => {
  if (authToken.value) {
    config.headers.Authorization = `Bearer ${authToken.value}`
  }
  return config
})

// Handle 401 responses globally
axios.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      logout()
    }
    return Promise.reject(error)
  }
)

export function useAuth() {
  const initializeGoogleAuth = () => {
    return new Promise((resolve) => {
      if (window.google) {
        resolve()
        return
      }

      const script = document.createElement('script')
      script.src = 'https://accounts.google.com/gsi/client'
      script.async = true
      script.defer = true
      script.onload = () => {
        resolve()
      }
      document.head.appendChild(script)
    })
  }

  const handleCredentialResponse = async (response) => {
    try {
      isLoading.value = true
      authError.value = ''

      const result = await axios.post(`${API_BASE_URL}/login`, {
        google_token: response.credential
      })

      if (result.data.success) {
        authToken.value = result.data.token
        currentUser.value = result.data.user
        isAuthenticated.value = true
        localStorage.setItem('authToken', authToken.value)
      } else {
        authError.value = result.data.message || 'Login failed'
      }
    } catch (error) {
      console.error('Login error:', error)
      authError.value = 'Login failed. Please try again.'
    } finally {
      isLoading.value = false
    }
  }

  const checkAuthStatus = async () => {
    if (!authToken.value) return

    try {
      isLoading.value = true
      const response = await axios.get(`${API_BASE_URL}/me`)
      currentUser.value = response.data
      isAuthenticated.value = true
    } catch (error) {
      console.error('Auth check error:', error)
      logout()
    } finally {
      isLoading.value = false
    }
  }

  const logout = () => {
    authToken.value = null
    currentUser.value = null
    isAuthenticated.value = false
    authError.value = ''
    localStorage.removeItem('authToken')
    // Refresh the page after logout
    window.location.reload()
  }

  const renderGoogleSignIn = (elementId) => {
    if (window.google) {
      window.google.accounts.id.initialize({
        client_id: '859163330769-g2gcui5tun3i6pampo2o1ei8penl5mat.apps.googleusercontent.com',
        callback: handleCredentialResponse
      })

      window.google.accounts.id.renderButton(
        document.getElementById(elementId),
        {
          type: 'standard',
          theme: 'outline',
          size: 'large',
          width: 250
        }
      )
    }
  }

  return {
    // State
    authToken,
    currentUser,
    isAuthenticated,
    isLoading,
    authError,

    // Methods
    initializeGoogleAuth,
    handleCredentialResponse,
    checkAuthStatus,
    logout,
    renderGoogleSignIn
  }
}