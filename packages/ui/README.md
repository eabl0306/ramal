# @git-rudder/ui

Tokens, temas, densidades, iconos y primitivas accesibles que comparten escritorio, web y los
webviews de VS Code. La diferencia entre hosts es un fichero de tema, un atributo de densidad y
un juego de iconos, nunca un `if (vscode)` en un componente.

## Montarlo en un host

```tsx
import '@git-rudder/ui/themes/git-rudder.css'; // VS Code: '@git-rudder/ui/themes/vscode.css'
import '@git-rudder/ui/themes/fonts'; // solo escritorio y web: IBM Plex empaquetada
import { lucideIcons } from '@git-rudder/ui/icons/lucide'; // VS Code: codicons de '@git-rudder/ui/icons/codicons'
import { I18nProvider, IconProvider, ToastProvider } from '@git-rudder/ui';

// <html data-theme="dark|light" data-density="comfortable|compact" data-host="web">
<I18nProvider locale={locale}>
  <IconProvider icons={lucideIcons}>
    <ToastProvider>{app}</ToastProvider>
  </IconProvider>
</I18nProvider>;
```

- `data-theme` solo aplica a `git-rudder.css`; `vscode.css` sigue el tema del editor, alto contraste
  incluido.
- `data-host="web"` sube la letra a 14 px y aplica los matices del claro de la web.
- Escritorio y web: `comfortable`; VS Code: `compact`. `DENSITIES` da los mismos numeros en
  JavaScript para el SVG del grafo, y un test comprueba que coinciden con el CSS.

## Que hay

| Pieza      | Contenido                                                                                                                                                                                                           |
| ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Tokens     | `--r-*` semanticos (architecture.md 7.1, lista en `tokens/tokens.ts`), `cssVar`, `laneColor` (con modulo, gaps B-14)                                                                                                |
| Temas      | `themes/git-rudder.css` con los oklch del handoff y `themes/vscode.css` sobre `--vscode-*`                                                                                                                          |
| Idioma     | `I18nProvider`, `useT` y `useLocale` sobre `@git-rudder/i18n`; tambien fija el idioma de React Aria                                                                                                                 |
| Iconos     | `<Icon name>` con `IconName` cerrado; juegos en `icons/lucide` y `icons/codicons`                                                                                                                                   |
| Primitivas | `Button`, `IconButton`, `Badge`, `PlatformBadge`, `PrStateBadge`, `FileStatusBadge`, `Chip`, `Toggle`, `SegmentedControl`, `TextField`, `Modal`, `ContextMenu` con `useContextMenu`, `ToastProvider` con `useToast` |

Las primitivas estan hechas sobre React Aria Components (ADR 0011), que da teclado, foco y
lectores de pantalla. La version 1.21 inyecta un unico `<style>` fijo (el `touch-action` de lo
pulsable): escritorio lo permite por su hash en la CSP; los webviews de VS Code todavia no. El aspecto va en CSS Modules
que solo leen tokens: stylelint y `git-rudder/no-raw-colors` rechazan cualquier color fuera de
`src/themes`, y `git-rudder/no-jsx-literal-text` cualquier texto visible que no venga de `@git-rudder/i18n`.

React Aria trae textos propios en muchos idiomas; cada app deberia recortar su bundle a `es` y
`en` al construir.

```sh
corepack pnpm --filter @git-rudder/ui test:coverage   # 60% minimo
```
