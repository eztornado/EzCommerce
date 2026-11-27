import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// En desarrollo el proxy envía /api al backend Rust local (34187).
// En producción nginx sirve el SPA y hace proxy de /api al API.
export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      '/api': 'http://127.0.0.1:34187',
      '/storage': 'http://127.0.0.1:34187',
      // Navegación agéntica (#40): mismos mapeos que nginx en producción.
      '/llms.txt': 'http://127.0.0.1:34187',
      '/ai-catalog.json': 'http://127.0.0.1:34187',
      '/.well-known': 'http://127.0.0.1:34187',
    },
  },
  preview: {
    proxy: {
      '/api': 'http://127.0.0.1:34187',
      '/storage': 'http://127.0.0.1:34187',
      '/llms.txt': 'http://127.0.0.1:34187',
      '/ai-catalog.json': 'http://127.0.0.1:34187',
      '/.well-known': 'http://127.0.0.1:34187',
    },
  },
});
