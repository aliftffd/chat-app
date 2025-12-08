# Docker Setup Guide for Chat-App with SQLite

## Overview

This chat application now uses **SQLite** for message persistence, making it easy to deploy in Docker with no external database dependencies.

## Dockerfile Changes

### ✅ Fixed Issues:

1. **Typo Fixed:** `/va/lib/apt/lists/*` → `/var/lib/apt/lists/*`
2. **Added SQLite runtime:** `libsqlite3-0` package
3. **Database persistence:** Volume mount at `/data`
4. **Correct default mode:** Server instead of client
5. **Environment variable:** `XDG_DATA_HOME=/data` for database location

### Key Features:

- **Multi-stage build** for smaller image size
- **Persistent storage** via Docker volumes
- **SQLite bundled** (no external database needed)
- **Ready for production** with proper defaults

## Quick Start

### 1. Build the Image

```bash
docker build -t chat-app:latest .
```

### 2. Run Server (Standalone)

```bash
docker run -d \
  --name chat-server \
  -p 8080:8080 \
  -v chat-data:/data \
  -e RUST_LOG=chat_app=info \
  chat-app:latest
```

### 3. Connect Client (from host)

```bash
cargo run -- client -a localhost:8080
```

## Using Docker Compose (Recommended)

### Start Server Only

```bash
docker-compose up -d
```

This will:
- Start the chat server on port 8080
- Create a persistent volume for the SQLite database
- Enable auto-restart on failure

### Start Server + Client

```bash
docker-compose --profile client up
```

This starts both server and a client container (useful for testing).

### View Logs

```bash
# Server logs
docker-compose logs -f chat-server

# All logs
docker-compose logs -f
```

### Stop Services

```bash
docker-compose down
```

### Stop and Remove Database

```bash
# WARNING: This deletes all message history!
docker-compose down -v
```

## Database Persistence

### Where is the database stored?

- **Inside container:** `/data/chat-app/messages.db`
- **Docker volume:** `chat-data` (managed by Docker)

### Access the database

```bash
# Enter the container
docker exec -it chat-server sh

# Run SQLite commands
cd /data/chat-app
ls -lh messages.db
sqlite3 messages.db "SELECT COUNT(*) FROM messages;"
sqlite3 messages.db "SELECT username, content FROM messages ORDER BY timestamp DESC LIMIT 5;"
```

### Backup the database

```bash
# Copy database from container to host
docker cp chat-server:/data/chat-app/messages.db ./backup-$(date +%Y%m%d).db

# Or backup the entire volume
docker run --rm \
  -v chat-data:/data \
  -v $(pwd):/backup \
  busybox tar czf /backup/chat-data-backup.tar.gz /data
```

### Restore the database

```bash
# Copy database from host to container
docker cp ./backup.db chat-server:/data/chat-app/messages.db

# Restart container to pick up changes
docker-compose restart chat-server
```

## Production Deployment

### On a Server

```bash
# Pull and run on remote server
git clone <your-repo>
cd chat-app
docker-compose up -d

# Check status
docker-compose ps
docker-compose logs -f chat-server
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `XDG_DATA_HOME` | `/data` | Database directory |
| `RUST_LOG` | `chat_app=info` | Log level |

### Expose to Network

Edit `docker-compose.yml` or use:

```bash
docker run -d \
  --name chat-server \
  -p 0.0.0.0:8080:8080 \
  -v chat-data:/data \
  --restart unless-stopped \
  chat-app:latest
```

**⚠️ Security Note:** For production, use Tailscale or firewall rules to restrict access.

## Tailscale Integration

### Option 1: Run on Tailscale Host

1. Install Tailscale on the Docker host
2. Run chat-app container as shown above
3. Connect from other Tailscale devices using Tailscale IP

```bash
# From another Tailscale device
cargo run -- client -a 100.x.x.x:8080
```

### Option 2: Container in Tailscale Network

```dockerfile
# Add to Dockerfile
RUN apt-get install -y curl
RUN curl -fsSL https://tailscale.com/install.sh | sh

# Run with Tailscale
docker run -d \
  --name chat-server \
  --cap-add NET_ADMIN \
  --device /dev/net/tun \
  -v chat-data:/data \
  -e TS_AUTHKEY=your-auth-key \
  chat-app-tailscale:latest
```

## Troubleshooting

### Database locked error

```bash
# Check if multiple processes are accessing the database
docker exec chat-server ps aux | grep chat-app

# Restart the container
docker-compose restart chat-server
```

### Database not persisting

```bash
# Check volume exists
docker volume ls | grep chat-data

# Inspect volume
docker volume inspect chat-data

# Verify files in container
docker exec chat-server ls -lah /data/chat-app/
```

### Connection refused

```bash
# Check server is listening
docker exec chat-server netstat -tuln | grep 8080

# Check server logs
docker-compose logs chat-server

# Test from inside container
docker exec chat-server chat-app client --address localhost:8080
```

### Image size too large

```bash
# Check image size
docker images chat-app

# Use multi-stage build (already implemented)
# Current Dockerfile uses rust:latest → debian:bookworm-slim
# Final image should be ~100-200MB
```

## Development Workflow

### Local development (no Docker)

```bash
# Terminal 1: Server
cargo run -- server -a 127.0.0.1:8080

# Terminal 2: Client
cargo run -- client -a 127.0.0.1:8080
```

### Test in Docker

```bash
# Build and test
docker build -t chat-app:test .
docker run --rm -it -p 8080:8080 chat-app:test

# In another terminal
cargo run -- client -a localhost:8080
```

### Hot reload (development)

```bash
# Use cargo-watch
cargo install cargo-watch
cargo watch -x 'run -- server -a 127.0.0.1:8080'
```

## Performance Notes

### SQLite in Docker

- ✅ **Pros:** No external dependencies, simple deployment, fast for small-medium loads
- ✅ **Single-user workload:** Perfect for your use case (4 devices max)
- ⚠️ **Write performance:** Adequate for chat (not high-frequency writes)
- ✅ **Read performance:** Excellent for message history queries

### Expected Performance

- **Concurrent users:** 4-10 devices (your personal machines)
- **Messages/second:** 10-100 (more than enough for chat)
- **Database size:** ~1MB per 10,000 messages
- **History queries:** Sub-millisecond for last 1000 messages

## Maintenance

### View database stats

```bash
docker exec chat-server sqlite3 /data/chat-app/messages.db << 'SQL'
.tables
SELECT COUNT(*) as total_messages FROM messages;
SELECT message_type, COUNT(*) as count FROM messages GROUP BY message_type;
SELECT device_id, COUNT(*) as message_count FROM messages WHERE device_id IS NOT NULL GROUP BY device_id;
SQL
```

### Optimize database

```bash
docker exec chat-server sqlite3 /data/chat-app/messages.db "VACUUM;"
docker exec chat-server sqlite3 /data/chat-app/messages.db "ANALYZE;"
```

### Clean old messages (optional)

```bash
# Keep only last 10,000 messages
docker exec chat-server sqlite3 /data/chat-app/messages.db << 'SQL'
DELETE FROM messages WHERE timestamp < (
  SELECT timestamp FROM messages ORDER BY timestamp DESC LIMIT 1 OFFSET 10000
);
VACUUM;
SQL
```

## Summary

Your Docker setup is now:
- ✅ **Fixed** for SQLite compatibility
- ✅ **Persistent** database storage
- ✅ **Production-ready** with proper defaults
- ✅ **Simple** to deploy (no external database)
- ✅ **Tailscale-compatible** for secure networking

**Next Steps:**
1. Test locally: `docker-compose up`
2. Deploy to server: `scp` files and run
3. Connect from Tailscale devices
4. Enjoy persistent chat history! 🚀
