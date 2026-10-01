#!/usr/bin/env bash
# Prueba de humo de la imagen de produccion de la API. Construir no prueba nada:
# la imagen se arranca y se comprueba su contrato.
#
#   bash apps/api/scripts/smoke-image.sh [imagen]      (por defecto ramal-api:local)
#
# Dos fases, con nombres y puertos propios para no chocar con la pila de compose:
#
#   1. Sin base de datos: /health/live responde 200 y /health/ready 503 degradado.
#      El proceso no muere por no tener base, que es lo que evita un bucle de
#      reinicios en el orquestador.
#   2. Con PostgreSQL 18: la API aplica las migraciones al arrancar, /health/ready
#      pasa a 200, el HEALTHCHECK de la imagen la marca sana y SIGTERM la para
#      con codigo 0.
#
# Lo arranca todo con `docker run` y lo borra al salir, falle o no.
set -euo pipefail

image="${1:-ramal-api:local}"
suffix="smoke-$$"
network="ramal-${suffix}"
db="ramal-db-${suffix}"
api_down="ramal-api-down-${suffix}"
api_ready="ramal-api-ready-${suffix}"
port_down="${SMOKE_PORT_DOWN:-18080}"
port_ready="${SMOKE_PORT_READY:-18081}"
body="$(mktemp)"

cleanup() {
  docker rm -f "${api_down}" "${api_ready}" "${db}" >/dev/null 2>&1 || true
  docker network rm "${network}" >/dev/null 2>&1 || true
  rm -f "${body}"
}
trap cleanup EXIT

fail() {
  echo "smoke: $1" >&2
  for container in "${api_down}" "${api_ready}"; do
    docker logs --tail 50 "${container}" 2>/dev/null || true
  done
  exit 1
}

# Codigo HTTP de un GET; deja el cuerpo en ${body}.
status_of() {
  curl -s -o "${body}" -w '%{http_code}' --max-time 5 "$1" || echo 000
}

# Repite un GET hasta que responda el codigo esperado o se acabe el plazo.
wait_for() {
  local url="$1" want="$2" seconds="$3"
  for _ in $(seq "${seconds}"); do
    [ "$(status_of "${url}")" = "${want}" ] && return 0
    sleep 1
  done
  return 1
}

echo "smoke: fase 1, ${image} sin base de datos"
docker run -d --name "${api_down}" -p "127.0.0.1:${port_down}:8080" \
  -e RAMAL__DATABASE__URL=postgres://ramal:ramal@127.0.0.1:1/ramal \
  "${image}" >/dev/null
wait_for "http://127.0.0.1:${port_down}/health/live" 200 30 || fail "/health/live no respondio 200"
grep -q '"status":"ok"' "${body}" || fail "/health/live no dice ok"
[ "$(status_of "http://127.0.0.1:${port_down}/health/ready")" = 503 ] \
  || fail "/health/ready deberia responder 503 sin base"
grep -q '"status":"degraded"' "${body}" || fail "/health/ready no dice degraded"
grep -q '"name":"database"' "${body}" || fail "/health/ready no lista la base"
[ "$(status_of "http://127.0.0.1:${port_down}/v1/ruta-que-no-existe")" = 404 ] \
  || fail "una ruta de /v1 que no existe deberia ser 404"
grep -q '"code":"route_not_found"' "${body}" || fail "el 404 no es problem+json"

echo "smoke: fase 2, ${image} con PostgreSQL 18"
docker network create "${network}" >/dev/null
docker run -d --name "${db}" --network "${network}" \
  -e POSTGRES_USER=ramal -e POSTGRES_PASSWORD=ramal -e POSTGRES_DB=ramal \
  postgres:18-alpine >/dev/null
docker run -d --name "${api_ready}" --network "${network}" -p "127.0.0.1:${port_ready}:8080" \
  -e "RAMAL__DATABASE__URL=postgres://ramal:ramal@${db}:5432/ramal" \
  "${image}" >/dev/null
wait_for "http://127.0.0.1:${port_ready}/health/ready" 200 90 \
  || fail "/health/ready no llego a 200 con la base arriba"
grep -q '"status":"ok"' "${body}" || fail "/health/ready no dice ok"

for _ in $(seq 40); do
  health="$(docker inspect -f '{{.State.Health.Status}}' "${api_ready}")"
  [ "${health}" = healthy ] && break
  sleep 1
done
[ "${health}" = healthy ] || fail "el HEALTHCHECK de la imagen no la marco sana (${health})"

docker stop --time 15 "${api_ready}" >/dev/null
exit_code="$(docker inspect -f '{{.State.ExitCode}}' "${api_ready}")"
[ "${exit_code}" = 0 ] || fail "SIGTERM deberia terminar con 0 y termino con ${exit_code}"

echo "smoke: la imagen cumple su contrato"
