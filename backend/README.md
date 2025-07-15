# Awesome Assistant - File Upload Backend

A secure file upload service with Google OAuth authentication and PostgreSQL database integration.

## Features

- 🔐 Google OAuth 2.0 authentication
- 📁 Secure file upload to Cloudflare R2
- 🗄️ PostgreSQL database for user and file management
- 🔒 User-specific file access control
- 🌐 Modern web interface
- 🚀 Built with Rust and Actix-web

## Prerequisites

- Rust (latest stable version)
- PostgreSQL database
- Cloudflare R2 storage account
- Google Cloud Console project with OAuth 2.0 credentials

## Setup Instructions

### 1. Database Setup

Ensure PostgreSQL is running and create the required tables:

```sql
-- Connect to your PostgreSQL database
psql -h 127.0.0.1 -p 5432 -U awesome -d awesome

-- Create users table
CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    third_party_id VARCHAR(255) UNIQUE NOT NULL
);

-- Create user_files table
CREATE TABLE user_files (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    file_key VARCHAR(255) NOT NULL
);
```

### 2. Cloudflare R2 Configuration

Create `src/conf/init.toml` with your Cloudflare R2 credentials:

```toml
[cloudflare]
account_id = "your-cloudflare-account-id"
access_key_id = "your-r2-access-key-id"
access_key_secret = "your-r2-secret-access-key"
```

### 3. Google OAuth Setup

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select an existing one
3. Enable the Google+ API
4. Go to "Credentials" → "Create Credentials" → "OAuth 2.0 Client IDs"
5. Configure the OAuth consent screen
6. Add authorized JavaScript origins:
   - `http://localhost:8080`
   - `http://127.0.0.1:8080`
7. Copy your Client ID
8. Update `static/index.html` and replace `YOUR_GOOGLE_CLIENT_ID` with your actual Client ID

### 4. Environment Configuration

Update the database connection string in `src/main.rs` if needed:

```rust
let database_url = "host=127.0.0.1 port=5432 user=awesome password=mysecretpassword-awesome-assistant dbname=awesome";
```

**Important**: Change the JWT secret in production:

```rust
let jwt_secret = "your-super-secret-jwt-key-change-this-in-production".to_string();
```

### 5. Build and Run

```bash
# Install dependencies
cargo build

# Run the server
cargo run
```

The server will start on `http://localhost:8080`

## API Endpoints

### Public Endpoints

- `GET /` - Serve the web interface
- `POST /login` - Google OAuth login

### Protected Endpoints (Require Authentication)

- `GET /me` - Get current user information
- `POST /upload` - Upload files
- `GET /files` - List user's files
- `GET /download/{id}` - Download a specific file

## Authentication

The API uses JWT tokens for authentication. Include the token in the Authorization header:

```
Authorization: Bearer <your-jwt-token>
```

## Security Features

- ✅ Google OAuth 2.0 authentication
- ✅ JWT token-based authorization
- ✅ User-specific file access control
- ✅ Database-backed user and file management
- ✅ Secure file storage with Cloudflare R2
- ✅ CORS protection
- ✅ Input validation and error handling

## File Upload Flow

1. User authenticates with Google OAuth
2. Backend verifies Google token and creates/finds user in database
3. JWT token is issued for subsequent requests
4. User uploads files (requires authentication)
5. Files are stored in Cloudflare R2
6. File metadata is saved in PostgreSQL with user association
7. Users can only access their own files

## Development

### Project Structure

```
src/
├── main.rs              # Main application and routes
├── auth.rs              # Authentication logic
└── storage/
    ├── mod.rs           # Storage abstraction
    └── cloudflare_s3.rs # Cloudflare R2 implementation
static/
└── index.html           # Web interface
src/conf/
└── init.toml           # Configuration file
```

### Adding New Features

1. **New API endpoints**: Add routes in `main.rs`
2. **Authentication changes**: Modify `auth.rs`
3. **Storage backends**: Implement in `storage/` module
4. **Frontend updates**: Edit `static/index.html`

## Troubleshooting

### Common Issues

1. **Database connection failed**
   - Ensure PostgreSQL is running
   - Check connection string in `main.rs`
   - Verify database and user exist

2. **Google OAuth not working**
   - Verify Client ID in `index.html`
   - Check authorized origins in Google Console
   - Ensure OAuth consent screen is configured

3. **File upload fails**
   - Check Cloudflare R2 credentials in `init.toml`
   - Verify bucket exists and is accessible
   - Check network connectivity

4. **Authentication errors**
   - Verify JWT secret is set
   - Check token expiration (24 hours by default)
   - Ensure user exists in database

### Logs

The application uses `env_logger`. Set log level:

```bash
RUST_LOG=debug cargo run
```

## Production Deployment

### Security Checklist

- [ ] Change JWT secret to a strong, random value
- [ ] Use environment variables for sensitive configuration
- [ ] Enable HTTPS/TLS
- [ ] Configure proper CORS origins
- [ ] Set up database connection pooling
- [ ] Implement rate limiting
- [ ] Add monitoring and logging
- [ ] Regular security updates

### Environment Variables

For production, consider using environment variables:

```bash
export DATABASE_URL="postgresql://user:pass@host:port/dbname"
export JWT_SECRET="your-production-jwt-secret"
export CLOUDFLARE_ACCOUNT_ID="your-account-id"
export CLOUDFLARE_ACCESS_KEY_ID="your-access-key"
export CLOUDFLARE_ACCESS_KEY_SECRET="your-secret-key"
```

## License

MIT License - see LICENSE file for details.