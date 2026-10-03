# git-rudder-vscode-engine

El motor de Git de Git Rudder servido por JSON-RPC 2.0 sobre stdio. La extension de VS Code lo
lanza como proceso hijo; el protocolo es el mismo que usa escritorio en proceso.

```
git-rudder-vscode-engine [--git-path <ruta>] [--log-level <filtro>] [--version]
```

- Marcos `Content-Length: <bytes>\r\n\r\n` mas el JSON. Mas de 32 MiB cierra la conexion.
- stdout es solo protocolo; el log va a stderr.
- Termina cuando se cierra stdin.

Recetas: `just engine-test` (tests, incluido uno que lanza el binario) y
`just vscode-engine-build` (release).
