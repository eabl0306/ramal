import { reactRouter } from '@react-router/dev/vite';
import { defineConfig } from 'vite';

const DEV_PORT = 5173;

/**
 * La API no manda CORS a proposito (ADR 0007): en Docker el gateway pone web y
 * API bajo un mismo origen, y con `pnpm dev` a secas lo hace este proxy. Son los
 * mismos prefijos que enruta docker/gateway.dev.conf.
 */
const API_TARGET = process.env['VITE_DEV_API_TARGET'] ?? 'http://127.0.0.1:8080';
const API_PREFIXES = ['/v1', '/health', '/openapi.json'];

const devProxy = Object.fromEntries(
  API_PREFIXES.map((prefix) => [prefix, { target: API_TARGET, changeOrigin: true }]),
);

export default defineConfig({
  plugins: [reactRouter()],
  server: { host: true, port: DEV_PORT, strictPort: true, proxy: devProxy },
});
