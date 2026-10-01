# Handoff: Ramal — cliente Git (Desktop, extensión VS Code y web)

## Overview
Ramal is a Git client (functionally inspired by GitKraken) made of three products that share one account system:

1. **Ramal Desktop** (Electron/Tauri): repository list with search and filters, workspaces with aggregated PRs, a commit graph of every branch, staging and commits, side-by-side diff, a line-by-line merge conflict resolver, interactive rebase, context menus for everything, and settings (profile, Ramal account and team, Git platform accounts, AI keys, Git options, appearance).
2. **Ramal for VS Code** (extension): the same repo view without the repo list or workspaces, since the repo is the open folder. It follows the VS Code theme and language (falls back to English).
3. **Ramal Web**: marketing site with per-platform downloads, sign-up and login, and a cloud dashboard (overview and devices, workspaces, repositories, linked accounts, team, cloud-sync settings).

The UI is bilingual (ES/EN) everywhere and supports dark and light themes.

## About the Design Files
The files in this bundle are **design references built in HTML**: working prototypes that show the intended look and behavior. They are not production code to copy directly. The task is to **recreate these designs in the target codebase** with its own patterns and libraries. If there is no codebase yet, pick the most suitable stack. Recommended:
- **Desktop:** Electron or Tauri + React + TypeScript. Git via `isomorphic-git`/`simple-git` (Electron) or `git2-rs` (Tauri).
- **VS Code extension:** extension host for Git and commands, with webviews for the graph, merge editor and settings, using `--vscode-*` CSS variables.
- **Web:** Next.js (or similar) with auth (email plus GitHub/GitLab/Google OAuth) and an API for syncing settings.

The prototypes are "Design Components" (`*.dc.html`). Each file has a template (markup with `{{ }}` holes, `<sc-for>` for loops and `<sc-if>` for conditionals) and a `class Component` holding all the state and logic in `renderVals()`. `support.js` is the runtime that renders them. Open any `.dc.html` in a browser, from the same folder, to try it. All data is fake (`ramal-data.js`).

## Fidelity
**High fidelity.** Colors, typography, spacing, states and interactions are final. Recreate them pixel-perfect with the target stack's libraries.

---

## Design Tokens

### Typography
- UI: **IBM Plex Sans** 400/500/600/700 (Google Fonts). Base 13px (desktop app), 14px (web).
- Code, SHAs, paths, commands: **IBM Plex Mono** 400/500/600.
- VS Code extension: system font `-apple-system, BlinkMacSystemFont, 'Segoe UI'` 13px; code `Menlo/Consolas`.
- Scale in use: 10, 11 (uppercase section labels, `letter-spacing:.06em`, weight 600), 12, 12.5 (code), 13, 14, 15, 16, 17, 18, 20, 24 (H1 `letter-spacing:-0.02em`), 28–40 (web H2), `clamp(38px,6vw,66px)` (web hero, `letter-spacing:-0.035em`, line-height 1.04).

### Colors — Desktop and Web (CSS variables on `body`)
| Token | Dark | Light |
|---|---|---|
| --bg | oklch(0.185 0.008 230) | oklch(0.965 0.004 230) (web 0.975) |
| --panel | oklch(0.21 0.009 230) | oklch(0.985 0.003 230) |
| --surface | oklch(0.24 0.01 230) | oklch(1 0 0) |
| --surface2 (hover) | oklch(0.285 0.012 230) | oklch(0.94 0.006 230) |
| --border | oklch(0.315 0.012 230) | oklch(0.89 0.008 230) |
| --text | oklch(0.95 0.005 230) | oklch(0.24 0.012 230) |
| --muted | oklch(0.76 0.012 230) | oklch(0.44 0.012 230) |
| --faint | oklch(0.62 0.012 230) | oklch(0.54 0.01 230) |
| --accent (teal) | oklch(0.74 0.12 190) | oklch(0.54 0.1 190) |
| --accent-ink | oklch(0.2 0.03 190) | oklch(0.99 0.005 190) |
| --accent-soft | accent / 0.14 | accent / 0.10 |
| --ok | oklch(0.76 0.15 150) | oklch(0.52 0.13 150) |
| --warn | oklch(0.8 0.14 80) | oklch(0.58 0.13 70) |
| --danger | oklch(0.7 0.17 25) | oklch(0.55 0.18 25) |
| --info | oklch(0.74 0.12 245) | oklch(0.52 0.13 250) |
| --add-bg / --del-bg | ok / danger at 13–14 % alpha | same, 12–13 % |

**Graph lane colors** (vivid, same in both themes):
`oklch(0.72 0.15 190)`, `oklch(0.68 0.2 300)`, `oklch(0.76 0.16 60)`, `oklch(0.68 0.21 355)`, `oklch(0.74 0.18 140)`, `oklch(0.7 0.16 250)`.

**Platform badges:** GitHub `oklch(0.5 0.03 260)` "GH", GitLab `oklch(0.64 0.17 40)` "GL", Bitbucket `oklch(0.56 0.15 255)` "BB", Gitea `oklch(0.6 0.14 145)` "GT", Azure DevOps `oklch(0.58 0.14 235)` "AZ".

**PR states:** open → info, draft → faint, approved → ok, changes requested → warn, merged → `oklch(0.68 0.18 300)`, closed → danger. Badge background = the color at 14 % (`color-mix(in oklch, C 14%, transparent)`).

**File status:** M → warn, A → ok, D → danger, R → info.

### Colors — VS Code extension
Uses VS Code Dark Modern / Light Modern values: editor `#1f1f1f`/`#ffffff`, sidebar `#181818`/`#f8f8f8`, border `#2b2b2b`/`#e5e5e5`, fg `#cccccc`/`#3b3b3b`, accent `#0078d4`/`#005fb8`, list selection `#04395e`, git decorations M `#e2c08d`, A `#73c991`, D `#c74e39`, conflict `#e4676b`. **In the real extension, use the `--vscode-*` variables** (e.g. `--vscode-editor-background`, `--vscode-gitDecoration-modifiedResourceForeground`, `--vscode-button-background`) and not the hex values.

### Radius, spacing, shadows
- Radius: 5–7 (chips, small buttons, menu items), 8–9 (inputs, buttons), 10–12 (cards, menus), 14 (large panels), 16–18 (modals, auth card).
- Heights: buttons 30/34/36/38/42/48; inputs 32–42; menu items 32 (desktop) / 24 (VS Code); graph rows 40 (desktop) / 28 (VS Code); sidebar items 30 / 22 (VS Code).
- Content padding: 32px × 36px, max width 1120–1200 (desktop home), 900 (settings), 1060 (web dashboard), 1160 (web landing).
- Shadow: `0 12px 40px oklch(0.05 0.01 230 / .5), 0 2px 6px oklch(0.05 0.01 230 / .3)` (dark); light uses oklch(0.3 0.02 230 / .16).

---

## Screens — Ramal Desktop (`Ramal.dc.html`)

### Global layout
- **Title bar / tabs (46px, --panel):** "R" logo (24px, teal, radius 8) + "Ramal". "Inicio" tab, a "⚙︎ Configuración" tab (when open), one tab per open repo (platform badge + name + ×, max 220px), "+". Active tab: `background:var(--bg)` + `box-shadow: inset 0 2px 0 var(--accent)`. On the right: ES/EN segmented control, ☀︎/☾ icon theme switch, ⚙︎, avatar (opens Profile).
- **Context menu (whole app):** 236px min, radius 12, items 32px. Submenus open on hover with "›". Danger items use --danger. Items with a check (✓) can stay open (`keep`) for multi-select. Opens with right-click and with the `⋯` buttons. Esc closes it.
- **Generic modal:** 460px (580 wide), radius 16. Field types: info, text, textarea, select, segmented, checkbox, action (AI button). Primary button uses accent, or danger for destructive actions.
- **Toasts:** bottom-right, colored dot (ok/warn/info), 3.2 s.

### 1. Home › Repositories
- Sidebar 232px: "Repositorios (n)", WORKSPACES section with a color dot and count, "+ Nuevo workspace".
- Header: H1 + subtitle + buttons Abrir / Iniciar / **Clonar** (primary).
- **Filter bar on a single line** (card with 8px padding): search (flex:1, min 140px) plus 5 **dropdown** buttons: Tipo (Todos/Local/Remoto), Plataforma (multi-select), Cuenta, Workspace (including "Sin workspace"), Agrupar (Nada/Workspace/Plataforma/Cuenta). Each button shows a 10px caption and the value in 600 below it (max 120px with ellipsis). Active filter: accent border + accent-soft. Below: "N repositorios · Limpiar filtros".
- **Repo card** (grid `34px | 1.5fr | 1.2fr | 1fr | 224px`, gap 14, padding 14/12/14/16, radius 12, hover border accent): platform badge 34px (or "⌂" if local) · **alias** (600, 14px) + real name in mono + path in mono · "GitHub · @account" + host · account label · "⎇ branch" in accent mono + "↑2 ↓0 · N cambios" · workspace chip (first one + "+n") + "hace 5 min" + `⋯`.
- Menu: Abrir, Asignar alias…, Renombrar…, **Workspaces ›** (multiple checkboxes, stays open, "Quitar de todos", "+ Nuevo workspace"), Fetch, Mostrar en carpeta, Abrir terminal, Copiar ruta, Quitar de la lista… (confirmation with "mover carpeta a la papelera" option).
- **A repo can belong to several workspaces** (`wss: string[]`).

### 2. Home › Workspace
- Header with color, name, "N repos · M PRs abiertos", Fetch de todos, Abrir todos, ⋯ (rename/delete).
- Repo cards grid (auto-fill minmax 250).
- **Aggregated PRs** across every repo in the workspace: state chips with counts. Table columns: repo (badge) | #num (GitLab uses "!num") + title + `from → to` | @author | state badge | checks (✓ Correctos / × Fallidos / ● En curso) | reviews n/m | updated. Clicking a row opens the repo.

### 3. Repository view
- **Toolbar (62px):** repo selector ▾ and branch ▾ (accent mono) · buttons with glyph + label: Deshacer ↶, Rehacer ↷ | Fetch ⟳, Pull ↓ (▾ fetch all / ff / ff-only / rebase), Push ↑ (▾ force-with-lease, upstream; "ahead" badge) | Rama ⎇, Stash ⊟, Pop ⊞ (count badge) | PR ⇄ · commit search (dims the rows that don't match).
- **Left sidebar** (`clamp(200px,20vw,264px)`): filter box + collapsible sections LOCAL (✓ HEAD, ↑ahead), REMOTOS (grouped by remote), PULL REQUESTS, STASHES, TAGS (◆), SUBMÓDULOS. Click = select commit, double-click = checkout, right-click = menu.
- **Center — graph:** columns `200px refs | 130px graph | 1fr message | 150 author | 110 date | 76 sha`, **rows 40px**. Ref pills (lane color, 14 % background, 32 % if HEAD; icons ✓ HEAD, ⇅ remote, ◆ tag), max 2 + "+n". **WIP** row at the top ("// WIP · N archivos", dashed node). **Graph drawn in SVG:** X = 16 + lane·20, Y = row·40 + 20. Same lane → straight line. Branch-off (parent in a lower lane) → vertical, then a curve on the last row. Merge → curve on the first row, then vertical. Nodes r=9 with author initials (7.5px, 700), merges r=5, selected node gets an outer ring. Tweak `graphNodes: avatar|dot`.
- **Right panel** (`clamp(280px,26vw,360px)`):
  - *WIP:* "Sin preparar (n)" / "Preparados (n)" with Preparar todo / Quitar todo, rows with status letter + name + dir + [+/−] button. Commit box: "✦ Generar con IA" button, summary with a 72-character counter, description, amend checkbox, "Commit de N archivos en <rama>" (disabled until a file is staged and a summary written).
  - *Commit:* SHA + Copiar, message, **AI summary** (optional), author, parents, Cherry-pick / Revert / Crear rama, files with +/−.
- **Diff:** "← Grafo", status, path, +/−, **Lado a lado** (default) / Unificada, stage/unstage button. Side by side: `48px num | 18px sign | code` ×2, deleted lines on --del-bg, added on --add-bg, missing side shown as a diagonal hatch.
- **Conflict resolver (key feature):**
  - Header: "← Grafo" (you can **go back to the graph at any time**), ⚠, "Conflicto en <file>", "A: current ← B: incoming", Todo de A / Todo de B.
  - Top half: **A (Tu rama)** | **B (Entrante)** side by side, full file with line numbers. Each conflict block has a header ("Conflicto n" + "Tomar bloque"/"Quitar bloque") and every line has a **+** (add to output) / **−** (remove) button, with the lane color on a 3px inset left bar.
  - Bottom half: **Resultado** ("así quedará <file> al completar el merge"). Updates live. Each line shows a bar for its origin and a **−** button to remove it. An unresolved block is shown as a dashed warn box. "✎ Editar a mano" turns it into a textarea (Listo / Restablecer).
  - Right panel: "Merge en curso", list of files (resolved ✓ / in conflict !), "Completar merge" (enabled when everything is resolved; creates a two-parent merge commit), "Abortar merge".
- **Interactive rebase:** list with drag and drop (plus ↑↓), action select `pick/reword/edit/squash/fixup/drop` with its own color, input when the action is reword, strikethrough when drop, summary and Iniciar rebase.
- **Commit menu:** Checkout, Crear rama aquí, Crear tag aquí | Cherry-pick, Revert, Reset › soft/mixed/hard (hard asks for confirmation), Rebase interactivo | Merge (if it's a branch) | Copiar SHA / mensaje.
- **Branch menu:** Checkout, Merge X en actual, Rebase actual sobre X, Rebase interactivo | Push, Pull, Upstream | Crear PR, Comparar | Renombrar, Copiar nombre, Eliminar.
- Other menus: file (stage, view diff, discard, history, blame, .gitignore, editor, copy path), stash (apply/pop/drop), tag, remote, submodule, PR.

### 4. Settings (its own tab, left nav with groups)
- **Usuario:** *Perfil* (64px avatar, name, username, email, time zone, Guardar). *Cuenta y equipo* (Ramal Pro plan + Gestionar suscripción, session + Cerrar sesión, "ACME Studio" organization with members, roles and pending invitations, Invitar, active sessions).
- **Conexiones:** *Plataformas Git* (groups per platform, several accounts each: @user, label, host, method OAuth/Token/SSH, scopes, "Usada en N repos", Predeterminada / Hacer predeterminada, menu to test/edit/disconnect). *IA*: keys (provider: Anthropic, OpenAI, Groq, Google Gemini, Mistral, OpenRouter, Ollama local, any OpenAI-compatible; label; masked key; model; **belongs to: Personal / Organización**; default). "Dónde usar la IA": toggle + key select for Mensajes de commit, Pull requests, Resumen de commits. Preferences: language, Conventional Commits, exclude sensitive files, extra instructions.
- **Aplicación:** *Git* (user.name/email, default branch, sign commits, auto fetch, default pull, prune, SSH keys). *Apariencia* (theme, language, graph nodes).
- Generic settings row: label + description on the left, control (input / select / segmented / toggle 38×22 / button / textarea) on the right, wrapping on narrow widths.

## Screens — Ramal for VS Code (`Ramal VS Code.dc.html`)
- Title bar with the command center (opens the **command palette**, ⇧⌘P). Activity bar 48px with the Ramal "R" icon active (inset 2px accent). ⚙︎ at the bottom opens settings.
- **"RAMAL" view** (`clamp(230px,24vw,300px)`) with 22px section headers: CAMBIOS (commit textarea with ✦ IA, Commit button ▾ Commit/Commit y push/Amend; groups "Cambios de merge" (C), "Cambios preparados", "Cambios", each with inline +/−), RAMAS, REMOTOS, PULL REQUESTS, STASHES, TAGS.
- **Editor tabs:** "Ramal Graph" (pinned; compact toolbar with icon-only buttons, 28px rows, detail pane on the right), "Merge: <file>" (same resolver as desktop), "<file> (Working Tree)" diff, "Ramal Settings" (styled like the VS Code settings editor: "Ramal › AI: **Commit Messages**" + checkbox/select; sections Cuenta de Ramal and sync, Plataformas Git, IA, Git). No appearance or language settings: they **follow VS Code**.
- Status bar: remote indicator, "⎇ branch*", "⟳ 0↓ 2↑" (push), "⚠ N conflictos", "✦ <AI provider>", "Ramal: @account".
- Dialogs use the quick-input style (centered at the top, 600px). Notifications bottom-right with "Origen: Ramal".
- Language: `vsLanguage` es → Spanish, **anything else → English**.

## Screens — Ramal Web (`Ramal Web.dc.html`)
- Sticky nav 64px with blur: logo, Funciones / Descargar / VS Code, ES/EN, ☀︎/☾, Iniciar sesión, **Crear cuenta** (or "Abrir panel" when logged in).
- **Landing:** v2.4 pill, hero H1 + subtitle, "Descargar para <OS detectado>" + "Todas las plataformas" + link to VS Code. Product preview (the real app in a window frame). Six features (grid with 1px borders, numbered 01–06). VS Code section with a preview and `code --install-extension ramal.ramal`. **Downloads:** macOS (Apple Silicon/Intel/Universal, `brew install --cask ramal`), Windows (x64/ARM64/.msi/portable, `winget install Ramal.Ramal`), Linux (.deb/.rpm/AppImage/Flathub/ARM64, `flatpak install flathub io.ramal.Ramal`), VS Code (Marketplace, Open VSX, .vsix). The card for the user's OS is highlighted ("Tu sistema"). Account CTA and footer.
- **Auth** (420px card): OAuth GitHub/GitLab/Google, "o con email", fields. Login: email, password + "¿La olvidaste?". Sign-up: name, email, password with an 8+ character hint, organization (optional), terms. Validation: required name, email regex, password ≥ 8, terms accepted. Error shown in a danger box.
- **Dashboard:** top bar 56px (organization switcher, Descargar, ES/EN, theme, avatar menu → Cerrar sesión). Nav: Resumen (counter tiles + synced devices), Workspaces (cards with repo chips, **Compartido con equipo / Solo tú** toggle, rename/delete), Repositorios (search; table repo | origin (platform + @account) | workspaces (multiple) | updated | ⋯ with workspace checkboxes and unlink), Cuentas vinculadas (per platform, owner Tuya/Equipo, test/share/disconnect), Equipo (used seats with a bar, members with a role select, invite, resend, remove), Configuración en la nube (profile; what syncs: workspaces and repos, accounts (encrypted tokens), AI preferences, AI keys (**off by default: they stay on each device**), Git, appearance; export JSON; delete account).

---

## State Management (main models)
```ts
Account   { id, platform: 'github'|'gitlab'|'bitbucket'|'gitea'|'azure', user, name, host, method, scopes, def: boolean, team?: boolean }
Workspace { id, name, color, shared?: boolean }
Repo      { id, name, alias, path, acc: Account.id|null, wss: Workspace.id[], branch, ahead, behind, changes, updatedAt }
PR        { repo, num, title, author, from, to, state: 'open'|'draft'|'approved'|'changes'|'merged'|'closed', checks: 'pass'|'fail'|'pending', reviews }
Commit    { id, sha, lane, parents: id[], msg, author, date, refs: {name, type:'local'|'remote'|'tag', head?}[] }
Conflict  { src, file, choices: { [fileIdx]: { ours: boolean[], theirs: boolean[] }[] /* per hunk, per line */ }, manual: {[fileIdx]: string} }
AIKey     { id, prov, label, scope: 'personal'|'org', masked, model, def }
AIConfig  { commit, pr, summary: boolean; commitKey, prKey, summaryKey; lang: 'app'|'es'|'en'; conventional; instructions; exclude }
Member    { name, email, role: 'owner'|'admin'|'member', pending? }
```
- A file counts as resolved when every hunk has at least one selected line, or when it has a manual edit with no `<<<<<<<`/`=======`/`>>>>>>>` markers left.
- The resolver output is the context lines plus, for each hunk, the selected lines of A followed by the selected lines of B, in their original order.
- Real lane assignment for the graph: see the `DIFF` sample in `ramal-data.js` (the reuse-free-lanes algorithm).
- Web ↔ app sync: account + workspaces + repos (without local paths from other devices) + accounts (encrypted tokens) + preferences. AI keys are not synced unless the user turns it on.

## Interactions to keep
- Right-click on every entity. `⋯` opens the same menu. Esc closes menus, modals and the palette.
- Workspaces menu with checkboxes that stays open.
- Merge with conflicts → resolver; you can leave it with "← Grafo" and come back through the WIP row or the file list.
- AI: disabled feature or no key → warning toast. Generation runs ~800 ms with "Generando…" and fills the summary/description or the PR title/description in the language set in AI preferences.
- Toggle knob `transition: transform .15s`. Toasts `toastIn .2s ease-out`.

## Assets
No images. Icons are Unicode glyphs (⎇ ⟳ ↓ ↑ ⊟ ⊞ ⇄ ◆ ⇅ ✦ ⚙︎ ☀︎ ☾ ⋯). **In production, replace them with a consistent icon set** (e.g. Lucide or Codicons in VS Code). The "R" logo is a placeholder.

## Files
- `Ramal.dc.html` — desktop app (home, workspaces, repo, diff, conflicts, rebase, settings).
- `Ramal VS Code.dc.html` — VS Code extension (props `vsTheme`, `vsLanguage`).
- `Ramal Web.dc.html` — landing, auth and dashboard (props `theme`, `lang`, `startPage`).
- `ramal-data.js` — fake data and the complete ES/EN dictionary (`D`). Reuse the keys as the base for real i18n.
- `support.js` — runtime needed to open the `.dc.html` files in a browser (not part of the product).
