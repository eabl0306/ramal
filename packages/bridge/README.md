# @git-rudder/bridge

Cliente tipado del motor de Git Rudder. Sin dependencias en tiempo de ejecucion.

```ts
const client = createBridgeClient(transport);
const window = await client.call('graph/window', { repoId, offset: 0, limit: 200 });
```

- `Transport` es el puerto: cada app trae el suyo (Tauri, postMessage de VS Code).
- `call` rechaza con `BridgeError { kind, message, data }`. Timeout de 30 s por defecto
  (`timeoutMs: null` lo quita); `signal` cancela y manda `$/cancel`.
- `src/generated/` sale de `git-rudder-protocol` con ts-rs. No se edita a mano:
  `just protocol-gen` lo regenera y `just protocol-check` falla si no esta al dia.
