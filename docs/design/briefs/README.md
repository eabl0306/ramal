# Briefs de diseno para Claude Design

Un brief por cada issue de diseno de la epica #2 (#62 a #71). Cada uno lleva lo necesario para que
Claude Design dibuje lo que falta sin adivinar: pantallas, estados, textos en castellano e ingles,
reglas y que exportar. El detalle de por que falta cada cosa esta en [`../gaps.md`](../gaps.md).

## Como usarlos

1. Abre en Claude Design **el mismo proyecto** donde se hizo el handoff original
   ([`../handoff/`](../handoff/README.md)). Los tres prototipos (`Ramal.dc.html`,
   `Ramal VS Code.dc.html` y `Ramal Web.dc.html`) son la fuente de verdad visual: tokens, medidas y
   componentes se reutilizan, no se reinventan.
2. Pega un brief entero como mensaje. Uno cada vez: espera el resultado y compruebalo contra su
   seccion "Hecho cuando" antes de pasar al siguiente.
3. Orden: **empieza por #62**, porque sus estados (carga, error, validacion, vacios, sin conexion,
   sincronizacion, foco) los usan todos los demas. Despues, cualquier orden. Sugerencia: #71 justo
   despues de #62, para que el resto parta de los prototipos ya corregidos.
4. Exporta el zip con el nombre que pide la seccion "Entregable" del brief y traelo al repo:
   descomprimelo en `docs/design/handoff/<issue>-<slug>/` (esa carpeta ya la ignora prettier). El de
   #71 sustituye los ficheros base de `docs/design/handoff/`.
5. Marca el checklist del issue en GitHub.

## Reglas comunes a todos los briefs

Cada brief repite las que mas le afectan; estas valen siempre.

- **Fuente de verdad.** Los prototipos y el README del handoff. Donde no coinciden, manda el
  prototipo. Las pantallas nuevas copian la cabecera de estilos y los componentes del prototipo del
  producto que toque.
- **Tokens.** Solo los de la tabla del README del handoff (en codigo, los `--r-*` de `@ramal/ui`).
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
- **Textos nuevos** en `ramal-data.js`, donde diga cada brief. Una clave existente con otro texto no se
  pisa: se crea otra.
- **VS Code: lo nativo se especifica, no se dibuja** (X-11). Paleta, quick input, notificaciones,
  barra de estado, arboles y `viewsWelcome` se describen con su texto. Solo se dibujan los webviews:
  grafo con detalle, editor de merge, crear PR y ajustes de Ramal (que incluye el panel de la
  boveda).
- **Acciones destructivas** con modal de peligro: titulo en pregunta, consecuencia en una caja, boton
  `--danger`, foco inicial en "Cancelar" (patron completo en el brief #71, seccion 10).
- **Decisiones de producto que el diseno no puede contradecir:**
  - Ramal funciona sin cuenta: las zonas de nube invitan a entrar en vez de desaparecer (X-06).
  - No hay cuentas de plataforma compartidas con el equipo (X-01) y la web solo ve y quita cuentas
    vinculadas (X-02).
  - Nadie puede recuperar los secretos de la boveda si se pierden la frase y el codigo, tampoco el
    soporte de Ramal (X-03).
  - Las sesiones y dispositivos no muestran ubicacion: dispositivo, sistema, app o navegador y ultima
    actividad (decision #53).
  - El selector de modelo de IA se rellena con la lista del proveedor y marca el recomendado por nivel:
    "Rápido" para mensajes de commit, "Equilibrado" para PRs y resumenes. Sin precios en la primera
    version; sin red ni cache, solo la lista (decision #54, X-10).
  - Las llaves de IA de la organizacion se ven de solo lectura en Desktop y VS Code, con la etiqueta
    "De la organización"; el admin fija su modelo por funcion (X-05, decision #54).
  - Ramal Pro es un plan de organizacion por asiento: USD 8 al mes facturado al ano (USD 96) o USD 10
    al mes, 14 dias de prueba sin tarjeta, minimo 1 asiento. Lo personal (Git local, PRs, IA con llave
    propia y sincronizacion personal) es gratis siempre (decision #51).
- **Dominio y datos de muestra.** La web es `https://ramal.bahoque.com` y la API
  `https://api.ramal.bahoque.com`. `ramal.dev` y `ramal.app` son de terceros: los correos de muestra
  usan `example.com`. Nada de "App password" de Bitbucket ni de modelos de IA fijos.

## Formato de exportacion

- Un zip por brief: `ramal-handoff-<issue>-<slug>.zip`, con la misma estructura que el handoff
  original:
  - `README.md` solo con lo nuevo (pantallas, componentes, medidas y estados, con los ids de
    `gaps.md`).
  - Los `*.dc.html` nuevos o corregidos. Props comunes: `theme` (`dark`, `light`) y `lang` (`es`,
    `en`); segun el brief, `product` (`desktop`, `vscode`, `web`), `os` y `screen` o `state` (un valor
    por pantalla o estado, con el id de `gaps.md`).
  - `ramal-data.js` con los textos nuevos.
  - `support.js` sin cambios.
- Cada `*.dc.html` se abre en el navegador desde la misma carpeta, como los originales.

## Indice

| Issue | Brief                                                                              | Pantallas o estados                                              |
| ----- | ---------------------------------------------------------------------------------- | ---------------------------------------------------------------- |
| #62   | [62-estados-comunes.md](62-estados-comunes.md)                                     | 10 (C-01 a C-10), mas la regla B-61                              |
| #63   | [63-boveda.md](63-boveda.md)                                                       | 10 (K-01 a K-10), mas los cambios X-03 y X-04                    |
| #64   | [64-entrada-y-ciclo-de-vida-del-repo.md](64-entrada-y-ciclo-de-vida-del-repo.md)   | 11 (D-01 a D-11)                                                 |
| #65   | [65-credenciales-y-operaciones-de-git.md](65-credenciales-y-operaciones-de-git.md) | 13 (D-12 a D-15, D-17 a D-25)                                    |
| #66   | [66-vistas-de-los-menus-y-diff-real.md](66-vistas-de-los-menus-y-diff-real.md)     | 13 (D-27 a D-33, D-35 a D-38, D-40 y D-41)                       |
| #67   | [67-ventana-ajustes-y-equipo.md](67-ventana-ajustes-y-equipo.md)                   | 9 (D-42 a D-49 y D-51)                                           |
| #68   | [68-contribuciones-vscode.md](68-contribuciones-vscode.md)                         | 22 estados (V-01 a V-22), mas la especificacion de X-11          |
| #69   | [69-autenticacion-web.md](69-autenticacion-web.md)                                 | 12 (W-01 a W-09, W-33 a W-35)                                    |
| #70   | [70-panel-sitio-equipo.md](70-panel-sitio-equipo.md)                               | 21 (W-11 a W-30 y W-36)                                          |
| #71   | [71-revision-de-bugs.md](71-revision-de-bugs.md)                                   | Ninguna nueva: 62 bugs revisados y las confirmaciones de peligro |
