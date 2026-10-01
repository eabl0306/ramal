# @ramal/tsconfig

Las configuraciones base de TypeScript. Cada paquete y cada app extiende una y no relaja sus
reglas: si una estorba, se discute aqui.

| Fichero        | Para que                                                                        |
| -------------- | ------------------------------------------------------------------------------- |
| `base.json`    | Reglas estrictas comunes, con `erasableSyntaxOnly` (sin enum ni namespace)      |
| `library.json` | Paquetes puros (`domain`, `i18n`, `bridge`): sin DOM y sin tipos de Node        |
| `react.json`   | Codigo de navegador con React: `ui`, `repo-view`, la web, escritorio y webviews |
| `node.json`    | Codigo de Node: el host de la extension, scripts y configuracion                |

`library.json` hace que el compilador compruebe la pureza: si el dominio usa `document` o
`process`, el typecheck falla.

Cada proyecto es `composite` y solo referencia los paquetes que le permite
`docs/architecture.md` (5.2). El `tsconfig.json` de la raiz solo lista las referencias, para
`tsc -b`.
