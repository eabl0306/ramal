#!/bin/sh
# Ejecutable falso para los tests del runner: se cuelga hasta que lo matan por
# timeout. Esta versionado con el bit de ejecucion para que ningun test lo
# escriba en tiempo de ejecucion (ver runner/tests.rs).
exec sleep 30
