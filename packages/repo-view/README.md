# @git-rudder/repo-view

La vista de repositorio que comparten Git Rudder Desktop y los webviews de VS Code. El primer corte
trae el grafo de commits: filas virtualizadas, carriles en SVG, pildoras de refs, seleccion con
teclado y paginas de `graph/window` al bajar.

Habla con el motor solo por el cliente de `@git-rudder/bridge`. No sabe si debajo hay Tauri o el
extension host: el transporte lo pone cada app.

## Montarlo en un host

```tsx
import { createBridgeClient } from '@git-rudder/bridge';
import { GraphView, RepoViewProvider } from '@git-rudder/repo-view';
import { I18nProvider, IconProvider } from '@git-rudder/ui';
import { lucideIcons } from '@git-rudder/ui/icons/lucide'; // VS Code: codicons de '@git-rudder/ui/icons/codicons'
import '@git-rudder/ui/themes/git-rudder.css'; // VS Code: '@git-rudder/ui/themes/vscode.css'

const client = createBridgeClient(transport); // TauriTransport o VsCodeTransport

<I18nProvider locale={locale}>
  <IconProvider icons={lucideIcons}>
    <RepoViewProvider>
      <GraphView
        client={client}
        repoId={repoId} // lo da repo/open, que llama el host
        density="comfortable" // VS Code: "compact"
        onSelectionChange={(row) => {}}
        onActivate={(row) => {}} // doble clic o Intro
      />
    </RepoViewProvider>
  </IconProvider>
</I18nProvider>;
```

- `GraphView` ocupa el alto de su contenedor y desplaza por dentro: dale un padre con altura.
- `RepoViewProvider` crea un QueryClient propio. Si el host ya tiene uno, pasalo con
  `queryClient`. `createRepoViewQueryClient()` da uno con los valores recomendados.
- La seleccion puede ir controlada con `selectedSha`. Sin esa prop, la lleva la vista.
- `useGraphWindow(client, repoId, { pageSize })` da las filas sin la tabla, para otra vista.
- `graphWindowKey(repoId, pageSize)` sirve para invalidar la cache cuando el motor avise de
  cambios. Todas las claves del paquete empiezan por `'repo-view'`.
- `ENGINE_ERROR_KEYS` traduce cada `BridgeErrorKind` a su clave de `@git-rudder/i18n`.

## Como dibuja

- Columnas `200 | grafo | 1fr | 150 | 110 | 76` en comfortable y las de VS Code en compact. La del
  grafo mide 130 (120 en compact) y crece si el motor pide mas carriles.
- `X = originX + carril * 20`, `Y = fila * alto + alto / 2`, con los numeros de `DENSITIES`.
- Cada arista va en tres tramos con el carril de viaje del motor: curva al salir si cambia de
  carril, vertical y curva al llegar. Asi salen las formas del diseno (recta, rama que sale,
  merge) y una arista nunca cruza un nodo ajeno.
- Si el padre no esta cargado, la arista baja hasta el borde inferior. Al llegar la pagina del
  padre, se resuelve por su sha.
- Un solo SVG para el tramo visible, encima de las filas y oculto a los lectores de pantalla.

## Accesibilidad

Es un `grid` de ARIA con cabeceras de columna y una sola parada de tabulador. Flechas, Re Pag,
Av Pag, Inicio y Fin mueven la seleccion; Intro activa. La fila activa va por
`aria-activedescendant`, porque una fila virtualizada puede desmontarse.

## Comprobaciones

```sh
corepack pnpm --filter @git-rudder/repo-view lint
corepack pnpm --filter @git-rudder/repo-view typecheck
corepack pnpm --filter @git-rudder/repo-view test:coverage   # 60% minimo
```

Las funciones puras (`src/graph`) tienen sus tests sin React. Los componentes se prueban con un
motor falso detras de un `Transport` real (`src/test/fake-engine.ts`).
