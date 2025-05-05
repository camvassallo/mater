#!/bin/bash

# Build and copy Rust binary
cd ../backend
cargo build --release
scp target/release/myapp root@your-server:/usr/local/bin/

# Copy systemd service
scp backend.service root@your-server:/etc/systemd/system/

# Build and copy React frontend
cd ../frontend
npm run build
scp -r build/* root@your-server:/var/www/yourapp/

# Restart backend and reload Nginx
ssh root@your-server "systemctl daemon-reload && systemctl enable --now backend && systemctl restart nginx"
