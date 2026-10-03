# ADR 0016: Actualizaciones de escritorio y descargas

- Estado: aceptado
- Fecha: 2026-09-30
- Revisado: 2026-10-01. Ratificado (matriz de plataformas, sin version portable, MSI sin
  actualizacion automatica) con cuatro correcciones: el motivo real de que MSI, deb y rpm no se
  actualicen solos, `{{bundle_type}}` en la URL del actualizador, el dominio definitivo y las
  publicaciones en el propio repositorio `eabl0306/git-rudder`. Lo que cambio esta al final, en
  [Cambios del 2026-10-01](#cambios-del-2026-10-01).

## Contexto

Git Rudder Desktop se instala en macOS, Windows y Linux, y tiene que actualizarse solo. La landing
ensena las descargas de la ultima version por sistema y arquitectura, con su tamano y sus
requisitos. La extension de VS Code tiene su propia version y se distribuye por las tiendas de
extensiones.

Hechos que condicionan la decision:

- El actualizador de Tauri verifica una firma propia, con un par de llaves independiente de la
  firma de codigo de macOS y Windows. **Perder la llave privada deja todas las instalaciones sin
  poder actualizarse nunca mas.**
- El actualizador acepta un manifiesto estatico (`latest.json`) o un endpoint dinamico con las
  variables `{{target}}`, `{{arch}}`, `{{current_version}}` y `{{bundle_type}}`, que responde 204
  si no hay nada o 200 con los datos. Admite varios endpoints como respaldo.
- **El actualizador sabe que formato esta instalado.** Desde `tauri-plugin-updater` 2.10 instala
  el `.app` de macOS, NSIS y MSI en Windows, y AppImage, `.deb` y `.rpm` en Linux, y manda el
  formato en `{{bundle_type}}` (`app`, `nsis`, `msi`, `appimage`, `deb` o `rpm`). En un
  `latest.json` busca primero la clave `{os}-{arch}-{formato}` y despues `{os}-{arch}`. Instalar un
  MSI de toda la maquina, un `.deb` o un `.rpm` pide permisos de administrador (el UAC de Windows,
  o la contrasena de root con pkexec, zenity o sudo).
- **La URL del actualizador queda grabada en cada instalacion** y no se puede cambiar despues de la
  primera build publica.
- Tauri 2 necesita `webkit2gtk-4.1` en Linux: la base real es Ubuntu 22.04 y Debian 12, no la
  Ubuntu 20.04 que decia el diseno.
- Sin firma de codigo del sistema, macOS bloquea la app y Windows ensena el aviso de SmartScreen.
- **GitHub no cobra los minutos de los runners estandar en un repositorio publico**, macOS
  incluido. En uno privado se pagan, y una publicacion completa (escritorio con toda la matriz mas
  los ocho VSIX) gasta del orden de 55 minutos de macOS, 60 de Windows y 55 de Linux.

Las opciones eran solo GitHub Releases con un `latest.json` estatico, el manifiesto servido por la
API con los artefactos en GitHub Releases, o todo en infraestructura propia.

## Decision

Tres piezas: los artefactos en GitHub Releases, el manifiesto del actualizador servido por la API
de Git Rudder desde el primer dia, y la landing leyendo la misma API.

### Artefactos en GitHub Releases del repositorio del producto

- Las publicaciones se suben a GitHub Releases del **propio repositorio del producto,
  `eabl0306/git-rudder`**, en la cuenta personal de Ezequiel. No hay repositorio aparte de
  publicaciones ni organizacion de GitHub: el plan de pago de Ezequiel y las herramientas que usa
  (GitKraken, SonarCloud) estan configurados en `eabl0306`.
- **El repositorio es privado hasta que abre la beta, y se hace publico entonces.** La primera build
  publica sale con el repositorio ya publico. Publico, los runners estandar de GitHub no gastan
  minutos y el cifrado de extremo a extremo se puede auditar
  ([ADR 0012](0012-boveda-de-extremo-a-extremo.md)); la FSL ya impide los clones comerciales
  ([ADR 0021](0021-licencia-fsl.md)).
- El workflow de publicacion compila la matriz con `tauri-action`: macOS arm64 y x64, mas un
  `.dmg` universal para quien no sabe que procesador tiene; Windows x64 y arm64; Linux x64 y arm64.
  Linux se compila sobre la base soportada mas antigua (Ubuntu 22.04), para que el minimo de glibc
  sea el mas bajo posible. El `.dmg` universal es solo de instalacion: el actualizador baja
  siempre el paquete de la arquitectura en la que corre la app.
- **Windows arm64 y Linux arm64 salen como beta** hasta que su E2E nativo pase en runners arm
  (`windows-11-arm` y `ubuntu-22.04-arm`). La matriz va con `fail-fast: false`: si un destino arm64
  falla, la publicacion sale sin el y la tarjeta lo dice.
- **Windows arm64 solo lleva NSIS**: el MSI de Tauri (WiX) no se compila para arm64.
- Al terminar, la CI registra la publicacion en la API. Se autentica con el token OIDC de GitHub
  Actions del workflow de publicacion (audiencia `git-rudder-api`, repositorio y workflow fijados en la
  configuracion), un reino de autenticacion aparte: no es un usuario y no puede hacer nada mas.
  Mientras no este, sirve un token de publicacion cuya API guarda solo el hash y compara en tiempo
  constante.

### El registro de una publicacion no se cree a ciegas

Quien tenga la credencial de publicacion decide lo que los usuarios descargan y ejecutan. Por eso
la API valida lo que registra la CI:

- Las URL de descarga tienen que empezar por
  `https://github.com/<propietario>/<repositorio>/releases/download/v{version}/`, con el
  propietario y el repositorio de la configuracion (`eabl0306/git-rudder`) y la version de la propia
  publicacion. Los enlaces a tiendas (Marketplace, Open VSX, Flathub) son URL fijas del codigo.
- El comando de instalacion no es texto libre: es un valor de una lista (`brew_cask`, `winget`,
  `flatpak`) y la web escribe el comando desde una tabla fija. Un texto libre seria un comando que
  la landing invita a pegar en una terminal.
- La firma del actualizador se verifica al registrar, con la llave publica de minisign que la API
  tiene en su configuracion. Un paquete con una firma que no verifica no se registra.
- Las notas se guardan y se pintan como texto plano o Markdown saneado, sin HTML, en la web y en el
  dialogo de actualizacion de Desktop.

### El manifiesto lo sirve la API

En `tauri.conf.json`, dos endpoints por orden:

1. `https://api.git-rudder.com/v1/desktop/updates/{{target}}/{{arch}}/{{current_version}}?channel=stable&bundle={{bundle_type}}`,
   que responde 204 si no hay nada mas nuevo para ese destino y ese formato, o 200 con el formato
   que dicta Tauri (en snake_case, porque no es nuestro). **`{{bundle_type}}` va en la URL desde la
   primera build publica**: como la URL no se puede cambiar, es lo que permite a la API responder
   distinto a cada formato (hoy, 204 a MSI, `.deb` y `.rpm`) sin tocar las apps instaladas. Un
   `bundle` que falte o que no se conozca recibe 204: no ofrecer nada es mejor que ofrecer un
   paquete que no es el suyo.
2. El `latest.json` estatico de las publicaciones de `eabl0306/git-rudder`, por si la API no responde.
   **Solo lleva claves por formato** (`darwin-aarch64-app`, `windows-x86_64-nsis`,
   `linux-x86_64-appimage`...) y ninguna generica: con una clave `{os}-{arch}`, un MSI o un `.deb`
   caerian en ella e intentarian instalar un paquete que no es el suyo.

Servirlo desde la API da lo que GitHub Releases solo no da:

- **Canales.** `stable` y `beta`; quien esta en beta recibe tambien las estables.
- **Despliegue gradual.** Cada publicacion lleva un porcentaje. La app manda en la cabecera
  `X-Git Rudder-Rollout-Bucket` un numero de 0 a 99 que calcula una vez a partir de una semilla
  aleatoria propia, distinta del `installationId`, y la API ofrece la version si ese numero cae por
  debajo del porcentaje. Sin cabecera, solo se ofrece al 100%. Asi la consulta del actualizador no
  lleva ningun identificador estable de la instalacion.
- **Retirada.** Una publicacion retirada deja de ofrecerse; es el interruptor de emergencia. El
  comparador de versiones es estrictamente "mayor que" y la API nunca ofrece una version igual o
  anterior a la instalada: retirar no baja de version a nadie, solo deja de ofrecer la mala hasta
  que salga una correccion.
- **Version minima soportada.** La API la devuelve con la ultima publicacion. Por debajo de ella,
  el motor deja de sincronizar con la nube y la interfaz ensena "Actualiza para seguir
  sincronizando"; todo lo local sigue funcionando ([ADR 0014](0014-usar-git-rudder-sin-cuenta.md)).

**El dominio es `api.git-rudder.com`** ([ADR 0023](0023-despliegue-en-apps-01-y-dominio.md)), y
tiene que estar sirviendo antes de la primera build publica, porque esa URL ya no se puede cambiar.

### Que formatos se actualizan solos

| Sistema | Formato                                              | Se actualiza    | Como                                                                                                                                                                                                                                                         |
| ------- | ---------------------------------------------------- | --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| macOS   | `.dmg` (instalacion) y `.app.tar.gz` (actualizacion) | Si              | Actualizador de Tauri                                                                                                                                                                                                                                        |
| macOS   | Homebrew cask                                        | Si              | El cask se marca con `auto_updates`, porque la app se actualiza sola y brew no debe pelearse con ella                                                                                                                                                        |
| Windows | NSIS (x64 y arm64)                                   | Si              | Actualizador de Tauri. Es el formato de actualizacion de Windows                                                                                                                                                                                             |
| Windows | MSI (solo x64)                                       | No, a proposito | Para despliegues gestionados por un equipo de IT, que lo actualiza con sus herramientas. Se instala para toda la maquina y actualizarlo pide administrador, que es justo lo que en una empresa gestionada no hay. La tarjeta lo dice. Sale en el lanzamiento |
| Windows | winget                                               | Si              | El manifiesto de winget apunta al instalador NSIS                                                                                                                                                                                                            |
| Linux   | AppImage                                             | Si              | Actualizador de Tauri                                                                                                                                                                                                                                        |
| Linux   | `.deb` y `.rpm`                                      | No, a proposito | El actualizador podria instalarlos, pero pidiendo la contrasena de root. La app consulta `GET /v1/releases/latest` y ensena "Hay una version nueva" con el enlace. Un repositorio APT o RPM queda para mas adelante                                          |
| Linux   | Flathub                                              | Si              | Lo actualiza Flatpak. Queda para despues del lanzamiento: Flathub exige compilar desde el codigo en su infraestructura y su sandbox complica git y ssh-agent                                                                                                 |

Que MSI, `.deb` y `.rpm` no se actualicen solos **es una decision de producto, no un limite de
Tauri**. Si algun dia cambia, cambia la respuesta de la API para ese `bundle`, no la app.

La tarjeta "Windows Portable (.zip)" del diseno se quita: el empaquetador de Tauri no la produce,
no seria portable de verdad (los datos van a `%APPDATA%`), tendria que llevar MinGit al lado, nunca
se actualizaria sola y su reputacion de SmartScreen iria aparte.

La busqueda de actualizaciones se puede apagar en los ajustes, y no necesita cuenta de Git Rudder
([ADR 0014](0014-usar-git-rudder-sin-cuenta.md)).

### La landing lee la misma API

`GET /v1/releases/latest?product=desktop` (y `product=vscode`, que tiene su propia version) da por
sistema los artefactos, su arquitectura, tamano, sha256 y requisitos. La landing lo lee al
prerenderizar y lo refresca en el cliente ([ADR 0015](0015-framework-web-con-prerender.md)). Las
etiquetas visibles ("Apple Silicon (.dmg)") las pone la web desde `@git-rudder/i18n`, no la base de
datos. Los artefactos del actualizador no aparecen en la pagina de descargas.

### Firmas

- **Actualizador:** par de llaves de `tauri signer`. La publica va en `tauri.conf.json`; la privada,
  en un secreto de la CI con acceso restringido y con una copia fuera de linea.
- **macOS:** Developer ID y notarizacion.
- **Windows:** Authenticode, con Azure Artifact Signing (antes Trusted Signing) o un certificado
  OV. EV ya no se salta el aviso de SmartScreen, y ninguna opcion da reputacion instantanea.

Las dos firmas del sistema tienen coste y tramites; se empiezan pronto porque bloquean la beta
publica. La via de cada una depende de si Ezequiel firma como persona o como empresa, que esta
sin decidir.

### La extension de VS Code

`vsce publish --target` al Marketplace y `ovsx publish --target` a Open VSX, en el mismo workflow
de publicacion que escritorio, con los ocho VSIX por plataforma
([ADR 0010](0010-un-motor-para-escritorio-y-vscode.md)). Las tiendas se encargan de actualizarla.
El identificador de editor `git-rudder` se reserva cuanto antes en las dos, porque el diseno ya usa
`git-rudder.git-rudder`. Al Marketplace se publica con Entra ID: los tokens globales de Azure DevOps dejan de
funcionar el 2026-12-01.

## Consecuencias

A favor:

- Canales, despliegue gradual y retirada desde la primera version, sin infraestructura de
  descargas propia.
- La landing y el actualizador leen de la misma fuente: no pueden contradecirse.
- Si la API cae, las apps siguen actualizandose por el `latest.json` estatico.
- Con `{{bundle_type}}` en la URL, la API puede cambiar lo que ofrece a cada formato sin publicar
  una version nueva.
- Con el repositorio publico, la CI de publicacion no gasta minutos y cualquiera puede comprobar el
  cifrado del cliente.

En contra:

- **Perder la llave del actualizador deja todas las instalaciones sin actualizaciones para
  siempre.** Copia fuera de linea obligatoria.
- **La URL del actualizador es para siempre**, y la del `latest.json` de respaldo lleva
  `eabl0306/git-rudder` dentro. Renombrar o transferir el repositorio depende de las redirecciones de
  GitHub, que se pierden si alguien reutiliza el nombre viejo.
- Los usuarios de MSI, `.deb` y `.rpm` no se actualizan solos.
- **Mientras el repositorio sea privado no hay builds publicas**, y su CI gasta los minutos del plan
  de pago de Ezequiel.
- **Hacer publico el repositorio no tiene vuelta atras**: las copias quedan. Antes hay que pasar
  gitleaks por todo el historial, revisar issues y documentos por si hay algo que no deba verse,
  proteger las ramas, exigir aprobacion para los workflows de PR de forks, no usar
  `pull_request_target` con checkout del PR, dejar los secretos de firma en un entorno `release`
  con aprobacion, no usar runners propios y decir en un CONTRIBUTING que no se aceptan PR hasta
  tener un CLA (ADR 0021).
- Windows arm64 y Linux arm64 se prueban solo en la CI, sin hardware propio, y salen como beta.
- Firmar en macOS y Windows cuesta dinero y tiempo, y la notarizacion alarga cada publicacion.
- La base de Linux es Ubuntu 22.04 y Debian 12. Quien tenga algo mas viejo no puede usar Git Rudder
  Desktop, aunque si la extension de VS Code.
- La consulta del actualizador lleva un numero de 0 a 99, tambien sin cuenta. No identifica la
  instalacion, pero es un dato mas que sale del equipo, y la busqueda se puede apagar.
- La migracion a Tauri 3, que pasa Linux a GTK4, tocara el empaquetado de Linux.

## Alternativas descartadas

**Solo GitHub Releases con `latest.json` estatico.** Cero infraestructura, pero sin canales, sin
despliegue gradual y sin interruptor de emergencia, y con la URL de GitHub grabada para siempre en
cada instalacion.

**Todo en infraestructura propia (API mas almacenamiento de objetos o CDN).** Control total, a
cambio del mayor coste y trabajo, para servir ficheros que GitHub sirve gratis. Si un dia hace
falta, cambia la URL de cada artefacto en el registro de publicaciones, no la del actualizador.

**Un repositorio publico aparte solo para publicaciones (`git-rudder-releases`).** Era la decision de la
primera version de este ADR, para que las descargas fueran publicas con el producto privado. Con el
producto publico desde la beta solo anade un token entre repositorios y otro sitio que vigilar.

**Una organizacion de GitHub para el producto.** Sacaria `eabl0306` de la URL de respaldo, pero el
plan de pago y las herramientas de Ezequiel estan en su cuenta personal, y moverlos cuesta mas de
lo que da.

**Actualizar MSI, `.deb` y `.rpm` con el actualizador.** Ya se puede, pero pide administrador en
cada actualizacion, y en una empresa gestionada el MSI lo actualiza IT con sus herramientas.

**Mac App Store y Microsoft Store como canal principal.** El sandbox de la Mac App Store limita
lanzar ejecutables como git, que es la base del motor ([ADR 0009](0009-motor-git-hibrido.md)).

**Mantener la version portable de Windows.** No la produce Tauri, no es portable de verdad y no se
actualizaria nunca.

## Cambios del 2026-10-01

Se ratifica la decision y se corrige lo que los hechos desmintieron o lo que se decidio despues:

- **Motivo de MSI, `.deb` y `.rpm`.** La primera version decia que Tauri no sabia actualizarlos y
  que el manifiesto solo daba un paquete por sistema y arquitectura. Desde `tauri-plugin-updater`
  2.10 si sabe, y distingue el formato con `{{bundle_type}}`. Que no se actualicen solos pasa a ser
  una decision de producto.
- **`{{bundle_type}}` en la URL del actualizador** desde la primera build publica, y el
  `latest.json` de respaldo solo con claves por formato.
- **Dominio.** El del diseno era de un tercero. El actualizador vive en `api.git-rudder.com`
  ([ADR 0023](0023-despliegue-en-apps-01-y-dominio.md), con el dominio propio que eligio el
  [ADR 0027](0027-renombrado-a-git-rudder.md)).
- **Donde se publican los artefactos.** Ya no hay `git-rudder-releases`: se publica en
  `eabl0306/git-rudder`, que es privado hasta la beta y publico desde entonces.
- **Matriz.** Se ratifica entera; Windows arm64 y Linux arm64 salen como beta hasta que su E2E pase
  en runners arm, y Windows arm64 solo lleva NSIS.
- **MSI x64** con la etiqueta "sin actualizaciones automáticas", en el lanzamiento, porque los
  equipos entran en el MVP ([ADR 0026](0026-planes-y-precios.md)).
