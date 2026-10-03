# Briefs de diseno para Claude Design

Un brief por cada issue de diseno de la epica #2 (#62 a #71). Cada uno lleva lo necesario para que
Claude Design dibuje lo que falta sin adivinar: pantallas, estados, textos en castellano e ingles,
reglas y que exportar. El detalle de por que falta cada cosa esta en [`../gaps.md`](../gaps.md).

**Estado (2026-10-02).** El handoff de los diez briefs llego el 2026-10-01 en un solo zip, y su
segunda ronda el 2026-10-02. Los dos estan en [`../handoff/`](../handoff/README.md): sustituyen al
original e incluyen los prototipos base corregidos. La columna "Estado" del [indice](#indice) dice
que issues quedan disenados. La fase de diseno se cerro el 2026-10-02 sin mas rondas: lo que no se
llego a disenar se resuelve en codigo al implementar, con los tokens y componentes de los
prototipos. El indice es el issue #523 y el detalle fila a fila, la columna "Diseno" de
[`../gaps.md`](../gaps.md).

## Como usarlos

1. Abre en Claude Design **el mismo proyecto** donde se hizo el handoff
   ([`../handoff/`](../handoff/README.md)). Sus prototipos son la fuente de verdad visual: tokens,
   medidas y componentes se reutilizan, no se reinventan.
2. Pega un brief entero como mensaje. Uno cada vez: espera el resultado y compruebalo contra su
   seccion "Hecho cuando" antes de pasar al siguiente.
3. Orden: **empieza por #62**, porque sus estados (carga, error, validacion, vacios, sin conexion,
   sincronizacion, foco) los usan todos los demas. Despues, cualquier orden. Sugerencia: #71 justo
   despues de #62, para que el resto parta de los prototipos ya corregidos.
4. Exporta el zip y traelo al repo. La primera entrega llego como un solo zip con todo el proyecto y
   sus ficheros sustituyeron los de `docs/design/handoff/` (la carpeta la ignoran prettier y las
   comprobaciones). Comprueba que trae el README de pantallas y estados que pide la seccion
   "Entregable": en las dos primeras entregas faltaba, y el README del handoff lo suple.
5. Marca el checklist del issue en GitHub.

## Reglas comunes a todos los briefs

Cada brief repite las que mas le afectan; estas valen siempre.

- **Fuente de verdad.** Los prototipos y el README del handoff. Donde no coinciden, manda el
  prototipo. Las pantallas nuevas copian la cabecera de estilos y los componentes del prototipo del
  producto que toque.
- **Tokens.** Solo los de la tabla del README del handoff (en codigo, los `--r-*` de `@git-rudder/ui`).
  Ningun color nuevo; los fondos suaves son el color al 10 o 14 % con `color-mix`. `--info` informa,
  `--accent` es la accion principal y la marca, `--ok` exito, `--warn` aviso y `--danger` peligro. En
  VS Code, los valores Dark Modern y Light Modern del README, que en la extension seran variables
  `--vscode-*`.
- **Tipografia.** IBM Plex Sans y Plex Mono; base 13 px en Desktop y 14 px en la web. VS Code usa la
  fuente del sistema a 13 px.
- **Densidades.**

  | Medida           | Desktop y web (`comfortable`) | VS Code (`compact`) |
  | ---------------- | ----------------------------- | ------------------- |
  | Fila de lista    | 40 px                         | 28 px               |
  | Elemento de menu | 32 px                         | 24 px               |
  | Barra lateral    | 30 px                         | 22 px               |
  | Barra de tareas  | 62 px                         | 34 px               |

- **Claro y oscuro** en todas las pantallas y estados.
- **Bilingue.** Todo texto visible en castellano y en ingles. Los textos de interfaz van en castellano
  correcto, con tildes. Plurales con claves `_one` y `_other` sobre `{count}`; marcadores con nombre
  (`{branch}`, `{device}`), nunca `{n}` ambiguo; glifos y atajos fuera de las cadenas; nunca se trocea
  una traduccion para fabricar otra (ADR 0017). Sin emojis, sin guiones largos y sin "simulado".
- **Textos nuevos** en `git-rudder-data.js`, donde diga cada brief. Una clave existente con otro texto no se
  pisa: se crea otra.
- **VS Code: lo nativo se especifica, no se dibuja** (X-11). Paleta, quick input, notificaciones,
  barra de estado, arboles y `viewsWelcome` se describen con su texto. Solo se dibujan los webviews:
  grafo con detalle, editor de merge, crear PR y ajustes de Git Rudder (que incluye el panel de la
  boveda).
- **Acciones destructivas** con modal de peligro: titulo en pregunta, consecuencia en una caja, boton
  `--danger`, foco inicial en "Cancelar" (patron completo en el brief #71, seccion 10).
- **Decisiones de producto que el diseno no puede contradecir:**
  - Git Rudder funciona sin cuenta: las zonas de nube invitan a entrar en vez de desaparecer (X-06).
  - No hay cuentas de plataforma compartidas con el equipo (X-01) y la web solo ve y quita cuentas
    vinculadas (X-02).
  - Nadie puede recuperar los secretos de la boveda si se pierden la frase y el codigo, tampoco el
    soporte de Git Rudder (X-03).
  - Las sesiones y dispositivos no muestran ubicacion: dispositivo, sistema, app o navegador y ultima
    actividad (decision #53).
  - El selector de modelo de IA se rellena con la lista del proveedor y marca el recomendado por nivel:
    "Rápido" para mensajes de commit, "Equilibrado" para PRs y resumenes. Sin precios en la primera
    version; sin red ni cache, solo la lista (decision #54, X-10).
  - Las llaves de IA de la organizacion se ven de solo lectura en Desktop y VS Code, con la etiqueta
    "De la organización"; el admin fija su modelo por funcion (X-05, decision #54).
  - Git Rudder Pro es un plan de organizacion por asiento: USD 8 al mes facturado al ano (USD 96) o USD 10
    al mes, 14 dias de prueba sin tarjeta, minimo 1 asiento. Lo personal (Git local, PRs, IA con llave
    propia y sincronizacion personal) es gratis siempre (decision #51).
- **Dominio y datos de muestra.** La web es `https://git-rudder.com` y la API
  `https://api.git-rudder.com`. Los correos de muestra usan `example.com`. Nada de "App password" de Bitbucket ni de modelos de IA fijos.

## Formato de exportacion

- Un zip por brief: `git-rudder-handoff-<issue>-<slug>.zip`, con la misma estructura que el handoff
  original:
  - `README.md` solo con lo nuevo (pantallas, componentes, medidas y estados, con los ids de
    `gaps.md`).
  - Los `*.dc.html` nuevos o corregidos. Props comunes: `theme` (`dark`, `light`) y `lang` (`es`,
    `en`); segun el brief, `product` (`desktop`, `vscode`, `web`), `os` y `screen` o `state` (un valor
    por pantalla o estado, con el id de `gaps.md`).
  - `git-rudder-data.js` con los textos nuevos.
  - `support.js` sin cambios.
- Cada `*.dc.html` se abre en el navegador desde la misma carpeta, como los originales.

## Indice

| Issue | Brief                                                                              | Pantallas o estados                                              | Estado                                                                                                        |
| ----- | ---------------------------------------------------------------------------------- | ---------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| #62   | [62-estados-comunes.md](62-estados-comunes.md)                                     | 10 (C-01 a C-10), mas la regla B-61                              | Disenado. Lo menor va en #523                                                                                 |
| #63   | [63-boveda.md](63-boveda.md)                                                       | 10 (K-01 a K-10), mas los cambios X-03 y X-04                    | Cerrado; lo pendiente se resuelve al implementar (#523)                                                       |
| #64   | [64-entrada-y-ciclo-de-vida-del-repo.md](64-entrada-y-ciclo-de-vida-del-repo.md)   | 11 (D-01 a D-11)                                                 | Disenado. Lo menor va en #523                                                                                 |
| #65   | [65-credenciales-y-operaciones-de-git.md](65-credenciales-y-operaciones-de-git.md) | 13 (D-12 a D-15, D-17 a D-25)                                    | Disenado. Lo menor y las confirmaciones que faltan en Git se resuelven al implementar (#523)                  |
| #66   | [66-vistas-de-los-menus-y-diff-real.md](66-vistas-de-los-menus-y-diff-real.md)     | 13 (D-27 a D-33, D-35 a D-38, D-40 y D-41)                       | Cerrado; lo pendiente se resuelve al implementar (#523)                                                       |
| #67   | [67-ventana-ajustes-y-equipo.md](67-ventana-ajustes-y-equipo.md)                   | 9 (D-42 a D-49 y D-51)                                           | Cerrado; lo pendiente se resuelve al implementar (#523)                                                       |
| #68   | [68-contribuciones-vscode.md](68-contribuciones-vscode.md)                         | 22 estados (V-01 a V-22), mas la especificacion de X-11          | Disenado. La especificacion es este brief, lo nativo va en texto (X-11) y V-17 se aplazo. Lo menor va en #523 |
| #69   | [69-autenticacion-web.md](69-autenticacion-web.md)                                 | 12 (W-01 a W-09, W-33 a W-35)                                    | Disenado. Lo menor va en #523                                                                                 |
| #70   | [70-panel-sitio-equipo.md](70-panel-sitio-equipo.md)                               | 21 (W-11 a W-30 y W-36)                                          | Disenado. Las capturas a 2x y lo menor van en #523                                                            |
| #71   | [71-revision-de-bugs.md](71-revision-de-bugs.md)                                   | Ninguna nueva: 62 bugs revisados y las confirmaciones de peligro | Cerrado; lo pendiente se resuelve al implementar (#523)                                                       |
