# ADR 0025: Modelos de IA recomendados desde un fichero remoto, por nivel y por funcion

- Estado: aceptado
- Fecha: 2026-10-01

## Contexto

Ramal genera con IA tres cosas: el mensaje de commit, el titulo y la descripcion de un pull
request y el resumen de un commit ([ADR 0013](0013-proxy-de-ia-de-organizacion.md)). El usuario
elige proveedor, llave y modelo. La pregunta es que modelo se le recomienda, sin escribir nombres
de modelos en el binario.

Lo que obliga a decidirlo con cuidado:

- **Los nombres caducan.** Los modelos por defecto del prototipo (`claude-sonnet-4-5`,
  `gpt-5-mini`, `gemini-2.5-flash`, `llama-3.3-70b-versatile`) ya no eran los buenos el dia del
  analisis, y alguno ya no esta disponible para cuentas nuevas. gaps X-10 prohibe las constantes:
  una app instalada, y mas un MSI que no se actualiza solo, no se puede corregir.
- **Los patrones de nombre tampoco valen.** OpenAI cambio la forma de sus nombres (de `mini` y
  `nano` a otra familia). Una heuristica en el binario se habria roto igual que una constante.
- **Los precios cambian aunque el nombre no.** Y entre proveedores, el modelo rapido de uno puede
  costar diez veces el de otro.
- **Hoy el modelo va en la llave.** Para usar un modelo barato en los commits y otro mejor en los
  PRs con el mismo proveedor habria que crear dos llaves con el mismo secreto.
- **La IA con llave personal funciona sin cuenta de Ramal**
  ([ADR 0014](0014-usar-ramal-sin-cuenta.md)). Lo que se use para recomendar no puede pedir sesion.
- Cada proveedor publica la lista de sus modelos por API (Anthropic, OpenAI, Gemini, Mistral, Groq
  y OpenRouter; Ollama lista los instalados), y el motor ya la pide despues de validar la llave.

## Decision

### Un fichero de recomendaciones servido por la API

La API sirve un JSON **publico, sin sesion y cacheable** en `GET /v1/ai/model-recommendations`
(contrato en [api-surface.md](../api-surface.md#recomendaciones-de-modelos-de-ia)). Por cada
proveedor y cada nivel trae una lista ordenada de ids de modelo, y dice que nivel usa cada
funcion:

```json
{
  "schemaVersion": 1,
  "revision": "2026-10-01",
  "nextReviewBy": "2027-01-01",
  "features": { "commitMessage": "fast", "pullRequest": "balanced", "commitSummary": "balanced" },
  "providers": {
    "anthropic": { "fast": ["claude-haiku-4-5"], "balanced": ["claude-sonnet-5-5"] },
    "openai": { "fast": ["gpt-6-luna"], "balanced": ["gpt-6.1-sol"] }
  }
}
```

Los ids del ejemplo son los de la primera revision; los que valen son los del fichero.

- **Es publico** porque lo necesita quien usa una llave personal sin cuenta, igual que el
  actualizador. No lleva ningun identificador del cliente.
- **El fichero vive en el repositorio**, dentro de `apps/api`, y la API lo incrusta al compilar.
  Un test lo valida (proveedores y niveles conocidos, ningun id vacio). Cambiarlo es un commit y un
  despliegue de la API, nunca una version de las apps.
- **Se revisa cada trimestre** y cuando un proveedor retira un modelo. `nextReviewBy` lo deja
  escrito.

### Como lo usa la app

- El motor pide la lista viva de modelos del proveedor despues de validar la llave (con cache) y
  **la cruza con el fichero**: marca como "Recomendado" el primer id de la lista del nivel que
  exista en la lista viva. **Nunca recomienda un modelo que el proveedor ya no lista.**
- La recomendacion **solo preselecciona** al crear una llave o al elegir el modelo de una funcion
  que no tiene ninguno. Nunca cambia un modelo que el usuario ya eligio.
- El motor guarda en disco la ultima copia con su `ETag` y la revalida como mucho una vez al dia.
- **Sin red y sin copia en cache, no hay recomendacion**: se ensena la lista del proveedor tal
  cual (en OpenRouter, ordenada por precio, que es el unico que lo da por API) y el usuario elige.
- Ollama y los endpoints personalizados no tienen recomendacion: Ollama lista lo instalado y un
  endpoint personalizado puede no tener lista.

### Dos niveles, razonamiento al minimo

- **Rapido** (`fast`): mensajes de commit. Son frecuentes, con entrada corta y salida de una
  linea.
- **Equilibrado** (`balanced`): titulo y descripcion de PR y resumen de commit. Son menos
  frecuentes y con mas texto; el resumen es bajo demanda, asi que su coste pesa poco.
- **Razonamiento al minimo** que admita cada proveedor, en los dos niveles. Los tokens de
  razonamiento ocultos pueden multiplicar varias veces el coste de un mensaje de commit. Lo aplican
  los adaptadores de `ramal-ai`, por proveedor.

### Un modelo por funcion

- Las preferencias de IA (`ai`) ganan un **modelo opcional por funcion**. Nulo es "el de la llave":
  la llave conserva su modelo, que pasa a ser el modelo por defecto de esa llave. Asi una misma
  llave sirve para commits con un modelo rapido y para PRs con uno equilibrado.
- **Llaves de organizacion: el modelo lo fija el admin, por funcion.** Cada llave de organizacion
  tiene su modelo por defecto y, si el admin quiere, uno por funcion. El miembro no lo puede
  cambiar: el proxy no acepta un modelo en la peticion y usa el que fijo el admin, y la app ensena
  ese modelo como dato de solo lectura. El modelo por funcion de las preferencias del miembro solo
  se aplica a sus llaves personales.

### Sin coste en la v1

La interfaz no ensena un coste aproximado por generacion. Solo OpenRouter da precios por API; para
el resto habria que mantenerlos en el fichero, con fecha, y un precio viejo en pantalla es peor que
ninguno. El texto de ajustes lo explica sin cifras: "Para los mensajes de commit te recomendamos un
modelo rápido y barato. Para PRs y resúmenes, uno equilibrado. Cada generación la paga tu llave."

## Consecuencias

A favor:

- Corregir una recomendacion es un despliegue de la API, no una version de las apps. Las
  instalaciones viejas y los MSI reciben la correccion.
- Nunca se recomienda un modelo retirado, porque la recomendacion se cruza con la lista viva.
- Funciona sin cuenta y degrada sin red: sin fichero, la lista del proveedor sigue ahi.
- Un modelo barato para los commits y otro mejor para los PRs con la misma llave.
- Las organizaciones controlan que modelo, y por tanto que coste, usa cada funcion con sus llaves.

En contra:

- **Alguien tiene que mantener el fichero.** Si nadie lo revisa, envejece igual que una constante,
  aunque sin llegar a recomendar algo retirado.
- Es una peticion mas que sale del equipo sin cuenta. No lleva identificadores, y se dice en la
  lista de [ADR 0014](0014-usar-ramal-sin-cuenta.md).
- El documento `ai` crece un campo por funcion, y la regla de resolucion del motor tiene que saber
  que el modelo de una llave de organizacion manda sobre el del miembro.

## Alternativas descartadas

**Constantes en el binario.** Lo que hacia el prototipo. Prohibido por X-10: caduca en meses y no
se corrige en instalaciones viejas.

**Heuristica de nombres en el binario** (`haiku`, `mini`, `flash-lite`, `small`). Sin servidor,
pero se rompe con cada cambio de nombre, y OpenAI ya cambio los suyos.

**Sin recomendacion.** Nada que mantener, pero el usuario no sabe cual es el barato y puede acabar
generando cada commit con el modelo mas caro. Queda como el comportamiento sin red.

**El fichero junto a los artefactos de publicacion en GitHub.** Tambien es publico y cacheable,
pero ataria su URL al repositorio y a su visibilidad, y la API ya sirve el actualizador con la
misma politica de cache.

**Ensenar el coste por generacion.** Solo un proveedor da precios por API; para el resto, un precio
mantenido a mano y desfasado confunde mas de lo que ayuda.
