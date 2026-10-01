# syntax=docker/dockerfile:1

# Imagen de PRODUCCION de la web: el build estatico de React Router servido por
# nginx sin root (ADR 0007 y 0015). No lleva node ni codigo fuente.
#
#   docker build -f apps/web/Dockerfile -t ramal-web:local .     (just web-docker)
#
# El contexto es la RAIZ del repo, porque el lockfile y los paquetes
# compartidos viven alli. Que entra lo decide apps/web/Dockerfile.dockerignore.

# Va a la par que .nvmrc: se suben juntos.
ARG NODE_VERSION=24

FROM node:${NODE_VERSION}-alpine AS base
# corepack instala el pnpm de `packageManager` (package.json) sin preguntar.
ENV PNPM_HOME=/pnpm \
    PATH=/pnpm:$PATH \
    COREPACK_ENABLE_DOWNLOAD_PROMPT=0
RUN corepack enable
WORKDIR /repo

# Los package.json de TODO el workspace, sin codigo. `--frozen-lockfile` necesita
# ver cada paquete que nombra el lockfile, y copiarlos uno a uno se queda viejo
# en cuanto alguien anade un paquete. Esta etapa se rehace con cualquier cambio,
# pero su resultado solo cambia con los manifiestos, y la capa de install queda
# en cache.
FROM base AS manifests
COPY . /src
RUN mkdir /manifests \
 && cd /src \
 && find . -name package.json -not -path '*/node_modules/*' | tar -cf - -T - | tar -xf - -C /manifests \
 && cp pnpm-lock.yaml pnpm-workspace.yaml /manifests/

FROM base AS deps
COPY --from=manifests /manifests/ ./
RUN --mount=type=cache,id=pnpm-store,target=/pnpm/store \
    pnpm install --frozen-lockfile --filter "@ramal/web..."

FROM deps AS build
COPY packages/ packages/
COPY apps/web/ apps/web/
# Vacia es el mismo origen: el gateway sirve la web y la API juntas.
ARG VITE_API_URL=
ENV VITE_API_URL=${VITE_API_URL}
RUN pnpm --filter @ramal/web build

FROM nginx:alpine AS runtime
# La configuracion es completa (no un server de conf.d) para correr sin root; el
# default.conf de la imagen sobra y sus scripts de arranque lo saltan si no esta.
RUN rm -f /etc/nginx/conf.d/default.conf
COPY apps/web/nginx.conf /etc/nginx/nginx.conf
COPY --from=build --chown=nginx:nginx /repo/apps/web/build/client /usr/share/nginx/html

USER nginx
EXPOSE 8080

HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --quiet --tries=1 --spider http://127.0.0.1:8080/healthz || exit 1

CMD ["nginx", "-g", "daemon off;"]
