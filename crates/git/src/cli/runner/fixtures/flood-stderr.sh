#!/bin/sh
# Ejecutable falso para los tests del runner: inunda stderr con 200000 bytes y
# sale con 3. Esta versionado con el bit de ejecucion para que ningun test lo
# escriba en tiempo de ejecucion (ver runner/tests.rs).
head -c 200000 /dev/zero | tr '\0' 'x' >&2
exit 3
