import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: {
    host: true, // Allow external access
    allowedHosts: ['campom.org'], // Allow Cloudflare tunnel domain
    proxy: {
      '/api': 'http://localhost:8000'  // or your Rust API port
    }
  }
});