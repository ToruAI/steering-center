# Steering Center - Open Source Starter Template

A complete self-hosted dashboard for monitoring system resources and executing scripts with real-time terminal output. Built with Rust (Axum) backend and React (Vite + TypeScript) frontend.

## Features

- **System Monitoring**: Real-time CPU, RAM, and uptime statistics
- **Script Execution**: Execute shell scripts with live terminal output via WebSocket
- **Task Management**: Cancel running tasks, view execution history
- **Quick Actions**: Create customizable quick action buttons for frequently used scripts
- **Mobile-First UI**: Responsive design optimized for mobile and desktop
- **Single Binary**: Everything bundled into one executable - no Docker required
- **SQLite Database**: Lightweight, file-based database for settings and history

## Architecture

**Monolith binary** serving both API and static frontend:

- **Backend**: Rust (Axum) with SQLite
- **Frontend**: Vite + React + TypeScript + Tailwind CSS
- **Communication**: REST API + WebSocket for real-time streaming

## Quick Start

### Prerequisites

- Rust (latest stable)
- Node.js 20+ and npm
- A Unix-like system (Linux, macOS)

### Build Steps

1. **Build the frontend:**
   ```bash
   cd frontend
   npm install
   npm run build
   cd ..
   ```

2. **Build the backend:**
   ```bash
   cargo build --release
   ```

3. **Run the server:**
   ```bash
   ./target/release/steering-center
   ```

   The server will start on `http://localhost:3000`

### Development Mode

For development, run frontend and backend separately:

**Terminal 1 - Frontend (with hot reload):**
```bash
cd frontend
npm run dev
```

**Terminal 2 - Backend:**
```bash
cargo run
```

Note: In development, update the WebSocket URL in `frontend/src/hooks/useWebSocket.ts` to point to the backend port.

## Project Structure

```
/my-dashboard
├── /frontend              # React frontend
│   ├── /src
│   │   ├── /components   # UI components (Layout)
│   │   ├── /pages        # Dashboard, Scripts, Settings
│   │   ├── /hooks        # useWebSocket, useSystemStats
│   │   ├── /lib          # API client, utilities
│   │   └── main.tsx
│   ├── package.json
│   └── vite.config.ts
├── /src                   # Rust backend
│   ├── main.rs           # Server entry point
│   ├── /routes           # API routes and WebSocket handler
│   ├── /services         # System monitoring, script executor
│   └── db.rs             # Database schema and queries
├── /scripts              # Example shell scripts
├── Cargo.toml
└── README.md
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/health` | GET | Health check |
| `/api/resources` | GET | CPU, RAM, uptime stats |
| `/api/scripts` | GET | List available scripts |
| `/api/settings` | GET/PUT | App configuration |
| `/api/history` | GET | Task execution history |
| `/api/quick-actions` | GET/POST/DELETE | Manage quick actions |
| `/api/ws` | WebSocket | Real-time terminal |

## WebSocket Protocol

**Client sends:**
- `{"type": "run", "script": "backup.sh"}`
- `{"type": "cancel", "task_id": "uuid"}`

**Server sends:**
- `{"type": "started", "task_id": "uuid"}`
- `{"type": "stdout", "data": "line of output"}`
- `{"type": "stderr", "data": "error line"}`
- `{"type": "exit", "code": 0}`
- `{"type": "cancelled"}`
- `{"type": "error", "data": "error message"}`

## Database Schema

The SQLite database (`steering.db`) contains:

- **settings**: Key-value configuration (e.g., `scripts_dir`)
- **task_history**: Execution logs with timestamps, exit codes, and output
- **quick_actions**: User-defined quick action buttons

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `STEERING_HOST` | `127.0.0.1` | Bind address. Set to `0.0.0.0` to allow external connections |
| `STEERING_PORT` | `3000` | Server port |
| `RUST_LOG` | `steering_center=info` | Log level (debug, info, warn, error) |

### Scripts Directory

By default, scripts are loaded from `./scripts`. Change this in Settings or by updating the `scripts_dir` setting in the database.

### Adding Scripts

1. Place executable shell scripts in the scripts directory
2. Ensure scripts have execute permissions: `chmod +x script.sh`
3. Scripts will appear in the Scripts page dropdown

## Extending the Application

### Adding a New API Endpoint

1. Add route handler in `src/routes/api.rs`:
   ```rust
   .route("/api/my-endpoint", get(my_handler))
   ```

2. Implement handler function:
   ```rust
   async fn my_handler(State(state): State<AppState>) -> Result<Json<MyData>, StatusCode> {
       // Your logic here
       Ok(Json(data))
   }
   ```

### Adding a New Frontend Page

1. Create page component in `frontend/src/pages/MyPage.tsx`
2. Add route in `frontend/src/App.tsx`:
   ```tsx
   <Route path="/my-page" element={<MyPage />} />
   ```
3. Add navigation item in `frontend/src/components/Layout.tsx`

### Adding a New Service

1. Create service module in `src/services/my_service.rs`
2. Export in `src/services/mod.rs`
3. Use in routes or main.rs

## Deployment

### Cloudflare Zero Trust (Recommended)

1. Build release binary: `cargo build --release`
2. Upload binary to your VPS
3. Configure Cloudflare Zero Trust tunnel to forward traffic to `localhost:3000`
4. Set up authentication rules in Cloudflare dashboard

### Systemd Service

Create `/etc/systemd/system/steering-center.service`:

```ini
[Unit]
Description=Steering Center Dashboard
After=network.target

[Service]
Type=simple
User=your-user
WorkingDirectory=/path/to/steering-center
ExecStart=/path/to/steering-center/target/release/steering-center
Restart=always

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable steering-center
sudo systemctl start steering-center
```

### Reverse Proxy (Nginx)

```nginx
server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
    }
}
```

## Security Considerations

- **Authentication**: This starter template does not include authentication. Use Cloudflare Zero Trust, reverse proxy with auth, or implement your own auth layer.
- **Script Execution**: Scripts run with the same permissions as the server process. Ensure proper file permissions and validate script paths.
- **WebSocket**: WebSocket connections are not authenticated by default. Add authentication if exposing publicly.

## Troubleshooting

### Frontend not loading

- Ensure `frontend/dist` directory exists and contains built files
- Check that `main.rs` is serving from the correct directory
- Verify build output: `ls -la frontend/dist`

### WebSocket connection fails

- Check that WebSocket URL matches server protocol (ws:// vs wss://)
- Verify CORS settings if accessing from different origin
- Check browser console for connection errors

### Scripts not executing

- Verify scripts have execute permissions: `chmod +x script.sh`
- Check scripts directory path in settings
- Review server logs for execution errors

## Contributing

This is an open-source starter template. Feel free to fork and customize for your needs!

## License

MIT License - feel free to use this template for any purpose.
