#!/usr/bin/env bash
# Prueba de humo de la pila compuesta: comprueba que por el gateway se llega a
# lo que tiene que llegar. Viene de un fallo real en Gastonomo: el gateway solo
# enrutaba /health y la web, asi que todas las rutas de /v1 acababan en el
# servidor de la web. Los tests de cada imagen por separado pasaban igual,
# porque ninguno miraba la composicion.
#
#   bash tools/scripts/smoke-stack.sh [base_url]
#
# La senal que mas importa: una respuesta text/html en una ruta de la API
# significa que el gateway mando la peticion a la web en vez de a la API. Cada
# prefijo nuevo de la API necesita su `location` en docker/gateway.dev.conf y
# una sonda aqui.
set -euo pipefail

base="${1:-http://127.0.0.1:${GATEWAY_PORT:-8090}}"
failures=0

say() { printf '  %-52s %s\n' "$1" "$2"; }

fail() {
  say "$1" "FALLO: $2"
  failures=$((failures + 1))
}

# Devuelve "codigo|content-type" y deja el cuerpo en el fichero indicado.
request() {
  local out="$1" method="$2" path="$3"
  shift 3
  curl -sS -o "${out}" -w '%{http_code}|%{content_type}' \
    -X "${method}" "${base}${path}" --max-time 20 "$@" || echo '000|error'
}

expect() {
  local label="$1" want_code="$2" want_type="$3" got="$4"
  local code="${got%%|*}" ctype="${got#*|}"
  if [ "${code}" != "${want_code}" ]; then
    fail "${label}" "esperaba ${want_code}, llego ${code} (${ctype})"
    return 1
  fi
  case "${ctype}" in
    *"${want_type}"*) say "${label}" "ok (${code} ${want_type})" ;;
    *) fail "${label}" "esperaba ${want_type}, llego ${ctype}" ; return 1 ;;
  esac
}

body=$(mktemp) ; trap 'rm -f "${body}"' EXIT

echo "Prueba de humo de la pila en ${base}"

expect "GET /health/live" 200 json \
  "$(request "${body}" GET /health/live)" || true

expect "GET /health/ready" 200 json \
  "$(request "${body}" GET /health/ready)" || true

# La sonda clave, y no depende de que endpoints existan: una ruta de /v1 que no
# existe la tiene que contestar la API con problem+json. Si llega text/html, el
# gateway no enruta /v1 hacia la API.
expect "GET /v1/ruta-que-no-existe" 404 problem+json \
  "$(request "${body}" GET /v1/ruta-que-no-existe)" || true

expect "GET /openapi.json" 200 json \
  "$(request "${body}" GET /openapi.json)" || true

expect "GET / (la web)" 200 text/html \
  "$(request "${body}" GET /)" || true

echo
if [ "${failures}" -ne 0 ]; then
  echo "prueba de humo: ${failures} comprobacion(es) fallaron." >&2
  exit 1
fi
echo "prueba de humo: la pila responde lo que debe."
