# ADR 0027: El producto se llama Git Rudder

- Estado: aceptado
- Fecha: 2026-10-02
- Sustituye la eleccion de dominio del [ADR 0023](0023-despliegue-en-apps-01-y-dominio.md)

## Contexto

El producto se llamo Ramal desde el primer commit. El 2026-10-02 Ezequiel decidio cambiarlo a
**Git Rudder**, antes de que exista cualquier publicacion: el repositorio es privado, no hay
instaladores, ni VSIX en Marketplace, ni imagen publicada, ni nada desplegado. Es el momento mas
barato para hacerlo, y el unico en el que no hay que mantener compatibilidad con el nombre viejo.

El nombre estaba en 4.498 sitios de 524 ficheros y en 18 rutas, y no en una sola forma: el nombre
visible, el prefijo de los crates, el scope de npm, los modulos de Rust, las claves de los ajustes
de VS Code, las variables de entorno, el usuario de PostgreSQL, el identificador de Flatpak, el
nombre del repositorio de GitHub y el dominio. Sin una tabla, cada sitio se habria renombrado a su
manera.

## Decision

El nombre visible es **Git Rudder**, con esa grafia (dos palabras, las dos en mayuscula). En
identificadores hay cuatro formas, una por convencion de cada sitio:

| Contexto                                                              | Forma                      | Ejemplo                            |
| --------------------------------------------------------------------- | -------------------------- | ---------------------------------- |
| Texto visible, titulos de ventana, categoria de comandos              | `Git Rudder`               | "Git Rudder: Open Graph"           |
| Crates, paquetes, repositorio, extension, nombres de fichero          | `git-rudder`               | `git-rudder-api-domain`            |
| Scope de npm                                                          | `@git-rudder/`             | `@git-rudder/repo-view`            |
| Modulos de Rust, upstreams de nginx, usuario y base de PostgreSQL     | `git_rudder`               | `git_rudder_core_domain`           |
| Comandos, vistas y ajustes de VS Code; claves de almacenamiento local | `gitRudder`                | `gitRudder.engine.path`            |
| Variables de entorno y constantes                                     | `GIT_RUDDER`               | `GIT_RUDDER__DATABASE__URL`        |
| Identificador de la extension (`publisher.name`)                      | `git-rudder.git-rudder`    | la URI de vuelta de OAuth lo usa   |
| Identificador de Tauri y de Flatpak                                   | `com.git_rudder.GitRudder` | un id de D-Bus no admite guiones   |
| Dominio                                                               | `git-rudder.com`           | y `api.git-rudder.com` para la API |

**El dominio es propio: `git-rudder.com`.** Ezequiel lo registra el. Esto cambia la respuesta del
[ADR 0023](0023-despliegue-en-apps-01-y-dominio.md), que habia elegido un subdominio del dominio
personal porque en ese momento no habia ninguno propio. Lo demas de ese ADR (la maquina `apps-01`,
el patron de despliegue, el enrutado y la vuelta atras) sigue igual, y la tabla de rutas de su
seccion "Dominio y enrutado" ya esta escrita con el dominio nuevo.

El dominio no se puede cambiar despues sin coste: la URL del actualizador queda grabada en cada
instalacion ([ADR 0016](0016-actualizaciones-y-descargas.md)), y el `iss` de los tokens
([ADR 0008](0008-autenticacion-con-tokens.md)), los enlaces de los correos, las URL de vuelta de
OAuth y el `type` de los errores tambien lo llevan. Por eso tiene que estar registrado y con sus
registros DNS antes de la primera build publica.

### Lo que no se renombra

- **El prefijo `--r-` de los tokens de CSS.** La `r` vale igual para Rudder. Cambiarlo tocaria
  cientos de declaraciones y las catorce maquetas del diseno sin ganar nada.
- **La marca del icono.** `apps/vscode/media/git-rudder.svg` dibuja una "R" con un nodo, y la "R"
  sigue valiendo. Si algun dia se quiere una marca de timon, es trabajo de diseno aparte.
- **El prefijo `repo-` de los `repoId`** y los scopes de commitlint: no llevan el nombre.
- **El comentario de la cabecera de `apps/api/migrations/20261001000000_initial_schema.sql`.** Dice
  "Ramal" y se queda. SQLx guarda la suma de cada migracion aplicada: editarla romperia las
  migraciones de cualquier base que ya la tenga y dejaria `/health/ready` en 503 reintentando.
  Una migracion que esta en `main` no se edita, ni por un comentario.

## Consecuencias

- La URL del repositorio cambia a `github.com/eabl0306/git-rudder`. GitHub redirige la vieja, pero
  los remotos locales conviene actualizarlos (`git remote set-url`).
- Las plataformas externas configuradas en la cuenta de Ezequiel (GitKraken, SonarCloud) ven el
  repositorio con el nombre nuevo; si alguna guarda la clave del proyecto por nombre, hay que
  reconectarla.
- El usuario y la base de datos de desarrollo pasan a `git_rudder`. El volumen de PostgreSQL que ya
  exista con la base `ramal` no sirve: `just down` lo borra y `just up` crea el nuevo.
- El proyecto de compose pasa a llamarse `git-rudder`, asi que los contenedores viejos
  (`ramal-db-1` y companhia) quedan huerfanos hasta que se borren a mano.
- El publisher `git-rudder` de Marketplace y Open VSX hay que reclamarlo cuando se publique la
  extension. Nadie habia reclamado `ramal` tampoco.
- Los diez briefs y las catorce maquetas de `docs/design/handoff/` llevan el nombre nuevo. Lo que
  no se toco son los dos documentos que escribio Claude Design (`CAMBIOS-pendientes.md` y la
  maqueta de correcciones): son su entrega y su registro de cambios.

## Alternativas descartadas

- **`rudder` a secas en los identificadores** (`rudder-api-domain`, `@rudder/ui`, `RUDDER__`). Mas
  corto de escribir, pero el repositorio se llamaria de una forma y el codigo de otra, y habria que
  recordar el apodo. Tambien choca de nombre con RudderStack, que publica `@rudderstack/*` y
  `rudder-sdk-js`; hoy no importa porque nada se publica, pero el dia que algo se publique si.
- **`gitrudder` en una palabra**, como `gitkraken`. Se lee peor en kebab-case y obliga a decidir
  caso por caso donde va el corte.
- **Dejar el nombre viejo en los identificadores y cambiar solo lo visible.** Es lo que hacen los
  productos que renombran tarde. Aqui no hay nada publicado, asi que no hay razon para cargar con
  dos nombres desde el primer dia.
