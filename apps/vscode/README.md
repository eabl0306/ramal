# Git Rudder for VS Code

El grafo de commits del repositorio que tienes abierto, con el tema y el idioma de VS Code.
Ejecuta **Git Rudder: Open Graph** desde la paleta de comandos o desde la vista Git Rudder de la barra de
actividad.

## Como esta hecho

- `src/extension`: el extension host (Node). `application` tiene los puertos y los casos de uso
  (abrir el grafo, supervisar el motor, enrutar los mensajes de los webviews) y no importa
  `vscode`; `infrastructure` tiene los adaptadores (API de VS Code, proceso del sidecar, marcos
  `Content-Length`, validacion con zod, HTML y CSP de los webviews); `app` es el composition root.
- `src/webview`: los webviews (React). `VsCodeTransport` lleva el cliente de `@git-rudder/bridge` por
  `postMessage` y `GraphApp` monta el `GraphView` de `@git-rudder/repo-view` con `vscode.css`,
  densidad compacta y Codicons.
- `src/shared`: los mensajes entre los dos lados.
- El motor es el binario `git-rudder-vscode-engine`. Instalado, siempre el de la VSIX
  (`dist/bin`). Al desarrollar, el ajuste de usuario `gitRudder.engine.path`, `GIT_RUDDER_ENGINE_PATH` o lo
  que haya en `target/` del monorepo.

## Desarrollo

```sh
corepack pnpm --filter git-rudder dev      # host con esbuild en modo watch y Vite en 127.0.0.1:5175
```

Despues, "Run Extension" en `.vscode/launch.json` de esta carpeta: carga los webviews desde Vite
con la CSP de desarrollo.

| Receta                         | Que hace                                                                              |
| ------------------------------ | ------------------------------------------------------------------------------------- |
| `just vscode-check`            | Lint, tipos, cobertura (80% en `application`, 60% global), build                      |
| `just vscode-e2e`              | Playwright contra el bundle del webview (puerto 4175)                                 |
| `just vscode-test-integration` | VS Code de verdad con el sidecar y un repo de prueba; xvfb solo en Linux sin pantalla |
| `just vscode-package`          | VSIX de esta plataforma con el motor en `dist/bin`                                    |
