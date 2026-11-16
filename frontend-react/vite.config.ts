import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// En desarrollo el proxy envía /api al backend Rust local (34187).
// En producción nginx sirve el SPA y hace proxy de /api al API.
export default defineConfig({
  plugins: [react()],
  server: {
    proxy: { '/api': 'http://127.0.0.1:34187', '/storage': 'http://127.0.0.1:34187' },
  },
});
