# @ramal/web

Ramal Web: la landing prerenderizada en castellano (`/`) y en ingles (`/en/`), entrar y crear
cuenta, y el estado del servicio (`/status`). React Router 8 en modo framework, sin SSR (ADR 0015).

```sh
just web-dev        # Vite con HMR en http://localhost:5173 (proxy de la API a 127.0.0.1:8080)
just web-check      # lint, tipos, cobertura y build
just web-e2e        # Playwright contra el build, con la red de la API simulada
just e2e-stack      # Playwright contra la pila Docker por el gateway (la deja arriba: just down)
just web-docker-smoke  # imagen de produccion arrancada y probada
```

## Capas

| Carpeta              | Que hay                                                                      |
| -------------------- | ---------------------------------------------------------------------------- |
| `src/domain`         | Informe de salud, idioma y tema, datos de descarga y que instalador destacar |
| `src/application`    | Puertos y casos de uso (`checkApiHealth`, `signIn`, `signUp`, `loadRelease`) |
| `src/infrastructure` | HTTP sobre `@ramal/api-client`, localStorage, navegador, datos fijos         |
| `src/ui`             | Pantallas; reciben los puertos por `UiPortsContext`                          |
| `src/app`            | `root.tsx`, `routes.ts`, modulos de ruta finos y `composition-root.ts`       |

`eslint-plugin-boundaries` vigila quien importa a quien (architecture.md 5.2).

## Como se sirve

`just web-build` deja en `build/client` las dos landings (`index.html`, `en/index.html`) y el
fallback de la SPA (`__spa-fallback.html`). El nginx de la imagen (`nginx.conf`) sirve el fichero
si existe y si no el fallback; el HTML va con `no-store` y `/assets/` con `immutable`. En local,
`node scripts/preview-server.mjs` hace lo mismo en el puerto 4173.

## Primer corte

- El envio de entrar y crear cuenta va a un `AuthGateway` que contesta "no disponible todavia":
  la API aun no tiene auth. La validacion ya es la de `@ramal/domain`.
- Los datos de descarga son fijos (`infrastructure/releases`). Cuando exista
  `GET /v1/releases/latest`, cambia el adaptador en el composition root.
