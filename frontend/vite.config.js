import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: {
    host: true, // Allow external access
    allowedHosts: ['.trycloudflare.com'], // Allow Cloudflare tunnel domains
    proxy: {
      '/api': 'http://localhost:8000'  // or your Rust API port
    }
  }
});