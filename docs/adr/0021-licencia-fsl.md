# ADR 0021: Licencia FSL con Apache 2.0 como licencia futura

- Estado: aceptado
- Fecha: 2026-10-01

## Contexto

Ramal es un producto comercial: el diseno trae un plan Ramal Pro, asientos de equipo y una
suscripcion. A la vez promete que los secretos de las cuentas Git van cifrados de extremo a extremo
(ADR 0012), y esa promesa solo se puede comprobar si el codigo del cliente se puede leer.

Hasta ahora el repositorio no tenia licencia, y la extension de VS Code se empaquetaba con
`--skip-license` porque la eleccion era del propietario. Las opciones que se miraron estan abajo.

## Decision

Todo el repositorio va con la **Functional Source License 1.1, con Apache 2.0 como licencia futura**
(identificador SPDX `FSL-1.1-ALv2`). El texto canonico de fsl.software esta en `LICENSE.md`, con el
aviso a nombre de Ezequiel Bahoque Lopez.

Lo que significa, en corto:

- Cualquiera puede leer, usar, modificar y redistribuir el codigo, salvo para un producto o
  servicio comercial que compita con Ramal.
- Cada version pasa sola a Apache 2.0 dos anos despues de publicarse.
- Apache 2.0 y no MIT como licencia futura porque trae concesion de patentes.

Donde se declara:

- `LICENSE.md` en la raiz. Prettier lo ignora: el texto tiene que seguir siendo el canonico.
- `"license": "FSL-1.1-ALv2"` en todos los `package.json` del workspace.
- `license = "FSL-1.1-ALv2"` en `[workspace.package]` de `Cargo.toml`, heredado por cada crate.
- La VSIX lleva `LICENSE.md`: el script `package` de la extension lo copia de la raiz antes de
  llamar a vsce. La copia no se versiona.
- Los paquetes de escritorio llevan `license` y `licenseFile` en `tauri.conf.json`.

`deny.toml` no anade `FSL-1.1-ALv2` a la lista de licencias permitidas: esa lista vigila lo que
entra de fuera, y los crates propios quedan fuera por `[licenses.private]` porque no se publican.

## Consecuencias

- El codigo se puede publicar cuando se quiera sin cambiar de licencia, y el cifrado de extremo a
  extremo se puede auditar.
- Nadie puede montar un clon comercial durante los dos primeros anos de cada version.
- La FSL no es una licencia de codigo abierto segun la OSI. Algun canal o comunidad puede
  tratarla como propietaria; Marketplace, Open VSX y Flathub la aceptan.
- Si algun dia se aceptan contribuciones de fuera, hace falta un CLA para poder seguir cambiando la
  licencia.
- Git es GPLv2. Llamarlo como programa aparte no obliga a nada, pero empaquetar MinGit en Windows
  (ADR 0009) exige incluir su licencia y ofrecer su codigo. Las fuentes IBM Plex van con licencia
  OFL y su aviso tiene que viajar con las apps. Las dos cosas son trabajo de la epica de release.
- Al ser un producto comercial, conviene que un abogado revise la eleccion antes del lanzamiento.

## Alternativas descartadas

- **Propietaria ("todos los derechos reservados").** Maximo control, pero nadie puede auditar el
  cifrado y se cierra la puerta a publicar el codigo sin un cambio de licencia.
- **Open core (clientes MIT o Apache, API cerrada).** Atrae contribuciones, pero cualquiera puede
  clonar los clientes y competir desde el primer dia.
- **GPL o AGPL.** Libre, pero obliga a quien lo use a publicar sus cambios y complica vender o
  cambiar de licencia si hay contribuciones externas.
- **FSL con MIT como licencia futura.** Igual de sencilla, pero sin concesion de patentes.
- **Business Source License 1.1.** El modelo del que sale la FSL, con un plazo de cambio de hasta
  cuatro anos y una concesion adicional que hay que redactar a mano.
