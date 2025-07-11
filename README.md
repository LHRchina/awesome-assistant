# File Upload System

A full-stack file upload application with Vue.js frontend and Rust backend.

## Features

- **Frontend (Vue.js)**:
  - Drag and drop file upload
  - Multiple file selection
  - Upload progress tracking
  - File list with download functionality
  - Responsive design
  - Modern UI with animations

- **Backend (Rust)**:
  - RESTful API with Actix-web
  - File upload endpoint
  - File listing endpoint
  - File download endpoint
  - CORS support for frontend integration
  - File metadata storage

## Project Structure

```
awesome-assistant/
├── backend/
│   ├── src/
│   │   └── main.rs
│   ├── Cargo.toml
│   └── uploads/          # Created automatically
├── frontend/
│   ├── src/
│   │   ├── App.vue
│   │   └── main.js
│   ├── public/
│   │   ├── index.html
│   │   └── favicon.ico
│   ├── package.json
│   └── vue.config.js
└── README.md
```

## Prerequisites

- **Rust** (latest stable version)
- **Node.js** (v16 or higher)
- **npm** or **yarn**

## Setup Instructions

### Backend Setup

1. Navigate to the backend directory:
   ```bash
   cd backend
   ```

2. Install dependencies and run:
   ```bash
   cargo run
   ```

   The server will start on `http://localhost:8080`

### Frontend Setup

1. Navigate to the frontend directory:
   ```bash
   cd frontend
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Start the development server:
   ```bash
   npm run serve
   ```

   The frontend will be available on `http://localhost:3000`

## API Endpoints

### Backend API

- **POST** `/upload` - Upload a file
  - Content-Type: `multipart/form-data`
  - Body: File data
  - Response: File metadata

- **GET** `/files` - List all uploaded files
  - Response: Array of file metadata

- **GET** `/download/{id}` - Download a file by ID
  - Response: File content with appropriate headers

## Usage

1. Start both backend and frontend servers
2. Open your browser to `http://localhost:3000`
3. Upload files by:
   - Clicking "Select Files" button
   - Dragging and dropping files onto the upload area
4. View uploaded files in the "Uploaded Files" section
5. Download files by clicking the "Download" button

## Development

### Backend Development

- The backend uses Actix-web framework
- Files are stored in the `uploads/` directory
- File metadata is stored as JSON files alongside the uploaded files
- CORS is configured to allow requests from the frontend

### Frontend Development

- Built with Vue 3 Composition API
- Uses Axios for HTTP requests
- Responsive design with CSS Grid and Flexbox
- Drag and drop functionality with native HTML5 APIs

## Production Deployment

### Backend
```bash
cd backend
cargo build --release
./target/release/file-upload-backend
```

### Frontend
```bash
cd frontend
npm run build
# Serve the dist/ directory with a web server
```

## Security Considerations

- File uploads are stored locally (consider cloud storage for production)
- No file type restrictions (implement as needed)
- No authentication (add as required)
- File size limits can be configured in Actix-web

## License

MIT License