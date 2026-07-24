import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';
import { readFileSync, writeFileSync } from 'fs';
import { resolve } from 'path';

const API_TARGET = 'http://localhost:3000';

function buildHash() {
  return `${Date.now()}`;
}

function injectServiceWorkerHash() {
  return {
    name: 'inject-service-worker-hash',
    closeBundle() {
      const swPath = resolve(__dirname, 'dist', 'sw.js');
      const sw = readFileSync(swPath, 'utf-8');
      writeFileSync(swPath, sw.replace(/__BUILD_HASH__/g, buildHash()));
    },
  };
}

export default defineConfig({
  plugins: [react(), tailwindcss(), injectServiceWorkerHash()],
  resolve: {
    dedupe: ['react', 'react-dom', 'react-router-dom'],
  },
  define: {
    __WEB_DEFAULT_API_URL__: JSON.stringify(''),
  },
  server: {
    port: 5174,
    proxy: {
      '/auth': API_TARGET,
      '/users': API_TARGET,
      '/organizations': API_TARGET,
      '/channels': API_TARGET,
      '/messages': API_TARGET,
      '/direct_messages': API_TARGET,
      '/files': API_TARGET,
      '/plugins': API_TARGET,
      '/web-push': API_TARGET,
      '/api': API_TARGET,
      '/mcp': API_TARGET,
      '/websocket': {
        target: 'ws://localhost:3000',
        ws: true,
      },
    },
  },
  build: {
    outDir: 'dist',
  },
});
