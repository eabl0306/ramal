# Ramal Desktop

La app de escritorio sobre Tauri 2. El primer corte abre una carpeta con un repositorio y pinta
su grafo con `@ramal/repo-view`, con el motor de Git en el mismo proceso.

## Como esta hecho

- `src-tauri/` es el crate `ramal-desktop`, el host. Tiene dos comandos:
  - `rpc`: lleva el sobre JSON-RPC al motor, el mismo que usa el sidecar de VS Code. Solo deja
    pasar los metodos de la lista de la ventana (`surfaces.rs`); la ventana principal solo puede
    pedir `graph/window`.
  - `repo_pick_and_open`: ensena el dialogo nativo de carpetas desde Rust y abre el repositorio.
    El webview nunca manda una ruta.
- `src/` es el frontend, por capas que vigila `eslint-plugin-boundaries`:
  - `application`: puertos del host (`RepositoryPicker`, `AppearanceStore`) y las pestanas.
  - `infrastructure`: `TauriTransport`, el selector sobre `repo_pick_and_open` y el
    `localStorage`.
  - `ui`: barra de titulo, Inicio y la vista del repositorio. No importa Tauri ni adaptadores.
  - `app`: composition root, proveedores y entrada.

## Comandos

```sh
just desktop-check          # lint, tipos, cobertura y build de Vite
just desktop-e2e            # Playwright contra el build, con el IPC simulado
just desktop-rust-check     # clippy, tests y cobertura del host (necesita webkit2gtk)
just desktop-build          # tauri build sin empaquetar
just desktop-e2e-native     # la app real con WebdriverIO; en Linux sin pantalla, bajo xvfb
just desktop-dev            # tauri dev, con Vite en el puerto 1420
```

Sin webkit2gtk en la maquina, el lado nativo corre en Docker con la misma imagen de Linux que
fija el repo (`tools/docker/tauri-linux.Dockerfile`, Ubuntu 22.04):

```sh
just desktop-docker-rust-check
just desktop-docker-build
just desktop-docker-e2e-native
```

Necesitan `node_modules` instalado en la maquina (`just setup`): el repo se monta en la misma
ruta dentro del contenedor.

## Cosas que conviene saber

- **CSP.** La de produccion vive en `src-tauri/tauri.conf.json`. El preview de Vite la sirve
  igual, asi que el E2E de Playwright falla con cualquier violacion. React Aria inyecta un
  `<style>` propio (el `touch-action` de lo pulsable): se permite ese texto exacto por su hash,
  nunca con `'unsafe-inline'`. Si React Aria lo cambia, el E2E lo dice.
- **E2E nativo.** Usa el driver embebido de `@wdio/tauri-service`, que solo entra con la feature
  `e2e` del crate. Esa feature tambien deja que `RAMAL_E2E_FOLDER` elija la carpeta, porque
  WebDriver no puede manejar el dialogo del sistema. El binario normal no lleva nada de esto.
- **knip.** No entiende el `runner` ni los `services` de WebdriverIO, por eso
  `@wdio/local-runner` y `@wdio/tauri-service` estan en su `ignoreDependencies`.
- **wdio y la ventana.** `@wdio/tauri-service` intenta reenfocar la ventana con la API global de
  Tauri, que esta app no lleva (`withGlobalTauri: false`). `wdio.conf.ts` elige la ventana a mano
  con `switchToWindow` y asi no espera 5 s en cada busqueda.
- **Log.** El host escribe en stderr; `RAMAL_LOG=debug` sube el detalle.
- **Sin verificar todavia:** Windows y macOS, el empaquetado (`desktop-build` usa `--no-bundle`)
  y `tauri dev` en una maquina sin webkit2gtk (las recetas de Docker no lo cubren).
