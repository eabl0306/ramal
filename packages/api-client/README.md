# @ramal/api-client

Cliente HTTP de la Ramal API para la web. Solo lo importa `apps/web/src/infrastructure`.

```ts
const client = createApiClient(); // mismo origen: el gateway sirve web y API juntas
const { data, error, response } = await client.GET('/health/ready');
```

- `src/generated/schema.ts` sale de `apps/api/openapi.json` con openapi-typescript. No se edita a
  mano: `just api-client` lo regenera y la CI falla si queda diferencia con lo commiteado.
- Una respuesta que no es 2xx llega en `error` (la API manda problem+json, o el informe de salud
  en un 503). El adaptador de la web valida el cuerpo con zod antes de pasarlo al dominio.
- Sin dependencias de otros paquetes del workspace; en tiempo de ejecucion, solo `openapi-fetch`.

```sh
just api-client                                        # regenera los tipos
corepack pnpm --filter @ramal/api-client test:coverage # 60% minimo
```
