# ADR 0017: Internacionalizacion compartida con claves tipadas

- Estado: aceptado
- Fecha: 2026-09-30
- Revisado: 2026-10-01, para alinearlo con el codigo del primer corte; 2026-10-23, con las
  reglas de los textos repetidos y de los recuentos

## Contexto

Las tres apps de Git Rudder estan en castellano y en ingles. Los prototipos traen dos diccionarios
planos, `D` (466 claves, en `git-rudder-data.js`) y `W` (148, el de la web), con tuplas
`[castellano, ingles]` y marcadores `{x}`. El analisis de esos diccionarios encontro lo que hay
que corregir antes de que llegue a produccion:

- **No hay plurales.** Trece claves con un recuento pintan "1 archivos", "1 repositorios".
- **Los marcadores tienen nombres de una letra con varios significados**: `{n}` es un recuento en
  unas claves, un nombre en otras y un numero de PR en otra.
- **Hay textos construidos troceando otras traducciones** (`tr('mCopySha').replace('SHA', ...)`),
  que dan "Copiar Nombre" con la mayuscula mal.
- **Hay glifos y atajos dentro de los textos**: marcas de check y de estado, y el atajo de Mac
  Cmd+Enter escrito en el texto, que en Windows y Linux es otro.
- **Hay textos fuera del diccionario**: el tiempo relativo, los mensajes de merge y revert, los
  nombres de idioma.
- **`D` y `W` comparten 27 claves y 8 tienen texto distinto**, asi que no se pueden fusionar en
  plano.
- Una clave que falta devuelve la propia clave en tiempo de ejecucion: el error se ve en pantalla,
  no al compilar.

Ademas, los textos tienen que llegar a sitios que no son React: el extension host de VS Code (sus
avisos, QuickPick y barra de estado), el manifiesto de la extension (titulos de comandos, nombres
de vistas, descripciones de ajustes), y los errores que vienen del motor de Rust y de la API.

## Decision

### Una sola fuente: `@git-rudder/i18n`

Un paquete sin dependencias, la unica fuente de textos de interfaz de las tres apps:

```
src/locales/es/<espacio>.json   Fuente de verdad
src/locales/en/<espacio>.json   Tipado contra es: una clave que falta o sobra en ingles no compila
src/catalog.ts                  Importa los JSON; el tipo Catalog sale del castellano
src/keys.ts                     Claves tipadas, plurales y parametros por clave
src/generated/params.ts         Parametros de cada texto, generado por scripts/generate-params.mjs
src/translator.ts               createTranslator(locale): t(clave, parametros)
src/plural.ts                   Sufijos _one y _other resueltos con Intl.PluralRules
src/format.ts                   formatRelativeTime, formatDate, formatTimeZone, formatShortcut
src/locale.ts                   Locale ("es" | "en") y resolveLocale
test/parity.test.ts             Paridad de claves, de parametros y de plurales entre es y en
test/consistency.test.ts        Que dos claves no signifiquen lo mismo, y los recuentos
```

Los textos van en JSON y no en TypeScript. TypeScript tipa las claves de un JSON importado, pero
no ve dentro de sus textos, asi que los parametros de cada texto los escribe
`scripts/generate-params.mjs` en `src/generated/params.ts` (`just i18n-gen`). El test de paridad
falla si ese fichero no esta al dia.

Espacios de nombres: `common`, `repos`, `graph`, `staging`, `diff`, `conflict`, `rebase`, `menu`,
`dialog`, `toast`, `settings`, `ai`, `team`, `errors`, `vscode`, `web`, y `email` cuando la API
mande correos. El enlace con React (`I18nProvider`, `useT`) vive en `@git-rudder/ui`, para que
`@git-rudder/i18n` siga sin dependencias y lo puedan usar el extension host y los scripts de build.

### Claves y parametros con tipos

- `t("toast.pushed", { branch: "main" })` compila; sin `branch`, o con otro nombre, no compila. Los
  tipos salen del propio diccionario castellano (las claves del JSON y los parametros generados
  desde el), sin una lista escrita a mano.
- Los marcadores tienen nombres descriptivos: `{count}`, `{branch}`, `{sha}`, `{source}`,
  `{target}`, `{file}`, `{provider}`, `{model}`, `{user}`, `{email}`, `{org}`, `{device}`.
- Nunca se trocea una traduccion para fabricar otra: cada texto que se pinta tiene su clave
  (`menu.copyName`, `menu.copyEmail`), y los titulos de dialogo tienen la suya en vez de quitarle
  los puntos suspensivos a la entrada del menu.
- Los glifos van en los componentes, no en los textos, y los atajos se pasan como parametro
  formateado por `formatShortcut` segun el sistema.

### Plurales

Sufijos `_one` y `_other` (y `_zero` si hace falta) resueltos con `Intl.PluralRules` sobre el
parametro `count`. `t("repos.results", { count: 1 })` da "1 repositorio". Es la misma convencion de
datos que i18next, asi que cambiar a i18next algun dia no obliga a reescribir los diccionarios.

### Dos claves nunca significan lo mismo

Separar los espacios de nombres evita que las claves choquen, pero no el problema que hay detras, y
tiene dos caras. Cuando dos claves dicen lo mismo, `common.cancel` y `web.common.cancel` conviven sin
chocar y durante un tiempo dicen lo mismo de verdad; el dia que alguien cambia una, el producto dice
dos cosas para el mismo boton y nada falla. Y cuando dos claves con el mismo nombre significan cosas
distintas, dejarlas en `repos.localOnly` y `states.localOnly` no dice cual es cual: el nombre tiene
que decirlo. De ahi cuatro reglas, que comprueba `test/consistency.test.ts`:

1. **Dos claves con el mismo texto castellano llevan el mismo ingles.** El castellano es la fuente,
   asi que si el ingles se separa, una de las dos traducciones esta mal.
2. **Dos claves con el mismo ingles llevan el mismo castellano.** Menos grave, porque el traductor
   busca por clave y no por texto ingles (es justo lo que se le reprocha a `vscode.l10n`), pero casi
   siempre es un descuido: "{count} PR abiertos" y "{count} PRs abiertos" para el mismo
   "{count} open PRs".
3. **`common` es el unico sitio de un texto compartido.** Ningun otro espacio repite un par
   (castellano, ingles) que ya este en `common`. Un texto generico de todo el producto va a
   `common`, nunca a `<espacio>.common`.
4. **Un texto con `{count}` tiene formas plurales.** La regla que ya habia era la contraria, que un
   plural ensene el recuento. Sin esta, una clave con contador y sin `_one` pasaba sin que nadie la
   viera, y en pantalla sale siempre la forma `_other` ("1 repositorios"), porque el traductor elige
   la forma con el parametro `count` y no encuentra ninguna.

Las dos primeras admiten excepciones, y se declaran en el test con su motivo y por el conjunto
exacto de claves: una clave nueva con ese texto vuelve a fallar, y una excepcion que ya no hace
falta tambien. Hoy son cuatro: "Usuario", que en ingles es "User" (el grupo de ajustes) y "Username"
(el campo); "Plataformas Git", porque el manifiesto de VS Code escribe las categorias de ajustes en
Title Case; "Open", que es "Abrir" en el menu y "Abierto" en el estado de un pull request; y "All",
que es "Todas" las plataformas y "Todos" los repositorios.

**Los numeros que no son recuentos no se llaman `count` y no llevan plural**: un ordinal
("Conflicto 3"), un identificador ("PR #142"), una posicion ("Paso 2 de 4"), un importe ("120 USD al
ano") o un nombre que el prototipo llamaba `{n}` ("¿Quitar {name} de la lista?"). Y **un texto no
lleva dos recuentos**: "{members} miembros y {pending} invitaciones" no se puede pluralizar con un
solo `count`, asi que son dos claves que la interfaz compone. En un "{used} de {count} asientos" el
sustantivo concuerda con el total, asi que el total es el que se llama `count`.

### Formatos

El tiempo relativo usa `Intl.RelativeTimeFormat` y las fechas `Intl.DateTimeFormat`, siempre con el
idioma de la interfaz. Sustituyen a la funcion `ago()` del prototipo, que tenia los textos escritos
a mano. `formatDate` tiene cuatro estilos: `date`, `dateTime`, `time` y `monthDayTime`, la columna
de fecha del grafo (el `MM-DD HH:mm` de los prototipos en el orden de cada idioma, con hora de
24 horas para que quepa en la columna de VS Code).

### La regla de idioma

`resolveLocale(etiqueta)` devuelve `"es"` si la subetiqueta principal es `es` (`es`, `es-ES`,
`es-419`) y `"en"` en cualquier otro caso. El ingles es el idioma de reserva.

| App     | De donde sale el idioma                                                                                                                                                                                            |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Desktop | El ajuste de apariencia. En el primer arranque, `resolveLocale` sobre el idioma del sistema                                                                                                                        |
| Web     | La landing prerenderizada, de su URL (`/` o `/en/`). Autenticacion y panel, de la preferencia guardada, con el conmutador ES/EN                                                                                    |
| VS Code | `resolveLocale(vscode.env.language)`, aplicado por el extension host y enviado a cada webview en el mensaje `init`. Sin selector: sigue a VS Code, y cambiar el idioma de VS Code exige reiniciarlo de todos modos |

El ajuste de idioma de la IA "igual que VS Code" usa la misma funcion.

### Los textos nativos de VS Code

- **El manifiesto** (titulos de comandos, nombres de vistas, descripciones de ajustes) usa
  `%clave%` en `package.json` y los ficheros `package.nls.json` (ingles, el de reserva de VS Code) y
  `package.nls.es.json`. Es obligatorio: VS Code lee el manifiesto antes de activar la extension, y
  no hay otro mecanismo. Los dos ficheros se generan desde el espacio `vscode.manifest` de
  `@git-rudder/i18n` (en el primer corte se escriben a mano).
- **Los textos en tiempo de ejecucion del extension host** (avisos, QuickPick, InputBox, barra de
  estado) usan el traductor de `@git-rudder/i18n` directamente, con el idioma de `resolveLocale`.
  **No se usa `vscode.l10n`.**

**Por que no `vscode.l10n`.** Sus bundles usan el texto en ingles como clave. Eso pierde las claves
tipadas, obliga a mantener una segunda fuente (o un generador que traduzca de una a otra) y tiene
un problema propio: dos claves con el mismo texto en ingles y distinto en castellano chocan en el
bundle. El diccionario de la web ya tiene un caso ("Revoke" se traduce como "Cerrar" en un sitio y
como "Desconectar" en otro). El traductor propio sigue el mismo idioma que `vscode.l10n` seguiria,
porque la regla es la misma, sin ninguno de esos costes.

### Rust no devuelve textos traducidos

- **El motor** devuelve un `ErrorKind` cerrado y la interfaz lo traduce con
  `errors.engine.<kind>`. La correspondencia es un `Record<BridgeErrorKind, clave>`
  (`ENGINE_ERROR_KEYS` en `@git-rudder/repo-view`, y otro en el extension host para sus avisos), asi
  que un `kind` nuevo sin traduccion no compila. El `message` del error va
  en ingles y es para el log.
- **La API** devuelve problem+json con los mensajes en castellano, como Gastonomo, y un `code`
  estable en el problema y en cada error de campo. La web traduce el `code` con
  `errors.api.<code>` y usa el mensaje en castellano solo si no conoce el codigo.
- **Los correos de la API** (verificacion, recuperacion, invitaciones) se escriben en el espacio
  `email` de `@git-rudder/i18n` y se exportan a JSON al construir; la API los embebe y elige el idioma
  segun la preferencia guardada del usuario. Lo exportado se commitea y la CI comprueba que esta al
  dia, igual que los tipos del protocolo (planificado, con el primer correo).

### Lo que hace cumplir las reglas

- El tipado: una clave o un parametro que no existe no compila, y el ingles esta tipado contra el
  castellano.
- `test/parity.test.ts`: mismas claves, mismos parametros y plurales completos en los dos idiomas.
- `test/consistency.test.ts`: las cuatro reglas de arriba, con sus excepciones declaradas y con
  motivo. Falla tambien si una excepcion ya no hace falta.
- La regla de ESLint `git-rudder/no-jsx-literal-text`: un texto literal en un componente no pasa el lint
  ([ADR 0005](0005-enforcement-automatizado.md)).

### La semilla del prototipo

Un script de un solo uso (`scripts/migrate-from-prototype.ts`) convierte `D` y `W` a espacios de
nombres. `W` va entero a `web.*`, porque 8 de sus claves chocan en texto con `D`. En la conversion
se renombran los marcadores, los recuentos pasan a plurales, y los glifos y los atajos salen de los
textos. Despues el script se borra: la fuente de verdad es el paquete, no el prototipo, y el
repositorio no lo conserva.

En la conversion se quitaron tres claves: `D.accSub`, que duplicaba otra, y `W.ownerTeam` y
`W.teamAcc`, de las cuentas de plataforma compartidas con el equipo, que ya no existen (gaps
X-01). La etiqueta del
rol de propietario es "Titular" (gaps B-50).

`STATES`, `VAULT`, `VS`, `WAUTH` y `WEB70` llegan despues. Las 28 claves de `STATES` que chocan
con `D` se resolvieron mirando donde esta cada una en el prototipo, no por el nombre: 20 tienen el
mismo texto y son la misma cosa, asi que se quedan con una sola clave. Las ocho restantes son dos
cosas distintas, y el nombre lo dice:

| Clave del prototipo | En `D` es                      | En `STATES` es                      | Claves                                             |
| ------------------- | ------------------------------ | ----------------------------------- | -------------------------------------------------- |
| `noPrs`             | el vacio con un filtro puesto  | el vacio sin filtro                 | `repos.pr.emptyForState` y `repos.pr.empty`        |
| `cloneUrl`          | el placeholder del campo       | la etiqueta del campo               | `dialog.clone.urlPlaceholder` y `dialog.clone.url` |
| `inviteTitle`       | el titulo con la organizacion  | el titulo del dialogo, sin ella     | `team.inviteTitle` y `dialog.invite.title`         |
| `planName`          | el nombre del plan             | el plan y sus asientos              | `settings.account.planName` mas `planSeats`        |
| `invite`            | la accion que abre el dialogo  | el boton que lo confirma            | `team.invite` y `common.invite`                    |
| `localOnly`         | un repositorio sin remoto      | un ajuste que no se ha sincronizado | `repos.localOnly` y `settings.sync.deviceOnly`     |
| `saved`             | el aviso de despues de guardar | el estado de la sincronizacion      | `toast.saved` y `settings.sync.saved`              |
| `secAccount`        | "Account & team"               | "Account and team"                  | una sola, con `&`                                  |

Las dos de la sincronizacion llegan con su pantalla; las demas ya estan en el paquete.

**No todo lo que hay en esos diccionarios es texto de producto.** Al traer `STATES` resulto que
99 de sus 250 claves son el andamio de la propia lamina del prototipo: su titulo y subtitulo, los
conmutadores de producto, tema e idioma, el nombre de cada variante dibujada (`vRepoCard`,
`vDlgOpen`, `v403`), la matriz de estados de controles (`stHover`, `kBtnP`) y, sobre todo, la
especificacion de cada patron (`c01T`, `c01P`, `c01Vs`), que habla de milisegundos, de tokens de
CSS y de `aria-busy`. Eso es documentacion de diseno, no se ensena a nadie, y en el diccionario
solo habria que traducirlo y mantenerlo para siempre, asi que no entra. Las especificaciones
citan textos de producto ("No hay stashes", "Ver detalle"), pero esos ya tienen su propia clave;
la cita es redundante. Al traer `VAULT`, `VS`, `WAUTH` y `WEB70` hay que mirar lo mismo.

Al escribir las reglas, el diccionario que ya estaba en el repositorio tenia 15 claves que
repetian un texto de `common` (casi todo `web.common.*` y los campos de `web.accounts.*`), que se
han borrado, y cinco inconsistencias reales, que se han corregido:
`settings.account.revokeSession` decia "Cerrar" para "Revoke", que es el mismo fallo que este ADR
cita del prototipo y habia sobrevivido a la conversion; `diff.unstageFile` no decia "archivo" en
castellano y su hermana `stageFile` si; `repos.workspace.openPrCount_other` pluralizaba la sigla
("PRs"); `web.nav.signup` decia "Sign up" donde el titulo de la pagina dice "Create account"; y el
enlace del pie a la pagina de estado tenia clave propia en vez de usar el titulo de esa pagina.

### Lo que queda por partir

Al traer el handoff quedaron siete textos con un numero que gobierna un sustantivo y sin
formas plurales, asi que diran "1 asientos". No se arreglan con renombrar el parametro,
porque **llevan dos o tres recuentos en la misma frase** y una clave solo puede pluralizar
por uno:

| Clave                       | Texto                                                                                   |
| --------------------------- | --------------------------------------------------------------------------------------- |
| `dialog.signOut.removed`    | Se borran: {accounts} cuentas y {keys} llaves de otros equipos                          |
| `dialog.signOut.kept`       | Se quedan: {accounts} cuentas y {keys} llaves anadidas aqui                             |
| `web.team.tmCounts`         | {members} miembros y {pending} invitacion pendiente · {used} de {total} asientos usados |
| `web.billing.biAnnualLine`  | Pago anual · {seats} asientos · se renueva el {date}                                    |
| `web.billing.biMonthlyLine` | Pago mensual · {seats} asientos · se renueva el {date}                                  |
| `web.billing.seMin`         | No puedes bajar de {used}: hay {members} miembros y {pending} invitaciones pendientes   |
| `web.billing.seReduce`      | Reducir a {seats} asientos                                                              |

La salida es la de arriba: una clave por recuento y que la interfaz las componga, que es
ademas lo que quita el `·` de los textos. Partirlas cambia como se pinta cada una de esas
lineas, asi que se hace con su pantalla y no en la importacion. Mientras no se parta, la
regla del `{count}` no las ve: su parametro no se llama `count`, y llamarlo asi sin partir
la frase solo elegiria la forma por uno de los dos numeros.

## Consecuencias

A favor:

- Una clave que falta, un parametro mal escrito o una traduccion que se olvido de un idioma se
  descubren al compilar, no en la pantalla de un usuario.
- "1 repositorio" y "2 repositorios" en los dos idiomas, y una clave con contador no puede
  olvidarse de sus formas.
- Dos textos iguales no pueden separarse en silencio: en cuanto una traduccion se aparta de otra
  identica, la CI lo dice.
- El motor y la API no saben nada de idiomas mas alla del castellano de los mensajes de respaldo.
- Una sola regla de idioma para las tres apps y para la IA.

En contra:

- El tipado de claves anidadas con plurales es TypeScript avanzado (tipos condicionales y de
  plantilla). Funciona, pero un error de tipos ahi se lee mal.
- Los plurales con sufijos solo cubren las formas de castellano e ingles. Un tercer idioma con mas
  formas (ruso, arabe) pediria revisar la convencion o pasar a ICU.
- El extension host traduce con su propio traductor en lugar del mecanismo de VS Code. Si VS Code
  anadiera algo que solo funciona con `vscode.l10n`, habria que adaptarse.
- Exportar los textos de correo a la API es un generador mas con su comprobacion en la CI.
- Las excepciones de las reglas de textos repetidos hay que mantenerlas: cambiar un texto puede
  obligar a quitar una o a anadir otra, y el test lo dice pero no lo arregla.

## Alternativas descartadas

**i18next con react-i18next.** Maduro, pero es una dependencia de ejecucion en las tres apps, y sus
claves tipadas se consiguen con fusion de declaraciones. Como los datos son compatibles, sigue
siendo una salida posible.

**FormatJS con mensajes ICU.** Plurales y selecciones completos, a cambio de mas peso y de una
sintaxis que no aporta nada con dos idiomas.

**Lingui.** Macros y un paso de compilacion de catalogos que no encaja con paquetes consumidos como
codigo fuente.

**`vscode.l10n` para el extension host.** Ver arriba.

**Mantener los diccionarios planos del prototipo.** Conserva todos los problemas del contexto.

**Un diccionario por app.** Los textos comunes (botones, errores, menus) se duplicarian y
divergirian.
