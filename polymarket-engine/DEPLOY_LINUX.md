# Polymarket Engine — Linux Server Migration Guide

## Prerequisites

- Linux server (Ubuntu 22.04+ / Debian 12+ recommended)
- 1 CPU core, 512MB RAM minimum
- SSH access
- Domain or static IP (optional, for remote dashboard access)

---

## 1. Server Setup

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install build essentials (for rusqlite bundled SQLite)
sudo apt install -y build-essential pkg-config libssl-dev curl git

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify
rustc --version
cargo --version
```

## 2. Clone / Upload Project

```bash
# Option A: Git clone
cd ~
git clone <your-repo-url> polymarket-engine
cd polymarket-engine

# Option B: SCP from local Windows
# (from Windows terminal)
# scp -r D:\Skills\polymarket-engine user@server:~/polymarket-engine
```

## 3. Configure Environment

```bash
cd ~/polymarket-engine
cp .env.example .env
nano .env
```

Edit these values:
```env
# CRITICAL — your Polygon wallet private key
POLYMARKET_PRIVATE_KEY=0x_your_private_key_here
POLYMARKET_FUNDER_ADDRESS=0x_your_polymarket_address

# Production settings
INITIAL_CAPITAL=50.0
DASHBOARD_HOST=0.0.0.0    # Listen on all interfaces
DASHBOARD_PORT=3001

# Database — persistent across restarts
DB_PATH=/home/user/polymarket-data/engine.db

# Production intervals
MARKET_SCAN_INTERVAL_SECS=300
POSITION_UPDATE_INTERVAL_SECS=60
HEARTBEAT_INTERVAL_SECS=30
```

## 4. Build Release Binary

```bash
cd ~/polymarket-engine
cargo build --release

# Binary location: target/release/polymarket-engine
# Size: ~15-20MB (SQLite bundled, no external deps)
```

## 5. Create Data Directory

```bash
mkdir -p /home/user/polymarket-data
# DB will be auto-created on first run at DB_PATH
```

## 6. Test Run

```bash
cd ~/polymarket-engine
./target/release/polymarket-engine

# Should see:
# ╔══════════════════════════════════════════════╗
# ║   POLYMARKET EXECUTION ENGINE v0.3           ║
# ║   Mode: COMMAND-DRIVEN (OpenClaw = brain)    ║
# ║   Capital: $50.00                            ║
# ║   DB: /home/user/polymarket-data/engine.db   ║
# ╚══════════════════════════════════════════════╝
# Database migrations applied
# Dashboard + Command API on http://0.0.0.0:3001

# Ctrl+C to stop
```

## 7. Systemd Service (Auto-start + Auto-restart)

```bash
sudo nano /etc/systemd/system/polymarket-engine.service
```

Paste:
```ini
[Unit]
Description=Polymarket Trading Engine
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=user
Group=user
WorkingDirectory=/home/user/polymarket-engine
ExecStart=/home/user/polymarket-engine/target/release/polymarket-engine
Restart=always
RestartSec=10
Environment=RUST_LOG=info

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=/home/user/polymarket-data
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable polymarket-engine
sudo systemctl start polymarket-engine

# Check status
sudo systemctl status polymarket-engine

# View logs
sudo journalctl -u polymarket-engine -f
```

## 8. Firewall

```bash
# Allow dashboard port (only if remote access needed)
sudo ufw allow 3001/tcp

# If using Nginx reverse proxy instead:
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
```

## 9. Nginx Reverse Proxy (Optional — for HTTPS + domain)

```bash
sudo apt install -y nginx certbot python3-certbot-nginx
sudo nano /etc/nginx/sites-available/polymarket
```

```nginx
server {
    listen 80;
    server_name your-domain.com;

    # REST API
    location /api/ {
        proxy_pass http://127.0.0.1:3001;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # WebSocket
    location /ws {
        proxy_pass http://127.0.0.1:3001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_read_timeout 86400;
    }

    # React Dashboard (if hosted on same server)
    location / {
        root /home/user/polymarket-dashboard/dist;
        try_files $uri $uri/ /index.html;
    }
}
```

```bash
sudo ln -s /etc/nginx/sites-available/polymarket /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx

# HTTPS (free SSL via Let's Encrypt)
sudo certbot --nginx -d your-domain.com
```

## 10. React Dashboard Deployment

```bash
# On local Windows, build production dashboard:
cd D:\Skills\polymarket-dashboard
npm run build

# Upload dist/ to server:
scp -r dist/ user@server:~/polymarket-dashboard/dist

# Update vite.config.js API URL before build if needed:
# proxy target → http://localhost:3001 (same server)
# or your-domain.com if different server
```

## 11. SQLite Backup Strategy

```bash
# Cron job: daily backup
crontab -e
```

Add:
```cron
# Backup SQLite DB daily at 00:00 UTC
0 0 * * * cp /home/user/polymarket-data/engine.db /home/user/polymarket-data/backups/engine_$(date +\%Y\%m\%d).db

# Keep only last 30 days
0 1 * * * find /home/user/polymarket-data/backups/ -name "engine_*.db" -mtime +30 -delete
```

```bash
mkdir -p /home/user/polymarket-data/backups
```

## 12. Monitoring Commands

```bash
# Engine status
sudo systemctl status polymarket-engine

# Live logs
sudo journalctl -u polymarket-engine -f --no-pager

# Check DB size
ls -lh /home/user/polymarket-data/engine.db

# Quick DB query (install sqlite3)
sudo apt install -y sqlite3
sqlite3 /home/user/polymarket-data/engine.db "SELECT COUNT(*) FROM trades;"
sqlite3 /home/user/polymarket-data/engine.db "SELECT * FROM portfolio_snapshots ORDER BY created_at DESC LIMIT 5;"

# Health check
curl http://localhost:3001/api/health

# Current state
curl http://localhost:3001/api/state | python3 -m json.tool
```

## 13. Update / Redeploy

```bash
cd ~/polymarket-engine
git pull                          # or scp new files
cargo build --release
sudo systemctl restart polymarket-engine

# DB is preserved — engine recovers state from SQLite on restart
```

---

## Directory Structure on Server

```
/home/user/
├── polymarket-engine/        # Rust source + binary
│   ├── src/
│   ├── target/release/
│   ├── .env
│   ├── Cargo.toml
│   └── ...
├── polymarket-dashboard/     # React build
│   └── dist/
└── polymarket-data/          # Persistent data
    ├── engine.db             # SQLite database
    └── backups/
        ├── engine_20260212.db
        └── ...
```

## Quick Reference

| Action | Command |
|--------|---------|
| Start | `sudo systemctl start polymarket-engine` |
| Stop | `sudo systemctl stop polymarket-engine` |
| Restart | `sudo systemctl restart polymarket-engine` |
| Logs | `sudo journalctl -u polymarket-engine -f` |
| Status | `curl localhost:3001/api/health` |
| DB shell | `sqlite3 ~/polymarket-data/engine.db` |
| Backup now | `cp ~/polymarket-data/engine.db ~/polymarket-data/backups/engine_manual.db` |
| Rebuild | `cargo build --release && sudo systemctl restart polymarket-engine` |
