# @ramal/eslint-config

La configuracion de ESLint de todo el monorepo, por capas. Cada app y cada paquete tiene su
`eslint.config.js` y compone estas piezas; la raiz solo linta `tools/` y este paquete.

| Export       | Que pone                                                                                  |
| ------------ | ----------------------------------------------------------------------------------------- |
| `base`       | JavaScript recomendado, TypeScript con tipos en modo estricto, orden de imports, Prettier |
| `quality`    | Los limites de ADR 0005 (`LIMITS`): tamano, complejidad, anidamiento, parametros          |
| `react`      | Hooks, compilador de React, accesibilidad y la regla de textos visibles                   |
| `node`       | Entorno de Node: host de la extension, scripts y configuracion                            |
| `boundaries` | Capas hexagonales con `eslint-plugin-boundaries` y denegacion por defecto                 |
| `tests`      | Lo que se relaja en los tests (largo de fichero y de funcion); la complejidad no          |

Reglas propias, en `rules/`:

- `ramal/no-raw-colors`: ningun color escrito a mano fuera de `packages/ui/src/themes`.
- `ramal/no-jsx-literal-text`: ningun texto visible en el JSX; sale de `@ramal/i18n`.
- `ramal/ticketed-todo`: un marcador de trabajo pendiente lleva ticket (`RAM-123` o `#123`).

Cambiar un limite es una decision de equipo y se hace aqui, nunca con un `eslint-disable`:
`tools/checks/no-limit-suppressions.mjs` lo rechaza.

```sh
just root-lint     # ESLint de la raiz, este paquete incluido
just tools-test    # tests de las reglas propias y de tools/checks
```
