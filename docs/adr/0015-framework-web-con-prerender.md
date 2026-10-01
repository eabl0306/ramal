# ADR 0015: React Router 8 en modo framework, sin SSR y con prerender de la landing

- Estado: aceptado
- Fecha: 2026-09-30

## Contexto

Ramal Web son tres cosas con necesidades distintas:

- **La landing.** Tiene que verse bien en buscadores y en las previsualizaciones de enlaces, en
  castellano y en ingles, pintar rapido y ensenar las descargas de la ultima version con su tamano
  y sus requisitos.
- **Las paginas de autenticacion.** Registro, login, OAuth, recuperacion de contrasena,
  verificacion de correo, "Autorizar Ramal Desktop en este equipo" y la pagina del codigo del
  device flow. No necesitan SEO.
- **El panel.** Detras del login: resumen, workspaces, repositorios, cuentas ligadas, equipo y
  ajustes en la nube. Una SPA.

Y tiene que encajar con el resto: escritorio y los webviews de VS Code son apps de Vite que usan
los mismos paquetes (`@ramal/ui`, `@ramal/i18n`, `@ramal/domain`), consumidos como codigo fuente
([ADR 0001](0001-monorepo-poliglota.md)). La web se sirve como ficheros estaticos detras del mismo
gateway que la API ([ADR 0007](0007-docker-desarrollo-y-produccion.md)), y la sesion va en cookies
HttpOnly de la API ([ADR 0008](0008-autenticacion-con-tokens.md)), no en un servidor de la web.

Una SPA de Vite a secas daria a los buscadores un HTML vacio. Un framework con SSR obligaria a
mantener un servidor de Node solo para la landing.

## Decision

React Router 8 (8.4.0) en modo framework, con el plugin de Vite, `ssr: false` y `prerender` para
las rutas de la landing.

### Configuracion

- `react-router.config.ts` fija `appDirectory: "src/app"`, para que las rutas vivan en la capa
  `app` del hexagono de la web, `ssr: false` y la lista de rutas a prerenderizar.
- Se prerenderizan la landing en castellano (`/`) y en ingles (`/en/`), y las paginas publicas que
  vengan despues con el mismo esquema de URL (notas de version, paginas legales). El idioma de una
  pagina prerenderizada sale de su URL, y cada una declara la otra con `hreflang`.
- Todo lo demas (autenticacion y panel) es SPA. Al construir, React Router genera el fichero de
  fallback de la SPA (`__spa-fallback.html`), que el nginx de la web sirve para cualquier ruta que
  no sea un fichero prerenderizado. El idioma de esas rutas sale de la preferencia guardada.
- React 19.3, Vite 8.3 y React Router 8.4. React Router 8 exige Node 22.22 o superior; el
  repositorio ya pide Node 24.

### Que se prerenderiza y que no

- Los `loader` de las rutas prerenderizadas corren en Node al construir, y **solo pueden leer datos
  publicos**: textos de `@ramal/i18n` y los datos de descarga de `GET /v1/releases/latest`. Nada de
  sesion en el prerender.
- Si la API no responde al construir, la pagina se prerenderiza sin los datos de descarga y el
  cliente los pide al hidratar. El build de la web no depende de que la API este levantada.
- Despues de hidratar, el cliente refresca los datos de descarga y detecta el sistema operativo en
  el navegador para destacar el instalador que toca. Detectar el SO al construir no tiene sentido.
- Las rutas de la SPA usan `clientLoader`, que llama a un caso de uso con los puertos del
  contenedor y devuelve una union de resultados, nunca lanza.
- La guardia del panel vive en el cliente, pero la autorizacion es siempre de la API: la guardia
  solo evita pintar una pantalla vacia antes del 401.

### Hexagonal dentro de la web

Cinco carpetas dentro de `src`, como Gastonomo ([ADR 0002](0002-arquitectura-hexagonal.md)):

- Los modulos de ruta de `src/app/routes` son finos: su `clientLoader` llama a un caso de uso y su
  componente pinta una pantalla de `src/ui`.
- El contenedor lo crea `src/app/composition-root.ts` una vez por carga y `root.tsx` lo entrega a
  la interfaz con providers. Las pantallas reciben los puertos por contexto y nunca construyen
  adaptadores.
- `@ramal/api-client` solo lo importa `src/infrastructure`.

### Como se sirve

- Ficheros estaticos detras del gateway, en el mismo origen que la API. No hay servidor de Node en
  produccion.
- `no-store` para el HTML (incluido el fallback de la SPA) e `immutable` para los assets con hash.
- La CSP la pone el gateway como cabecera. React Router incrusta scripts de hidratacion en el HTML
  prerenderizado, asi que `script-src` lleva los hashes que genera el build; lo que no se acepta es
  `'unsafe-inline'` ([ADR 0011](0011-ui-compartida-y-puente-tipado.md)).

### La vista previa del producto

El prototipo de la landing incrustaba el prototipo de escritorio en un `<iframe>` que cargaba
React y Babel de unpkg y evaluaba codigo con `new Function`. Eso no se publica nunca. La vista
previa de produccion pinta componentes de `@ramal/repo-view` con datos de demostracion y un
transporte en memoria, cargados de forma diferida por debajo del primer pantallazo (planificado).

## Consecuencias

A favor:

- Los buscadores y las previsualizaciones ven contenido real en los dos idiomas.
- No hay un servidor de la web que desplegar, vigilar ni escalar: la web son ficheros.
- Mismo toolchain (Vite) y misma familia de router que el resto de apps.
- La landing ensena los datos de la ultima version aunque la API no estuviera al construir.

En contra:

- Las convenciones de los modulos de ruta del modo framework hay que aprenderlas, y sin servidor no
  hay `action` ni `headers`: todo lo que escribe va por la API desde el cliente.
- React Router saca versiones mayores deprisa (de la 7 a la 8 en menos de dos anos). Se fija la
  version exacta y se sube leyendo las notas.
- Los datos de descarga prerenderizados envejecen hasta el siguiente build. El refresco en el
  cliente lo corrige al cargar, pero un buscador vera la version del ultimo build.
- La CSP con hashes obliga a regenerar la cabecera del gateway en cada build de la web.

## Alternativas descartadas

**SPA de Vite sin prerender.** La mas simple, pero la landing seria un HTML vacio para buscadores y
previsualizaciones.

**Next.js 16.** SSR e ISR potentes, pero necesita un runtime de Node para casi todo, y su modelo de
componentes de servidor y su bundler se alejan del pipeline de Vite para el que estan hechos los
paquetes compartidos. Demasiado para una landing estatica mas una SPA.

**TanStack Start en modo SPA con prerender.** Capacidades parecidas y rutas con tipos, pero un
ecosistema mas joven y otro router distinto del que se usa en el resto.

**Astro para la landing y una SPA de Vite aparte.** Una landing estatica excelente, a cambio de dos
frameworks y dos builds para una sola web.

**SSR con servidor de Node.** Un proceso mas en produccion, y ningun dato del panel gana nada con
pintarse en el servidor.
