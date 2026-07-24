import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';

const API_TARGET = 'http://localhost:3000';

export default defineConfig({
  plugins: [react(), tailwindcss()],
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
