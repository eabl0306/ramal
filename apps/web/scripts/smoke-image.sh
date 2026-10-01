#!/usr/bin/env bash
# Prueba de humo de la imagen de produccion de la web. Construir no prueba nada:
# la imagen se arranca y se comprueba lo que sirve.
#
#   bash apps/web/scripts/smoke-image.sh [imagen]      (por defecto ramal-web:local)
#
# - /healthz responde y el HEALTHCHECK de la imagen la marca sana.
# - / y /en/ son las landings prerenderizadas, cada una en su idioma.
# - Una ruta de la SPA devuelve el fallback, no un 404.
# - Un asset que no existe es un 404, no la SPA.
# - El HTML va con no-store, los assets con immutable y las cabeceras de seguridad
#   salen tambien en las rutas que sirven contenido.
set -euo pipefail

image="${1:-ramal-web:local}"
name="ramal-web-smoke-$$"
port="${SMOKE_WEB_PORT:-18090}"
base="http://127.0.0.1:${port}"
headers="$(mktemp)"
body="$(mktemp)"
failures=0

cleanup() {
  docker rm -f "${name}" >/dev/null 2>&1 || true
  rm -f "${headers}" "${body}"
}
trap cleanup EXIT

fail() {
  echo "  FALLO: $1" >&2
  failures=$((failures + 1))
}

# Codigo de un GET; deja las cabeceras y el cuerpo en sus ficheros.
fetch() {
  curl -s -D "${headers}" -o "${body}" -w '%{http_code}' --max-time 5 "${base}$1" || echo 000
}

header() {
  grep -i "^$1:" "${headers}" | head -1 | cut -d' ' -f2- | tr -d '\r'
}

expect_page() {
  local path="$1" code="$2" text="$3"
  local got
  got="$(fetch "${path}")"
  [ "${got}" = "${code}" ] || { fail "${path} devolvio ${got}, esperaba ${code}"; return; }
  grep -q -- "${text}" "${body}" || fail "${path} no contiene: ${text}"
  echo "  ok ${path} (${got})"
}

docker run -d --name "${name}" -p "127.0.0.1:${port}:8080" "${image}" >/dev/null

for _ in $(seq 30); do
  [ "$(fetch /healthz)" = "200" ] && break
  sleep 1
done
[ "$(fetch /healthz)" = "200" ] || { docker logs "${name}" >&2; echo "smoke: /healthz no responde" >&2; exit 1; }

echo "smoke: ${image} en ${base}"
expect_page / 200 'Un cliente de Git para todos tus repositorios'
[ "$(header cache-control)" = "no-store" ] || fail "el HTML no va con no-store: $(header cache-control)"
[ "$(header x-frame-options)" = "DENY" ] || fail "falta X-Frame-Options en /"
[ "$(header x-content-type-options)" = "nosniff" ] || fail "falta X-Content-Type-Options en /"
expect_page /en/ 200 'A Git client for all your repositories'
expect_page /en 200 '<html lang="en"'
expect_page /status 200 '__reactRouterContext'
expect_page /ruta/que/no/existe 200 '__reactRouterContext'

asset="$(grep -o '/assets/[^"]*\.js' "${body}" | head -1)"
[ "$(fetch "${asset}")" = "200" ] || fail "${asset} no se sirve"
case "$(header cache-control)" in
  *immutable*) echo "  ok ${asset} (immutable)" ;;
  *) fail "${asset} no va con immutable: $(header cache-control)" ;;
esac
[ "$(fetch /assets/no-existe.js)" = "404" ] || fail "un asset que falta no es un 404"

for _ in $(seq 20); do
  status="$(docker inspect --format '{{.State.Health.Status}}' "${name}")"
  [ "${status}" = "healthy" ] && break
  sleep 1
done
[ "${status}" = "healthy" ] || fail "el HEALTHCHECK no la marca sana (${status})"

if [ "${failures}" -ne 0 ]; then
  docker logs --tail 50 "${name}" >&2
  echo "smoke: ${failures} comprobacion(es) fallaron." >&2
  exit 1
fi
echo "smoke: la imagen sirve lo que debe."
