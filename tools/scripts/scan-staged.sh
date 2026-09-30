#!/usr/bin/env bash
# Ultima red antes del commit: ni secretos ni ficheros enormes en el historial.
# Recibe los ficheros del index como argumentos (lefthook pasa {staged_files}).
#
#   bash tools/scripts/scan-staged.sh src/a.ts .env
#
# Limite de tamano configurable con MAX_STAGED_BYTES (por defecto 1 MiB). Un
# fichero de mas de un mega en un repositorio de codigo casi siempre es un
# binario, un volcado o un artefacto de build que se colo, y una vez en el
# historial pesa en cada clon para siempre.
set -uo pipefail

max_bytes="${MAX_STAGED_BYTES:-1048576}"
status=0

# Los patrones se arman por trozos para que este fichero no se delate a si mismo.
aws="AKIA"
gh="gh"
key="PRIVATE KEY"
# Patrones precisos: un acierto es casi con seguridad una credencial de verdad.
# Se aplican a TODOS los ficheros, tests incluidos: una clave de AWS no pinta
# nada en un fixture. Ramal habla con GitHub, asi que los tokens de GitHub son
# justo lo que mas facil se cuela en un fixture copiado de una sesion real.
strict_patterns=(
  "${aws}[0-9A-Z]{16}"
  "${gh}[pousr]_[A-Za-z0-9]{30,}"
  "${gh}ithub_pat_[A-Za-z0-9_]{30,}"
  "glpat-[A-Za-z0-9_-]{20,}"
  "-----BEGIN [A-Z ]*${key}-----"
  "xox[baprs]-[A-Za-z0-9-]{12,}"
)

# Heuristica de credencial en linea. Es util pero ruidosa, y su falso positivo
# mas comun son los tests: no se puede probar un login sin una contrasena
# literal en el fixture. Por eso no se aplica a los ficheros de test, donde los
# patrones precisos de arriba siguen vigentes.
loose_patterns=(
  "(secret|password|passwd|token|api_key|apikey)[[:space:]]*[:=][[:space:]]*['\"][^'\"]{12,}['\"]"
)

# Mismos criterios que TEST_PATTERNS en tools/checks/lib/scan.mjs.
is_test_file() {
  case "$1" in
    *.test.js | *.test.ts | *.test.jsx | *.test.tsx | *.test.mjs | *.test.cjs) return 0 ;;
    *.spec.js | *.spec.ts | *.spec.jsx | *.spec.tsx | *.spec.mjs | *.spec.cjs) return 0 ;;
    */test/* | */tests/* | */e2e/* | test/* | tests/* | e2e/*) return 0 ;;
    */tests.rs) return 0 ;;
  esac
  return 1
}

for file in "$@"; do
  [ -f "${file}" ] || continue

  case "${file##*/}" in
    # Las plantillas si se commitean: son la documentacion de que variables hay
    # y no llevan valores. Lo que esto salta es la comprobacion del NOMBRE: el
    # contenido se sigue escaneando mas abajo, asi que una clave de verdad dentro
    # de una plantilla se caza igual.
    *.example) ;;
    .env | .env.*)
      echo "  ${file}  los ficheros .env no se commitean (usa una plantilla .example)" >&2
      status=1
      ;;
  esac

  size="$(wc -c <"${file}" | tr -d ' ')"
  if [ "${size}" -gt "${max_bytes}" ]; then
    echo "  ${file}  pesa ${size} bytes, el limite son ${max_bytes}" >&2
    status=1
  fi

  scan_patterns=("${strict_patterns[@]}")
  if ! is_test_file "${file}"; then
    scan_patterns+=("${loose_patterns[@]}")
  fi

  for pattern in "${scan_patterns[@]}"; do
    # El -e es obligatorio: hay patrones que empiezan por guion y sin el grep
    # los toma por opciones, falla y deja pasar el secreto en silencio.
    if match="$(grep -IEn -m 1 -e "${pattern}" "${file}" 2>/dev/null)"; then
      echo "  ${file}:${match%%:*}  parece un secreto (patron: ${pattern})" >&2
      status=1
    fi
  done
done

if [ "${status}" -ne 0 ]; then
  echo "" >&2
  echo "scan-staged: saca eso del commit antes de seguir." >&2
fi
exit "${status}"
