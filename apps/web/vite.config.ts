import { reactRouter } from '@react-router/dev/vite';
import { defineConfig } from 'vite';

const DEV_PORT = 5173;

/**
 * La API no manda CORS a proposito (ADR 0007): en Docker el gateway pone web y
 * API bajo un mismo origen, y con `pnpm dev` a secas lo hace este proxy. Son los
 * mismos prefijos que enruta docker/gateway.dev.conf.
 */
const API_TARGET = process.env['VITE_DEV_API_TARGET'] ?? 'http://127.0.0.1:8080';

/**
 * Solo esta maquina por defecto: `host: true` abria el servidor de desarrollo a
 * toda la red local. El contenedor de compose pone VITE_DEV_HOST=0.0.0.0
 * (Dockerfile.dev), porque el gateway le llega por la red de Docker.
 */
const DEV_HOST = process.env['VITE_DEV_HOST'] ?? '127.0.0.1';
const API_PREFIXES = ['/v1', '/health', '/openapi.json'];

const devProxy = Object.fromEntries(
  API_PREFIXES.map((prefix) => [prefix, { target: API_TARGET, changeOrigin: true }]),
);

/**
 * Dependencias que Vite preempaqueta al arrancar. Sin la lista, las descubre en
 * la primera visita, recarga la pagina a mitad de la carga y los e2e contra la
 * pila de desarrollo (just e2e-stack) fallan la primera vez en cada contenedor.
 * Las de los paquetes compartidos se nombran a traves del paquete (`a > b`):
 * con pnpm no estan en los node_modules de la web.
 */
const PREBUNDLED = [
  '@ramal/ui > react-aria-components',
  '@ramal/ui > lucide-react',
  '@ramal/api-client > openapi-fetch',
  'zod',
];

export default defineConfig({
  plugins: [reactRouter()],
  optimizeDeps: { include: PREBUNDLED },
  server: {
    host: DEV_HOST,
    port: DEV_PORT,
    strictPort: true,
    proxy: devProxy,
    // Transforma la app al arrancar, no en la primera peticion.
    warmup: { clientFiles: ['./src/app/root.tsx', './src/app/routes/*.tsx'] },
  },
});
