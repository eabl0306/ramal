# ADR 0016: Actualizaciones de escritorio y descargas

- Estado: aceptado
- Fecha: 2026-09-30

## Contexto

Ramal Desktop se instala en macOS, Windows y Linux, y tiene que actualizarse solo. La landing
ensena las descargas de la ultima version por sistema y arquitectura, con su tamano y sus
requisitos. La extension de VS Code tiene su propia version y se distribuye por las tiendas de
extensiones.

Hechos que condicionan la decision:

- El actualizador de Tauri verifica una firma propia, con un par de llaves independiente de la
  firma de codigo de macOS y Windows. **Perder la llave privada deja todas las instalaciones sin
  poder actualizarse nunca mas.**
- El actualizador acepta un manifiesto estatico (`latest.json`) o un endpoint dinamico con las
  variables `{{target}}`, `{{arch}}` y `{{current_version}}`, que responde 204 si no hay nada o 200
  con los datos. Admite varios endpoints como respaldo.
- **No todos los formatos se actualizan solos.** Tauri sabe actualizar el `.app` de macOS, los
  instaladores NSIS y MSI de Windows y la AppImage de Linux. No los `.deb` ni los `.rpm`. Y el
  manifiesto da un solo paquete de actualizacion por sistema y arquitectura, asi que en Windows
  solo uno de los dos instaladores puede actualizarse solo.
- **La URL del actualizador queda grabada en cada instalacion** y no se puede cambiar despues de la
  primera build publica.
- Tauri 2 necesita `webkit2gtk-4.1` en Linux: la base real es Ubuntu 22.04 y Debian 12, no la
  Ubuntu 20.04 que decia el diseno.
- Sin firma de codigo del sistema, macOS bloquea la app y Windows ensena el aviso de SmartScreen.

Las opciones eran solo GitHub Releases con un `latest.json` estatico, el manifiesto servido por la
API con los artefactos en GitHub Releases, o todo en infraestructura propia.

## Decision

Tres piezas: los artefactos en GitHub Releases, el manifiesto del actualizador servido por la API
de Ramal desde el primer dia, y la landing leyendo la misma API.

### Artefactos en GitHub Releases

- Un repositorio publico de publicaciones (`ramal-releases`) guarda los instaladores y los paquetes
  del actualizador. Asi las descargas son publicas aunque el repositorio del producto no lo sea, y
  el ancho de banda no cuesta.
- El workflow de publicacion compila la matriz con `tauri-action`: macOS arm64 y x64, mas un
  `.dmg` universal para quien no sabe que procesador tiene; Windows x64 y arm64; Linux x64 y arm64.
  Linux se compila sobre la base soportada mas antigua (Ubuntu 22.04), para que el minimo de glibc
  sea el mas bajo posible. El `.dmg` universal es solo de instalacion: el actualizador baja
  siempre el paquete de la arquitectura en la que corre la app.
- Al terminar, la CI registra la publicacion en la API. Se autentica con el token OIDC de GitHub
  Actions del workflow de publicacion (audiencia `ramal-api`, repositorio y workflow fijados en la
  configuracion), un reino de autenticacion aparte: no es un usuario y no puede hacer nada mas.
  Mientras no este, sirve un token de publicacion cuya API guarda solo el hash y compara en tiempo
  constante.

### El registro de una publicacion no se cree a ciegas

Quien tenga la credencial de publicacion decide lo que los usuarios descargan y ejecutan. Por eso
la API valida lo que registra la CI:

- Las URL de descarga tienen que empezar por
  `https://github.com/<propietario>/ramal-releases/releases/download/v{version}/`, con el
  propietario de la configuracion y la version de la propia publicacion. Los enlaces a tiendas
  (Marketplace, Open VSX, Flathub) son URL fijas del codigo.
- El comando de instalacion no es texto libre: es un valor de una lista (`brew_cask`, `winget`,
  `flatpak`) y la web escribe el comando desde una tabla fija. Un texto libre seria un comando que
  la landing invita a pegar en una terminal.
- La firma del actualizador se verifica al registrar, con la llave publica de minisign que la API
  tiene en su configuracion. Un paquete con una firma que no verifica no se registra.
- Las notas se guardan y se pintan como texto plano o Markdown saneado, sin HTML, en la web y en el
  dialogo de actualizacion de Desktop.

### El manifiesto lo sirve la API

En `tauri.conf.json`, dos endpoints por orden:

1. `https://ramal.dev/v1/desktop/updates/{{target}}/{{arch}}/{{current_version}}?channel=stable`,
   que responde 204 si no hay nada mas nuevo para ese destino o 200 con el formato que dicta Tauri
   (en snake_case, porque no es nuestro).
2. El `latest.json` estatico del repositorio de publicaciones, por si la API no responde.

Servirlo desde la API da lo que GitHub Releases solo no da:

- **Canales.** `stable` y `beta`; quien esta en beta recibe tambien las estables.
- **Despliegue gradual.** Cada publicacion lleva un porcentaje. La app manda en la cabecera
  `X-Ramal-Rollout-Bucket` un numero de 0 a 99 que calcula una vez a partir de una semilla
  aleatoria propia, distinta del `installationId`, y la API ofrece la version si ese numero cae por
  debajo del porcentaje. Sin cabecera, solo se ofrece al 100%. Asi la consulta del actualizador no
  lleva ningun identificador estable de la instalacion.
- **Retirada.** Una publicacion retirada deja de ofrecerse; es el interruptor de emergencia. El
  comparador de versiones es estrictamente "mayor que" y la API nunca ofrece una version igual o
  anterior a la instalada: retirar no baja de version a nadie, solo deja de ofrecer la mala hasta
  que salga una correccion.
- **Version minima soportada.** La API la devuelve con la ultima publicacion. Por debajo de ella,
  el motor deja de sincronizar con la nube y la interfaz ensena "Actualiza para seguir
  sincronizando"; todo lo local sigue funcionando ([ADR 0014](0014-usar-ramal-sin-cuenta.md)).

**El dominio tiene que ser el definitivo antes de la primera build publica**, porque esa URL ya no
se puede cambiar. `ramal.dev` es el que usa el diseno.

### Que formatos se actualizan solos

| Sistema | Formato                                              | Se actualiza | Como                                                                                                                                       |
| ------- | ---------------------------------------------------- | ------------ | ------------------------------------------------------------------------------------------------------------------------------------------ |
| macOS   | `.dmg` (instalacion) y `.app.tar.gz` (actualizacion) | Si           | Actualizador de Tauri                                                                                                                      |
| macOS   | Homebrew cask                                        | Si           | El cask se marca con `auto_updates`, porque la app se actualiza sola y brew no debe pelearse con ella                                      |
| Windows | NSIS                                                 | Si           | Actualizador de Tauri. Es el formato de actualizacion de Windows                                                                           |
| Windows | MSI                                                  | No           | Para despliegues gestionados por un equipo de IT, que lo actualiza con sus herramientas. La tarjeta lo dice                                |
| Windows | winget                                               | Si           | El manifiesto de winget apunta al instalador NSIS                                                                                          |
| Linux   | AppImage                                             | Si           | Actualizador de Tauri                                                                                                                      |
| Linux   | `.deb` y `.rpm`                                      | No           | La app consulta `GET /v1/releases/latest` y ensena "Hay una version nueva" con el enlace. Un repositorio APT o RPM queda para mas adelante |
| Linux   | Flathub                                              | Si           | Lo actualiza Flatpak                                                                                                                       |

La tarjeta "Windows Portable (.zip)" del diseno se quita: el empaquetador de Tauri no la produce y
nunca se actualizaria sola.

La busqueda de actualizaciones se puede apagar en los ajustes, y no necesita cuenta de Ramal
([ADR 0014](0014-usar-ramal-sin-cuenta.md)).

### La landing lee la misma API

`GET /v1/releases/latest?product=desktop` (y `product=vscode`, que tiene su propia version) da por
sistema los artefactos, su arquitectura, tamano, sha256 y requisitos. La landing lo lee al
prerenderizar y lo refresca en el cliente ([ADR 0015](0015-framework-web-con-prerender.md)). Las
etiquetas visibles ("Apple Silicon (.dmg)") las pone la web desde `@ramal/i18n`, no la base de
datos. Los artefactos del actualizador no aparecen en la pagina de descargas.

### Firmas

- **Actualizador:** par de llaves de `tauri signer`. La publica va en `tauri.conf.json`; la privada,
  en un secreto de la CI con acceso restringido y con una copia fuera de linea.
- **macOS:** Developer ID y notarizacion.
- **Windows:** Authenticode, con Azure Trusted Signing o un certificado OV o EV.

Las dos firmas del sistema tienen coste y tramites; se empiezan pronto porque bloquean la beta
publica.

### La extension de VS Code

`vsce publish --target` al Marketplace y `ovsx publish --target` a Open VSX, en el mismo workflow
de publicacion que escritorio, con un VSIX por plataforma
([ADR 0010](0010-un-motor-para-escritorio-y-vscode.md)). Las tiendas se encargan de actualizarla.
El identificador de editor `ramal` se reserva cuanto antes en las dos, porque el diseno ya usa
`ramal.ramal`.

## Consecuencias

A favor:

- Canales, despliegue gradual y retirada desde la primera version, sin infraestructura de
  descargas propia.
- La landing y el actualizador leen de la misma fuente: no pueden contradecirse.
- Si la API cae, las apps siguen actualizandose por el `latest.json` estatico.

En contra:

- **Perder la llave del actualizador deja todas las instalaciones sin actualizaciones para
  siempre.** Copia fuera de linea obligatoria.
- **La URL del actualizador es para siempre.** Un error de dominio en la primera build publica no
  tiene arreglo.
- Los usuarios de `.deb` y `.rpm` no se actualizan solos hasta que exista un repositorio de
  paquetes.
- Firmar en macOS y Windows cuesta dinero y tiempo, y la notarizacion alarga cada publicacion.
- La base de Linux es Ubuntu 22.04 y Debian 12. Quien tenga algo mas viejo no puede usar Ramal
  Desktop, aunque si la extension de VS Code.
- La consulta del actualizador lleva un numero de 0 a 99, tambien sin cuenta. No identifica la
  instalacion, pero es un dato mas que sale del equipo, y la busqueda se puede apagar.
- Quien instale el MSI no recibe actualizaciones automaticas.
- La migracion a Tauri 3, que pasa Linux a GTK4, tocara el empaquetado de Linux.

## Alternativas descartadas

**Solo GitHub Releases con `latest.json` estatico.** Cero infraestructura, pero sin canales, sin
despliegue gradual y sin interruptor de emergencia, y con la URL de GitHub grabada para siempre en
cada instalacion.

**Todo en infraestructura propia (API mas almacenamiento de objetos o CDN).** Control total, a
cambio del mayor coste y trabajo, para servir ficheros que GitHub sirve gratis. Si un dia hace
falta, cambia la URL de cada artefacto en el registro de publicaciones, no la del actualizador.

**Mac App Store y Microsoft Store como canal principal.** El sandbox de la Mac App Store limita
lanzar ejecutables como git, que es la base del motor ([ADR 0009](0009-motor-git-hibrido.md)).

**Mantener la version portable de Windows.** No la produce Tauri y no se actualizaria nunca.
