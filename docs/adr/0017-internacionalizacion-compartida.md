# ADR 0017: Internacionalizacion compartida con claves tipadas

- Estado: aceptado
- Fecha: 2026-09-30

## Contexto

Las tres apps de Ramal estan en castellano y en ingles. Los prototipos traen dos diccionarios
planos, `D` (466 claves, en `ramal-data.js`) y `W` (148, el de la web), con tuplas
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

### Una sola fuente: `@ramal/i18n`

Un paquete sin dependencias, la unica fuente de textos de interfaz de las tres apps:

```
src/locales/es/<espacio>.ts   Fuente de verdad
src/locales/en/<espacio>.ts   Tipado contra es: una clave que falta en ingles no compila
src/translator.ts             createTranslator(locale): t(clave, parametros)
src/plural.ts                 Sufijos _one y _other resueltos con Intl.PluralRules
src/format.ts                 formatRelativeTime, formatDate, formatShortcut
src/locale.ts                 Locale ("es" | "en") y resolveLocale
test/parity.test.ts           Paridad de claves, de parametros y de plurales entre es y en
```

Espacios de nombres: `common`, `repos`, `graph`, `staging`, `diff`, `conflict`, `rebase`, `menu`,
`dialog`, `toast`, `settings`, `ai`, `team`, `errors`, `vscode`, `web`, y `email` cuando la API
mande correos. El enlace con React (`I18nProvider`, `useT`) vive en `@ramal/ui`, para que
`@ramal/i18n` siga sin dependencias y lo puedan usar el extension host y los scripts de build.

### Claves y parametros con tipos

- `t("toast.pushed", { branch: "main" })` compila; sin `branch`, o con otro nombre, no compila. Los
  tipos salen del propio diccionario castellano, sin una lista escrita a mano.
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

### Formatos

El tiempo relativo usa `Intl.RelativeTimeFormat` y las fechas `Intl.DateTimeFormat`, siempre con el
idioma de la interfaz. Sustituyen a la funcion `ago()` del prototipo, que tenia los textos escritos
a mano.

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
  `@ramal/i18n` (en el primer corte se escriben a mano).
- **Los textos en tiempo de ejecucion del extension host** (avisos, QuickPick, InputBox, barra de
  estado) usan el traductor de `@ramal/i18n` directamente, con el idioma de `resolveLocale`.
  **No se usa `vscode.l10n`.**

**Por que no `vscode.l10n`.** Sus bundles usan el texto en ingles como clave. Eso pierde las claves
tipadas, obliga a mantener una segunda fuente (o un generador que traduzca de una a otra) y tiene
un problema propio: dos claves con el mismo texto en ingles y distinto en castellano chocan en el
bundle. El diccionario de la web ya tiene un caso ("Revoke" se traduce como "Cerrar" en un sitio y
como "Desconectar" en otro). El traductor propio sigue el mismo idioma que `vscode.l10n` seguiria,
porque la regla es la misma, sin ninguno de esos costes.

### Rust no devuelve textos traducidos

- **El motor** devuelve un `ErrorKind` cerrado y la interfaz lo traduce con
  `errors.engine.<kind>`. La correspondencia es un `Record<ErrorKind, clave>` en
  `@ramal/repo-view`, asi que un `kind` nuevo sin traduccion no compila. El `message` del error va
  en ingles y es para el log.
- **La API** devuelve problem+json con los mensajes en castellano, como Gastonomo, y un `code`
  estable en el problema y en cada error de campo. La web traduce el `code` con
  `errors.api.<code>` y usa el mensaje en castellano solo si no conoce el codigo.
- **Los correos de la API** (verificacion, recuperacion, invitaciones) se escriben en el espacio
  `email` de `@ramal/i18n` y se exportan a JSON al construir; la API los embebe y elige el idioma
  segun la preferencia guardada del usuario. Lo exportado se commitea y la CI comprueba que esta al
  dia, igual que los tipos del protocolo (planificado, con el primer correo).

### Lo que hace cumplir las reglas

- El tipado: una clave o un parametro que no existe no compila, y el ingles esta tipado contra el
  castellano.
- `test/parity.test.ts`: mismas claves, mismos parametros y plurales completos en los dos idiomas.
- La regla de ESLint `ramal/no-jsx-literal-text`: un texto literal en un componente no pasa el lint
  ([ADR 0005](0005-enforcement-automatizado.md)).

### La semilla del prototipo

Un script de un solo uso (`scripts/migrate-from-prototype.ts`) convierte `D` y `W` a espacios de
nombres. `W` va entero a `web.*`, porque 8 de sus claves chocan en texto con `D`. En la conversion
se renombran los marcadores, los recuentos pasan a plurales, y los glifos y los atajos salen de los
textos. Despues el script se borra: la fuente de verdad es el paquete, no el prototipo.

## Consecuencias

A favor:

- Una clave que falta, un parametro mal escrito o una traduccion que se olvido de un idioma se
  descubren al compilar, no en la pantalla de un usuario.
- "1 repositorio" y "2 repositorios" en los dos idiomas.
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
