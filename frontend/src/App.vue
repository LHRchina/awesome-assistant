<template>
  <div id="app">
    <div class="container">
      <h1>File Upload System</h1>
      
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
      </div>
      
      <!-- Files List Section -->
      <div class="files-section">
        <h2>Uploaded Files</h2>
        <button @click="loadFiles" class="refresh-btn">Refresh</button>
        
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
</template>

<script>
import axios from 'axios'

const API_BASE_URL = 'http://localhost:8080'

export default {
  name: 'App',
  data() {
    return {
      selectedFiles: [],
      uploadedFiles: [],
      uploading: false,
      loading: false,
      uploadProgress: 0
    }
  },
  mounted() {
    this.loadFiles()
  },
  methods: {
    handleFileSelect(event) {
      const files = Array.from(event.target.files)
      this.selectedFiles = [...this.selectedFiles, ...files]
    },
    
    handleDrop(event) {
      event.preventDefault()
      const files = Array.from(event.dataTransfer.files)
      this.selectedFiles = [...this.selectedFiles, ...files]
    },
    
    removeFile(index) {
      this.selectedFiles.splice(index, 1)
    },
    
    async uploadFiles() {
      if (this.selectedFiles.length === 0) return
      
      this.uploading = true
      this.uploadProgress = 0
      
      try {
        for (let i = 0; i < this.selectedFiles.length; i++) {
          const file = this.selectedFiles[i]
          const formData = new FormData()
          formData.append('file', file)
          
          await axios.post(`${API_BASE_URL}/upload`, formData, {
            headers: {
              'Content-Type': 'multipart/form-data'
            },
            onUploadProgress: (progressEvent) => {
              const progress = Math.round(
                ((i + progressEvent.loaded / progressEvent.total) / this.selectedFiles.length) * 100
              )
              this.uploadProgress = progress
            }
          })
        }
        
        this.selectedFiles = []
        this.loadFiles()
        alert('Files uploaded successfully!')
      } catch (error) {
        console.error('Upload error:', error)
        alert('Error uploading files. Please try again.')
      } finally {
        this.uploading = false
        this.uploadProgress = 0
      }
    },
    
    async loadFiles() {
      this.loading = true
      try {
        const response = await axios.get(`${API_BASE_URL}/files`)
        this.uploadedFiles = response.data.files
      } catch (error) {
        console.error('Error loading files:', error)
        alert('Error loading files. Please check if the server is running.')
      } finally {
        this.loading = false
      }
    },
    
    async downloadFile(fileId, filename) {
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
        alert('Error downloading file.')
      }
    },
    
    formatFileSize(bytes) {
      if (bytes === 0) return '0 Bytes'
      const k = 1024
      const sizes = ['Bytes', 'KB', 'MB', 'GB']
      const i = Math.floor(Math.log(bytes) / Math.log(k))
      return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
    },
    
    formatDate(dateString) {
      return new Date(dateString).toLocaleString()
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
  font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
  background-color: #f5f5f5;
}

.container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}

h1 {
  text-align: center;
  color: #333;
  margin-bottom: 30px;
  font-size: 2.5rem;
}

h2 {
  color: #444;
  margin-bottom: 20px;
  font-size: 1.8rem;
}

.upload-section, .files-section {
  background: white;
  border-radius: 12px;
  padding: 30px;
  margin-bottom: 30px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
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

.loading, .no-files {
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
  .container {
    padding: 10px;
  }
  
  .upload-section, .files-section {
    padding: 20px;
  }
  
  .files-grid {
    grid-template-columns: 1fr;
  }
  
  h1 {
    font-size: 2rem;
  }
}
</style>