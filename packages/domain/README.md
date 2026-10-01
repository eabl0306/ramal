# @ramal/domain

Logica pura de estado de interfaz que comparten la web, el escritorio y VS Code. Cero
dependencias: ni npm, ni `node:*`, ni DOM. Lo comprueban el compilador (`tsconfig.lib.json`
sin DOM ni tipos de Node), ESLint (boundaries) y `tools/checks/package-graph.mjs`.

| Modulo      | Que hace                                                                                               |
| ----------- | ------------------------------------------------------------------------------------------------------ |
| `auth`      | Validacion de los formularios de entrar y crear cuenta (codigos, no textos)                            |
| `downloads` | Sistema y arquitectura del visitante, y que descarga destacar                                          |
| `conflict`  | Resolvedor de conflictos: elecciones por linea, vacio explicito, salida sin duplicados, edicion manual |
| `rebase`    | Editor de rebase interactivo: reordenar, acciones, resumen, validacion inmediata y el todo de git      |
| `repos`     | Filtros y agrupacion de la lista de repositorios                                                       |
| `prs`       | Agregacion de pull requests por estado                                                                 |
| `time`      | Tramos de tiempo relativo (el texto lo pone `formatRelativeTime` de `@ramal/i18n`)                     |
| `secrets`   | Enmascarado de llaves                                                                                  |
| `commits`   | Contador de 72 grafemas, Conventional Commits y mensajes de merge, revert y cherry-pick                |
| `refs`      | Nombres de rama y tag segun `git check-ref-format`, y los validadores declarativos de `RepoViewHost`   |
| `platforms` | La unica tabla de plataformas Git                                                                      |

Todo devuelve codigos o claves, nunca textos visibles: el texto lo pone la interfaz con
`@ramal/i18n`. El motor de Rust vuelve a validar lo que importa (plan de rebase, marcadores);
esto es la respuesta inmediata.

```sh
corepack pnpm --filter @ramal/domain test:coverage   # 80% minimo
```
