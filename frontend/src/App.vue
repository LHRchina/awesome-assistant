<template>
  <div id="app">
    <div class="container">
      <div class="header">
        <h1>🚀 Awesome Assistant</h1>
        <p>Secure File Upload with Google Authentication</p>
      </div>

      <!-- Login Section -->
      <div v-if="!isAuthenticated" class="login-section">
        <h2>Login Required</h2>
        <p>Please sign in with your Google account to upload and manage files.</p>
        <div id="google-signin-button"></div>
        <div v-if="authError" class="error">{{ authError }}</div>
        <div v-if="isLoading" class="loading">Signing in...</div>
      </div>

      <!-- Main App Section -->
      <div v-else class="app-section">
        <div class="user-info">
          <h3>Welcome, {{ currentUser?.name }}!</h3>
          <p>Email: {{ currentUser?.email }}</p>
          <button @click="logout" class="btn btn-danger">Logout</button>
        </div>

        <!-- Upload Section -->
        <div class="upload-section">
          <h2>Upload Files</h2>
          <div class="upload-area" @drop="handleDrop" @dragover.prevent @dragenter.prevent">
            <input
              type="file"
              ref="fileInput"
              @change="handleFileSelect"
              multiple
              class="file-input"
            />
            <div class="upload-content">
              <svg class="upload-icon" viewBox="0 0 24 24" width="48" height="48">
                <path fill="currentColor" d="M14,2H6A2,2 0 0,0 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V8L14,2M18,20H6V4H13V9H18V20Z" />
              </svg>
              <p>Drag and drop files here or click to select</p>
              <button @click="$refs.fileInput.click()" class="select-btn">Select Files</button>
            </div>
          </div>

          <!-- Selected Files -->
          <div v-if="selectedFiles.length > 0" class="selected-files">
            <h3>Selected Files:</h3>
            <div v-for="(file, index) in selectedFiles" :key="index" class="file-item">
              <span>{{ file.name }} ({{ formatFileSize(file.size) }})</span>
              <button @click="removeFile(index)" class="remove-btn">×</button>
            </div>
            <button @click="uploadFiles" :disabled="uploading" class="upload-btn">
              {{ uploading ? 'Uploading...' : 'Upload Files' }}
            </button>
          </div>

          <!-- Upload Progress -->
          <div v-if="uploading" class="progress-bar">
            <div class="progress-fill" :style="{ width: uploadProgress + '%' }"></div>
          </div>

          <!-- Upload Status -->
          <div v-if="uploadStatus" class="upload-status" :class="uploadStatusType">
            {{ uploadStatus }}
          </div>
        </div>

        <!-- Files List Section -->
        <div class="files-section">
          <h2>Your Files</h2>
          <button @click="loadFiles" class="refresh-btn">Refresh Files</button>

          <div v-if="loading" class="loading">Loading files...</div>

          <div v-else-if="uploadedFiles.length === 0" class="no-files">
            No files uploaded yet.
          </div>

          <div v-else class="files-grid">
            <div v-for="file in uploadedFiles" :key="file.id" class="file-card">
              <div class="file-info">
                <h4>{{ file.filename }}</h4>
                <p>Size: {{ formatFileSize(file.size) }}</p>
                <p>Type: {{ file.content_type || 'Unknown' }}</p>
                <p>Uploaded: {{ formatDate(file.upload_time) }}</p>
              </div>
              <div class="file-actions">
                <button @click="downloadFile(file.id, file.filename)" class="download-btn">
                  Download
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, onMounted, nextTick } from 'vue'
import axios from 'axios'
import { useAuth } from './composables/useAuth.js'

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'https://dochelp.pro/api'

export default {
  name: 'App',
  setup() {
    const {
      authToken,
      currentUser,
      isAuthenticated,
      isLoading,
      authError,
      initializeGoogleAuth,
      checkAuthStatus,
      logout,
      renderGoogleSignIn
    } = useAuth()

    const selectedFiles = ref([])
    const uploadedFiles = ref([])
    const uploading = ref(false)
    const loading = ref(false)
    const uploadProgress = ref(0)
    const uploadStatus = ref('')
    const uploadStatusType = ref('')

    onMounted(async () => {
      await initializeGoogleAuth()

      if (authToken.value) {
        await checkAuthStatus()
      }

      if (isAuthenticated.value) {
        loadFiles()
      }

      // Render Google Sign-In button after component is mounted
      nextTick(() => {
        if (!isAuthenticated.value) {
          renderGoogleSignIn('google-signin-button')
        }
      })
    })
  const handleFileSelect = (event) => {
      const files = Array.from(event.target.files)
      selectedFiles.value = [...selectedFiles.value, ...files]
    }

    const handleDrop = (event) => {
      event.preventDefault()
      const files = Array.from(event.dataTransfer.files)
      selectedFiles.value = [...selectedFiles.value, ...files]
    }

    const removeFile = (index) => {
      selectedFiles.value.splice(index, 1)
    }

    const uploadFiles = async () => {
      if (selectedFiles.value.length === 0) return

      uploading.value = true
      uploadProgress.value = 0
      uploadStatus.value = ''

      try {
        for (let i = 0; i < selectedFiles.value.length; i++) {
          const file = selectedFiles.value[i]
          const formData = new FormData()
          formData.append('file', file)

          await axios.post(`${API_BASE_URL}/upload`, formData, {
            headers: {
              'Content-Type': 'multipart/form-data'
            },
            onUploadProgress: (progressEvent) => {
              const progress = Math.round(
                ((i + progressEvent.loaded / progressEvent.total) / selectedFiles.value.length) * 100
              )
              uploadProgress.value = progress
            }
          })
        }

        selectedFiles.value = []
        uploadStatus.value = 'Files uploaded successfully!'
        uploadStatusType.value = 'success'
        loadFiles()
      } catch (error) {
        console.error('Upload error:', error)
        uploadStatus.value = error.response?.data?.message || 'Error uploading files. Please try again.'
        uploadStatusType.value = 'error'
      } finally {
        uploading.value = false
        uploadProgress.value = 0
        setTimeout(() => {
          uploadStatus.value = ''
        }, 5000)
      }
    }

    const loadFiles = async () => {
      if (!isAuthenticated.value) return

      loading.value = true
      try {
        const response = await axios.get(`${API_BASE_URL}/files`)
        uploadedFiles.value = response.data.files || []
      } catch (error) {
        console.error('Error loading files:', error)
        uploadStatus.value = 'Error loading files. Please try again.'
        uploadStatusType.value = 'error'
      } finally {
        loading.value = false
      }
    }

    const downloadFile = async (fileId, filename) => {
      try {
        const response = await axios.get(`${API_BASE_URL}/download/${fileId}`, {
          responseType: 'blob'
        })

        const url = window.URL.createObjectURL(new Blob([response.data]))
        const link = document.createElement('a')
        link.href = url
        link.setAttribute('download', filename)
        document.body.appendChild(link)
        link.click()
        link.remove()
        window.URL.revokeObjectURL(url)
      } catch (error) {
        console.error('Download error:', error)
        uploadStatus.value = 'Error downloading file.'
        uploadStatusType.value = 'error'
      }
    }

    const formatFileSize = (bytes) => {
      if (bytes === 0) return '0 Bytes'
      const k = 1024
      const sizes = ['Bytes', 'KB', 'MB', 'GB']
      const i = Math.floor(Math.log(bytes) / Math.log(k))
      return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
    }

    const formatDate = (dateString) => {
      return new Date(dateString).toLocaleString()
    }

    return {
      // Auth state
      authToken,
      currentUser,
      isAuthenticated,
      isLoading,
      authError,
      logout,

      // App state
      selectedFiles,
      uploadedFiles,
      uploading,
      loading,
      uploadProgress,
      uploadStatus,
      uploadStatusType,

      // Methods
      handleFileSelect,
      handleDrop,
      removeFile,
      uploadFiles,
      loadFiles,
      downloadFile,
      formatFileSize,
      formatDate
    }
  }
}
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background-color: #f5f5f5;
}

#app {
  max-width: 800px;
  margin: 0 auto;
  padding: 20px;
}

.container {
  background: white;
  padding: 30px;
  border-radius: 10px;
  box-shadow: 0 2px 10px rgba(0,0,0,0.1);
}

.header {
  text-align: center;
  margin-bottom: 30px;
}

.header h1 {
  color: #333;
  margin-bottom: 10px;
  font-size: 2.5rem;
}

.header p {
  color: #666;
  font-size: 16px;
}

/* Authentication Styles */
.login-section {
  margin: 20px 0;
  padding: 20px;
  border: 1px solid #ddd;
  border-radius: 8px;
  text-align: center;
}

.login-section h2 {
  margin-bottom: 15px;
  color: #333;
  font-size: 1.8rem;
}

.login-section p {
  margin-bottom: 20px;
  color: #666;
}

#google-signin-button {
  display: flex;
  justify-content: center;
  margin: 20px 0;
}

.user-info {
  background: #e7f3ff;
  padding: 15px;
  border-radius: 5px;
  margin-bottom: 20px;
}

.user-info h3 {
  margin-bottom: 5px;
  color: #333;
}

.user-info p {
  margin-bottom: 10px;
  color: #666;
}

/* Button Styles */
.btn {
  background: #007bff;
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 5px;
  cursor: pointer;
  margin: 5px;
  font-size: 14px;
  transition: background-color 0.2s;
}

.btn:hover {
  background: #0056b3;
}

.btn:disabled {
  background: #ccc;
  cursor: not-allowed;
}

.btn-danger {
  background: #dc3545;
}

.btn-danger:hover {
  background: #c82333;
}

/* Status Messages */
.error {
  color: #dc3545;
  margin: 10px 0;
  padding: 10px;
  background: #f8d7da;
  border: 1px solid #f5c6cb;
  border-radius: 4px;
}

.success {
  color: #155724;
  margin: 10px 0;
  padding: 10px;
  background: #d4edda;
  border: 1px solid #c3e6cb;
  border-radius: 4px;
}

.loading {
  color: #007bff;
  margin: 10px 0;
  text-align: center;
  font-size: 1.1rem;
  padding: 20px;
}

.upload-status {
  margin: 15px 0;
  padding: 10px;
  border-radius: 4px;
  text-align: center;
}

.upload-status.success {
  color: #155724;
  background: #d4edda;
  border: 1px solid #c3e6cb;
}

.upload-status.error {
  color: #721c24;
  background: #f8d7da;
  border: 1px solid #f5c6cb;
}

/* Upload Section */
.upload-section, .files-section {
  margin: 20px 0;
  padding: 20px;
  border: 1px solid #ddd;
  border-radius: 8px;
}

.upload-section h2, .files-section h2 {
  color: #444;
  margin-bottom: 20px;
  font-size: 1.8rem;
}

.upload-area {
  border: 3px dashed #ddd;
  border-radius: 12px;
  padding: 40px;
  text-align: center;
  transition: all 0.3s ease;
  cursor: pointer;
  position: relative;
}

.upload-area:hover {
  border-color: #007bff;
  background-color: #f8f9fa;
}

.file-input {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  opacity: 0;
  cursor: pointer;
}

.upload-content {
  pointer-events: none;
}

.upload-icon {
  color: #007bff;
  margin-bottom: 15px;
}

.upload-content p {
  font-size: 1.1rem;
  color: #666;
  margin-bottom: 15px;
}

.select-btn {
  background: #007bff;
  color: white;
  border: none;
  padding: 12px 24px;
  border-radius: 6px;
  font-size: 1rem;
  cursor: pointer;
  pointer-events: all;
  transition: background-color 0.3s ease;
}

.select-btn:hover {
  background: #0056b3;
}

.selected-files {
  margin-top: 20px;
  padding: 20px;
  background: #f8f9fa;
  border-radius: 8px;
}

.file-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px;
  background: white;
  margin-bottom: 10px;
  border-radius: 6px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.remove-btn {
  background: #dc3545;
  color: white;
  border: none;
  width: 30px;
  height: 30px;
  border-radius: 50%;
  cursor: pointer;
  font-size: 1.2rem;
  display: flex;
  align-items: center;
  justify-content: center;
}

.upload-btn {
  background: #28a745;
  color: white;
  border: none;
  padding: 12px 24px;
  border-radius: 6px;
  font-size: 1rem;
  cursor: pointer;
  margin-top: 15px;
  transition: background-color 0.3s ease;
}

.upload-btn:hover:not(:disabled) {
  background: #218838;
}

.upload-btn:disabled {
  background: #6c757d;
  cursor: not-allowed;
}

.progress-bar {
  width: 100%;
  height: 8px;
  background: #e9ecef;
  border-radius: 4px;
  margin-top: 15px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: #007bff;
  transition: width 0.3s ease;
}

.refresh-btn {
  background: #17a2b8;
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 6px;
  cursor: pointer;
  margin-bottom: 20px;
}

.no-files {
  text-align: center;
  color: #666;
  font-size: 1.1rem;
  padding: 40px;
}

.files-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
}

.file-card {
  background: #f8f9fa;
  border-radius: 8px;
  padding: 20px;
  border: 1px solid #dee2e6;
  transition: transform 0.2s ease;
}

.file-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
}

.file-info h4 {
  color: #333;
  margin-bottom: 10px;
  word-break: break-word;
}

.file-info p {
  color: #666;
  margin-bottom: 5px;
  font-size: 0.9rem;
}

.file-actions {
  margin-top: 15px;
}

.download-btn {
  background: #007bff;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: background-color 0.3s ease;
}

.download-btn:hover {
  background: #0056b3;
}

@media (max-width: 768px) {
  #app {
    padding: 10px;
  }

  .container {
    padding: 20px;
  }

  .upload-section, .files-section {
    padding: 15px;
  }

  .files-grid {
    grid-template-columns: 1fr;
  }

  .header h1 {
    font-size: 2rem;
  }
}
</style>