# Cambios sobre los pendientes de #63 a #71 (2026-10-02)

Sustituye los ficheros del handoff (`docs/design/handoff/`) por los de esta carpeta. Lo que queda abierto va al final.

## #70 Web
- `Ramal Web.dc.html`: se quitó el panel antiguo. Iniciar sesión, OAuth y `startPage=dashboard` llevan a `Ramal Web Panel.dc.html` (cierra X-01, X-02, B-37, B-38 y B-58).
- Panel: el selector de organización dice "Nueva organización…" y el aviso de correo sin confirmar usa `auth.vBanner`, `auth.vResend` y `auth.vResent`. La etiqueta del proveedor ya no sale como `{serviceProvider}`: `t()` buscaba `legal.provider` antes que las claves locales.
- W-18: chips de plan `trial_last_day` (la banda pasa a `--warn` y usa el plural `_one`) y `active_monthly` (`biMonthlyLine` y `biPriceMonth`). El modal de asientos cobra según el ciclo.
- W-22: variante `rotacion-recomendada` con la banda `dvRotate`. Al cerrar la sesión de otro dispositivo, la banda aparece en Cuenta.
- W-28 `404-panel`: barra lateral del panel a partir de 900 px.

## #71 Bugs
- `Ramal.dc.html`: "Cerrar sesión" abre K-09 (`k9Title`, `k9Body` y la casilla `k9Check`, apagada). `marta@ramal.dev` pasa a `marta@example.com`.
- `Ramal Git.dc.html`: "Abortar" (merge, rebase, cherry-pick, revert y el rebase del squash) y "Eliminar stash" confirman con el modal de peligro, con el foco en Cancelar. Los títulos de peligro ya no se concatenan (antes salía "¿Remove lock and retry?").
- VS Code B-10: el primer resumen falla con "Reintentar" y la nota "no se ha cobrado". B-23, B-26 y B-27 ya estaban en el prototipo: etiquetas por operación, los menús actúan sobre la rama del elemento y "Eliminar rama remota…" va aparte.

## #63 Bóveda
- K-05: con "Sin bóveda" y "Categoría apagada" ya no aparece la bóveda bloqueada. Las cuentas salen como "Solo en este equipo" y una nota explica el motivo (`vaultNone` o `accOffD`).
- K-04: nueva fila "token rotado en otro equipo" con "Reconectar" (enlaza a D-14). Cada cuenta muestra sus permisos y "Usada en N repos".
- VS Code (`product=vscode`): tarjeta nativa en K-01 (`vsSyncPrompt` con "Configurar la bóveda" y "Ahora no") y en K-04 `useLocked` (notificación con "Desbloquear"). Las listas pasan a filas de una columna.

## #64 Entrada
- D-03 dentro de la tarjeta de D-01: variante `onboarding` de `d03-navegador`, y el mismo recorrido desde "Iniciar sesión" en D-01. Cancelar vuelve a D-01 y el éxito pasa a D-01 identidad.
- D-05 `narrow`: ventana de 1024 px, tarjetas de acción apiladas y menos relleno.
- D-06 `grid`: tarjeta de rejilla propia. Los no clonados llevan borde discontinuo.

## #67 Ajustes
- D-42 `d42-minimo` a 1024 px: barra lateral de 196, panel de 252, búsqueda de 132 y sin columna de autor. Los mensajes ya no se cortan.
- `ramal-data.js`: `AIPROV` ya no trae modelo por defecto. `Ramal.dc.html`: el formulario de llave (X-10) quita el alcance (siempre personal; las de la organización se gestionan en la web) y el modelo se elige de la lista del proveedor.

## #68 VS Code
- V-13 Historial de archivo: estados `V-13` y `V-13-sin-historial`, y el menú "Historial del archivo" abre la pestaña "Historial: {file}" con carga, filas filtradas, detalle de un solo archivo y vacío (`noHistory`).
- Sin sesión ya no aparece la llave de la organización (ni en el estado ni al cerrar sesión).

## #66 Vistas
- Chip "Estado" en D-27 a D-31: Cargando, Vacío, Error y, en D-30 y D-31, Sin cuenta.
- D-40: Primera carga, Error, "Cargar más" y aristas que siguen en las filas de carga.
- D-41: 14 carriles (los carriles a partir del 10 se comprimen a 12 px y el color va con módulo, B-14).
- Un diff por archivo: en D-33, D-37 y D-38, clic en un archivo del panel derecho cambia el diff (con su cabecera @@ y sus contadores).
- D-37: "Guardar en stash los seleccionados" abre su modal.

## Sigue abierto
- #66: casos menores de D-32, D-35, D-36 y D-38, plurales y repaso de formularios (B-04).
- #68: la especificación de contribuciones pasada a limpio, y V-02, V-03, V-04, V-10 y V-17 solo en textos.
- #70: capturas de la landing a 2x sin anotaciones (hay que regenerarlas a doble densidad).
- #62, #65 y #69: los menores del README.
