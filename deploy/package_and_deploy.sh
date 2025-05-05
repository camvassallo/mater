#!/bin/bash

APP_NAME="myapp"
SERVER="root@your-server-ip-or-domain"
PACKAGE_NAME="deploy_package.tar.gz"

# Clean up any old package
rm -f $PACKAGE_NAME

echo "▶️ Building Rust app..."
cd ../backend
cargo build --release

echo "▶️ Building React app..."
cd ../frontend
npm run build

echo "📦 Packaging files..."
cd ..
mkdir -p deploy/tmp_package
cp backend/target/release/$APP_NAME deploy/tmp_package/
cp backend/backend.service deploy/tmp_package/
cp -r frontend/build deploy/tmp_package/frontend
tar -czvf $PACKAGE_NAME -C deploy/tmp_package .

echo "📤 Uploading to server..."
scp $PACKAGE_NAME $SERVER:~/

echo "🚀 Deploying on server..."
ssh $SERVER << EOF
  set -e
  mkdir -p ~/deploy_temp
  tar -xzvf $PACKAGE_NAME -C ~/deploy_temp

  # Install backend binary
  sudo mv ~/deploy_temp/$APP_NAME /usr/local/bin/
  sudo chmod +x /usr/local/bin/$APP_NAME

  # Install systemd service
  sudo mv ~/deploy_temp/backend.service /etc/systemd/system/backend.service
  sudo systemctl daemon-reexec
  sudo systemctl enable backend.service
  sudo systemctl restart backend.service

  # Install React frontend
  sudo mkdir -p /var/www/yourapp
  sudo rm -rf /var/www/yourapp/*
  sudo cp -r ~/deploy_temp/frontend/* /var/www/yourapp/

  # Reload Nginx
  sudo systemctl reload nginx

  # Cleanup
  rm -rf ~/deploy_temp ~/deploy_package.tar.gz
EOF

echo "✅ Deployment complete!"