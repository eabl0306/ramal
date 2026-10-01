#!/usr/bin/env node
// Sirve build/client como lo hace el nginx de produccion (apps/web/nginx.conf):
// el fichero si existe, si no su index.html (la landing prerenderizada de /en/),
// y si no el fallback de la SPA. Bajo /assets/ un fichero que falta es un 404, no
// la SPA. Existe porque `vite preview` cae en index.html, que es la landing
// prerenderizada y no el fallback, y los e2e probarian algo que no se despliega.
//
//   node scripts/preview-server.mjs        (puerto 4173, o PORT)
import { createReadStream, statSync } from 'node:fs';
import { createServer } from 'node:http';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', 'build', 'client');
const PORT = Number(process.env.PORT ?? '4173');
const HOST = process.env.HOST ?? '127.0.0.1';
const FALLBACK = path.join(ROOT, '__spa-fallback.html');

const TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json',
  '.map': 'application/json',
  '.data': 'text/x-script',
  '.woff': 'font/woff',
  '.woff2': 'font/woff2',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.ico': 'image/x-icon',
};

function isFile(candidate) {
  try {
    return statSync(candidate).isFile();
  } catch {
    return false;
  }
}

/** try_files $uri $uri/index.html /__spa-fallback.html, sin salir nunca de ROOT. */
function resolve(urlPath) {
  const pathname = decodeURIComponent(urlPath.split('?')[0] ?? '/');
  const target = path.normalize(path.join(ROOT, pathname));
  if (target !== ROOT && !target.startsWith(`${ROOT}${path.sep}`)) return null;
  if (isFile(target)) return target;
  if (pathname.startsWith('/assets/')) return null;
  const index = path.join(target, 'index.html');
  return isFile(index) ? index : FALLBACK;
}

const server = createServer((request, response) => {
  const file = resolve(request.url ?? '/');
  if (file === null) {
    response.writeHead(404, { 'content-type': 'text/plain; charset=utf-8' });
    response.end('not found\n');
    return;
  }
  const type = TYPES[path.extname(file)] ?? 'application/octet-stream';
  response.writeHead(200, { 'content-type': type, 'cache-control': 'no-store' });
  createReadStream(file).pipe(response);
});

server.listen(PORT, HOST, () => {
  process.stdout.write(`preview: http://${HOST}:${String(PORT)} sirve ${ROOT}\n`);
});
