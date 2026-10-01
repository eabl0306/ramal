window.RAMAL = (function () {
  const PLAT = {
    github: { label: 'GitHub', abbr: 'GH', color: 'oklch(0.5 0.03 260)' },
    gitlab: { label: 'GitLab', abbr: 'GL', color: 'oklch(0.64 0.17 40)' },
    bitbucket: { label: 'Bitbucket', abbr: 'BB', color: 'oklch(0.56 0.15 255)' },
    gitea: { label: 'Gitea', abbr: 'GT', color: 'oklch(0.6 0.14 145)' },
    azure: { label: 'Azure DevOps', abbr: 'AZ', color: 'oklch(0.58 0.14 235)' }
  };
  const LANE = ['oklch(0.72 0.15 190)', 'oklch(0.68 0.2 300)', 'oklch(0.76 0.16 60)', 'oklch(0.68 0.21 355)', 'oklch(0.74 0.18 140)', 'oklch(0.7 0.16 250)'];
  const AUTH = { MR: 'Marta Ruiz', DL: 'Diego León', AT: 'Aiko Tanaka', SO: 'Samuel Ortiz' };
  const RAW = [
    ['c1', 2, ['c3'], 'Añadir zoom y desplazamiento al grafo', 'MR', '2026-09-30 11:42', 'a41f9c2', [['feature/graph-ui', 'local', 1]]],
    ['c2', 1, ['c4', 'c5'], "Merge branch 'feature/auth' into develop", 'DL', '2026-09-30 10:15', '7be02d1', [['develop', 'local'], ['origin/develop', 'remote']]],
    ['c3', 2, ['c6'], 'Dibujar carriles con curvas bézier', 'MR', '2026-09-29 18:03', '3c9d8e4', [['origin/feature/graph-ui', 'remote']]],
    ['c4', 1, ['c7'], 'Actualizar Electron a la versión 38', 'SO', '2026-09-29 16:40', 'e12a7f0', [['origin/release/2.4', 'remote']]],
    ['c5', 3, ['c8'], 'Validar tokens OAuth por cuenta', 'AT', '2026-09-29 12:22', '58f3b19', [['feature/auth', 'local'], ['origin/feature/auth', 'remote']]],
    ['c6', 2, ['c7'], 'Estructura base del componente Graph', 'MR', '2026-09-28 17:55', '9d04c6a'],
    ['c7', 1, ['c9'], 'Configurar i18n es/en', 'DL', '2026-09-28 11:30', 'b7e11f3'],
    ['c8', 3, ['c9'], 'Soporte multi-cuenta para GitLab self-hosted', 'AT', '2026-09-27 19:10', '24ac9d7'],
    ['c9', 1, ['c11'], 'Extraer servicio de remotos', 'SO', '2026-09-27 10:02', 'f0b83e5'],
    ['c10', 0, ['c14', 'c12'], "Merge branch 'hotfix/login'", 'DL', '2026-09-26 20:45', 'c3d9a70', [['main', 'local'], ['origin/main', 'remote'], ['upstream/main', 'remote'], ['v2.3.1', 'tag']]],
    ['c11', 1, ['c13'], 'Tests del servicio de credenciales', 'AT', '2026-09-26 15:20', '6a2f4b8'],
    ['c12', 4, ['c14'], 'Corregir refresco de token expirado', 'DL', '2026-09-26 13:05', '0e7c5d2', [['hotfix/login', 'local']]],
    ['c13', 1, ['c14'], 'Iniciar ciclo 2.4', 'MR', '2026-09-25 09:12', 'd81b6e9'],
    ['c14', 0, ['c15'], 'Release 2.3.0', 'MR', '2026-09-24 18:00', '41c0a3f', [['v2.3.0', 'tag']]],
    ['c15', 0, ['c16', 'c17'], "Merge branch 'develop' into main", 'MR', '2026-09-23 17:44', '8f5e2c1'],
    ['c16', 0, ['c18'], 'Ajustar notarización en macOS', 'SO', '2026-09-23 12:31', 'a9d7e04'],
    ['c17', 1, ['c18'], 'Vista de stashes en la barra lateral', 'AT', '2026-09-22 16:08', '5b3c8f2'],
    ['c18', 0, ['c19'], 'Configurar pipeline de CI', 'SO', '2026-09-20 10:00', '2e6f1a9', [['v2.2.0', 'tag']]],
    ['c19', 0, [], 'Commit inicial', 'MR', '2026-09-18 09:00', '0a1b2c3']
  ];
  const mkCommits = () => RAW.map(r => ({ id: r[0], lane: r[1], parents: r[2], msg: r[3], au: r[4], date: r[5], sha: r[6], refs: (r[7] || []).map(x => ({ name: x[0], type: x[1], head: !!x[2] })) }));
  const ACCOUNTS = [
    { id: 'a1', platform: 'github', user: 'marta-dev', name: 'Personal', host: 'github.com', method: 'OAuth', def: true, scopes: 'repo, read:org, workflow' },
    { id: 'a2', platform: 'github', user: 'marta-acme', name: 'ACME Corp', host: 'github.com', method: 'Token (PAT)', def: false, scopes: 'repo, read:org' },
    { id: 'a3', platform: 'gitlab', user: 'mruiz', name: 'Empresa', host: 'gitlab.empresa.com', method: 'Token (PAT)', def: true, scopes: 'api, read_repository' },
    { id: 'a4', platform: 'gitlab', user: 'marta', name: 'Personal', host: 'gitlab.com', method: 'OAuth', def: false, scopes: 'api' },
    { id: 'a5', platform: 'bitbucket', user: 'marta_r', name: 'Freelance', host: 'bitbucket.org', method: 'App password', def: true, scopes: 'repository:write, pullrequest:write' },
    { id: 'a6', platform: 'gitea', user: 'marta', name: 'Homelab', host: 'git.casa.lan', method: 'SSH + Token', def: true, scopes: 'write:repository' },
    { id: 'a7', platform: 'azure', user: 'mruiz-corp', name: 'Cliente Banca', host: 'dev.azure.com/banca', method: 'Token (PAT)', def: true, scopes: 'Code (Read & Write)' }
  ];
  const WS = [{ id: 'w1', name: 'Producto', color: 'oklch(0.72 0.15 190)' }, { id: 'w2', name: 'Backend', color: 'oklch(0.68 0.2 300)' }, { id: 'w3', name: 'Infra', color: 'oklch(0.76 0.16 60)' }];
  const REPOS = [
    { id: 'r1', name: 'gitflow-desktop', alias: 'Cliente Git', path: '~/code/gitflow-desktop', acc: 'a1', wss: ['w1', 'w2'], branch: 'feature/graph-ui', ahead: 2, behind: 0, changes: 6, mins: 5 },
    { id: 'r2', name: 'api-gateway', alias: '', path: '~/work/api-gateway', acc: 'a3', wss: ['w2', 'w3'], branch: 'develop', ahead: 0, behind: 3, changes: 0, mins: 90 },
    { id: 'r3', name: 'billing-service', alias: 'Facturación', path: '~/freelance/billing-service', acc: 'a5', wss: ['w2'], branch: 'main', ahead: 0, behind: 0, changes: 0, mins: 1500 },
    { id: 'r4', name: 'design-tokens', alias: '', path: '~/work/design-tokens', acc: 'a2', wss: ['w1'], branch: 'main', ahead: 1, behind: 0, changes: 2, mins: 300 },
    { id: 'r5', name: 'notas-personales', alias: 'Notas', path: '~/Documents/notas', acc: null, wss: [], branch: 'main', ahead: 0, behind: 0, changes: 1, mins: 2880 },
    { id: 'r6', name: 'infra-terraform', alias: 'Infra prod', path: '~/work/infra-terraform', acc: 'a3', wss: ['w3'], branch: 'main', ahead: 0, behind: 1, changes: 0, mins: 4320 },
    { id: 'r7', name: 'mobile-app', alias: '', path: '~/code/mobile-app', acc: 'a1', wss: ['w1'], branch: 'release/1.8', ahead: 0, behind: 0, changes: 3, mins: 720 },
    { id: 'r8', name: 'dotfiles', alias: '', path: '~/.dotfiles', acc: null, wss: [], branch: 'master', ahead: 0, behind: 0, changes: 0, mins: 10080 },
    { id: 'r9', name: 'docs-site', alias: '', path: '~/homelab/docs-site', acc: 'a6', wss: [], branch: 'main', ahead: 0, behind: 0, changes: 0, mins: 20000 },
    { id: 'r10', name: 'auth-service', alias: 'Auth (banca)', path: '~/clientes/banca/auth-service', acc: 'a7', wss: ['w2', 'w3'], branch: 'feature/mfa', ahead: 3, behind: 0, changes: 4, mins: 200 },
    { id: 'r11', name: 'ray-tracer', alias: '', path: '~/code/ray-tracer', acc: 'a4', wss: [], branch: 'main', ahead: 0, behind: 0, changes: 0, mins: 30000 }
  ];
  const PRS = [
    { repo: 'r1', num: 142, title: 'Grafo de commits con zoom', author: 'marta-dev', from: 'feature/graph-ui', to: 'develop', state: 'draft', checks: 'pending', reviews: '0/2', mins: 30 },
    { repo: 'r1', num: 141, title: 'Multi-cuenta por plataforma', author: 'aiko-t', from: 'feature/accounts', to: 'develop', state: 'open', checks: 'pass', reviews: '1/2', mins: 180 },
    { repo: 'r1', num: 138, title: 'Validar tokens OAuth', author: 'aiko-t', from: 'feature/auth', to: 'develop', state: 'merged', checks: 'pass', reviews: '2/2', mins: 1440 },
    { repo: 'r4', num: 57, title: 'Nuevos tokens de color para diff', author: 'diego-l', from: 'tokens/diff', to: 'main', state: 'approved', checks: 'pass', reviews: '2/2', mins: 400 },
    { repo: 'r7', num: 311, title: 'Pantalla de login biométrico', author: 'samu-o', from: 'feature/biometric', to: 'develop', state: 'changes', checks: 'fail', reviews: '1/2', mins: 600 },
    { repo: 'r7', num: 309, title: 'Actualizar SDK de notificaciones', author: 'marta-dev', from: 'chore/push-sdk', to: 'develop', state: 'open', checks: 'pending', reviews: '0/1', mins: 900 },
    { repo: 'r2', num: 88, title: 'Rate limiting por cliente', author: 'mruiz', from: 'feature/rate-limit', to: 'develop', state: 'open', checks: 'pass', reviews: '1/2', mins: 120 },
    { repo: 'r2', num: 85, title: 'Migrar a Node 22', author: 'diego.leon', from: 'chore/node22', to: 'develop', state: 'closed', checks: 'fail', reviews: '0/2', mins: 5000 },
    { repo: 'r3', num: 23, title: 'Facturas en PDF', author: 'marta_r', from: 'feature/pdf', to: 'main', state: 'approved', checks: 'pass', reviews: '1/1', mins: 2000 },
    { repo: 'r10', num: 7, title: 'MFA con TOTP', author: 'mruiz-corp', from: 'feature/mfa', to: 'main', state: 'open', checks: 'pending', reviews: '0/2', mins: 60 },
    { repo: 'r6', num: 40, title: 'Módulo VPC eu-west', author: 'mruiz', from: 'feature/vpc', to: 'main', state: 'merged', checks: 'pass', reviews: '2/2', mins: 8000 }
  ];
  const UNSTAGED = [{ path: 'src/graph/Graph.tsx', st: 'M' }, { path: 'src/graph/lanes.ts', st: 'M' }, { path: 'src/graph/Tooltip.tsx', st: 'A' }, { path: 'src/i18n/en.json', st: 'M' }, { path: 'src/old/LegacyGraph.tsx', st: 'D' }];
  const STAGED = [{ path: 'src/graph/zoom.ts', st: 'A' }];
  const FILESETS = [
    [['src/graph/Graph.tsx', 'M', 42, 8], ['src/graph/zoom.ts', 'A', 88, 0], ['src/graph/lanes.ts', 'M', 12, 5]],
    [['package.json', 'M', 6, 6], ['pnpm-lock.yaml', 'M', 210, 188]],
    [['src/auth/oauth.ts', 'M', 31, 4], ['src/auth/accounts.ts', 'M', 18, 2], ['src/auth/__tests__/oauth.test.ts', 'A', 64, 0], ['src/auth/legacy.ts', 'D', 0, 40]],
    [['src/i18n/es.json', 'M', 22, 3], ['src/i18n/en.json', 'M', 22, 3], ['src/i18n/index.ts', 'A', 35, 0]]
  ];
  const DIFF = [
    [1, 'import { Commit } from "./types";', 1, 'import { Commit } from "./types";', 'c'],
    [2, '', 2, '', 'c'],
    [3, 'const LANE_WIDTH = 16;', 3, 'const LANE_WIDTH = 20;', 'm'],
    [4, 'const ROW_HEIGHT = 32;', 4, 'const ROW_HEIGHT = 40;', 'm'],
    [null, '', 5, 'const CURVE = 0.5;', 'a'],
    [5, '', 6, '', 'c'],
    [6, 'export function assignLanes(commits: Commit[]) {', 7, 'export function assignLanes(commits: Commit[]) {', 'c'],
    [7, '  const lanes: string[] = [];', 8, '  const lanes: (string | null)[] = [];', 'm'],
    [8, '  for (const c of commits) {', 9, '  for (const c of commits) {', 'c'],
    [9, '    let i = lanes.indexOf(c.sha);', 10, '    let i = lanes.indexOf(c.sha);', 'c'],
    [10, '    if (i === -1) i = lanes.push(c.sha) - 1;', null, '', 'd'],
    [null, '', 11, '    if (i === -1) {', 'a'],
    [null, '', 12, '      i = lanes.indexOf(null);', 'a'],
    [null, '', 13, '      if (i === -1) i = lanes.length;', 'a'],
    [null, '', 14, '    }', 'a'],
    [11, '    c.lane = i;', 15, '    c.lane = i;', 'c'],
    [12, '    lanes[i] = c.parents[0];', 16, '    lanes[i] = c.parents[0] ?? null;', 'm'],
    [13, '    for (const p of c.parents.slice(1)) lanes.push(p);', 17, '    for (const p of c.parents.slice(1)) lanes.push(p);', 'c'],
    [14, '  }', 18, '  }', 'c'],
    [15, '  return lanes.length;', 19, '  return lanes.length;', 'c'],
    [16, '}', 20, '}', 'c']
  ];
  const CF = [
    { path: 'src/auth/session.ts', segs: [
      { c: ['import { api } from "../net/api";', 'import { store } from "./store";', 'import { vault } from "./vault";', 'import type { Account } from "./types";', '', '/** Renueva la sesión de una cuenta concreta. */', 'export async function refreshSession(acc: Account) {'] },
      { h: [['  const token = await store.get(acc.id);', '  if (!token) return null;'], ['  const token = await vault.read(acc.id, { decrypt: true });', '  if (!token || token.expired) {', '    return reauth(acc);', '  }']] },
      { c: ['', '  const res = await api(acc).post("/oauth/refresh", {'] },
      { h: [['    refresh_token: token.refresh,', '    timeout: 5000,'], ['    refresh_token: token.refresh,', '    scope: acc.scopes.join(" "),']] },
      { c: ['    client_id: acc.clientId,', '  });', '  await store.set(acc.id, res.data);', '  return res.data;', '}'] }
    ] },
    { path: 'src/i18n/es.json', segs: [
      { c: ['{', '  "graph.zoom": "Zoom",', '  "graph.reset": "Restablecer vista",'] },
      { h: [['  "auth.expired": "Sesión caducada",'], ['  "auth.expired": "Tu sesión ha expirado. Vuelve a iniciar sesión.",', '  "auth.reauth": "Iniciar sesión de nuevo",']] },
      { c: ['  "auth.retry": "Reintentar"', '}'] }
    ] }
  ];
  const AIPROV = {
    anthropic: { label: 'Anthropic', abbr: 'AN', color: 'oklch(0.66 0.13 45)', model: 'claude-sonnet-4-5', ph: 'sk-ant-…' },
    openai: { label: 'OpenAI', abbr: 'OA', color: 'oklch(0.55 0.02 180)', model: 'gpt-5-mini', ph: 'sk-proj-…' },
    groq: { label: 'Groq', abbr: 'GQ', color: 'oklch(0.62 0.18 30)', model: 'llama-3.3-70b-versatile', ph: 'gsk_…' },
    google: { label: 'Google Gemini', abbr: 'GG', color: 'oklch(0.6 0.15 255)', model: 'gemini-2.5-flash', ph: 'AIza…' },
    mistral: { label: 'Mistral', abbr: 'MI', color: 'oklch(0.7 0.16 60)', model: 'mistral-large-latest', ph: '…' },
    openrouter: { label: 'OpenRouter', abbr: 'OR', color: 'oklch(0.55 0.12 280)', model: 'openrouter/auto', ph: 'sk-or-…', url: 'https://openrouter.ai/api/v1' },
    ollama: { label: 'Ollama (local)', abbr: 'OL', color: 'oklch(0.5 0.01 230)', model: 'qwen2.5-coder:14b', ph: '(sin llave)', url: 'http://localhost:11434' },
    custom: { label: 'Compatible con OpenAI', abbr: 'API', color: 'oklch(0.58 0.08 200)', model: '', ph: '…', url: 'https://' }
  };
  const AIKEYS = [
    { id: 'k1', prov: 'anthropic', label: 'Personal', scope: 'personal', masked: 'sk-ant-••••••••4f2a', model: 'claude-sonnet-4-5', def: true },
    { id: 'k2', prov: 'openai', label: 'ACME (equipo)', scope: 'org', masked: 'sk-proj-••••••••9b1c', model: 'gpt-5-mini', def: false },
    { id: 'k3', prov: 'groq', label: 'Pruebas rápidas', scope: 'personal', masked: 'gsk_••••••••77de', model: 'llama-3.3-70b-versatile', def: false }
  ];
  const MEMBERS = [
    { name: 'Marta Ruiz', email: 'marta@ramal.dev', role: 'owner', ini: 'MR', lane: 1 },
    { name: 'Diego León', email: 'diego@acme.studio', role: 'admin', ini: 'DL', lane: 0 },
    { name: 'Aiko Tanaka', email: 'aiko@acme.studio', role: 'member', ini: 'AT', lane: 3 },
    { name: 'Samuel Ortiz', email: 'samuel@acme.studio', role: 'member', ini: 'SO', lane: 2 },
    { name: 'lucia@acme.studio', email: 'lucia@acme.studio', role: 'member', ini: 'L', lane: 4, pending: true }
  ];
  const SESSIONS = [{ dev: 'MacBook Pro · macOS 15', city: 'Madrid', mins: 0, current: true }, { dev: 'Ubuntu 24.04 · Ramal 2.3', city: 'Barcelona', mins: 4320 }];
  const SSH = [{ file: 'id_ed25519.pub', fp: 'SHA256:q3Vf…Kx8A', used: 'GitHub @marta-dev · Gitea @marta' }, { file: 'id_ed25519_acme.pub', fp: 'SHA256:9nTp…c1Lw', used: 'GitHub @marta-acme · GitLab @mruiz' }];
  const SUBS = [{ path: 'vendor/libgit2-bindings', st: 'M' }, { path: 'docs/theme', st: '✓' }];
  const STASHES = [{ msg: 'WIP: estilos del tooltip' }, { msg: 'experimento de zoom con rueda' }];
  const REPO_PRS = [{ num: 142, title: 'Grafo de commits con zoom', from: 'feature/graph-ui', to: 'develop', state: 'draft' }, { num: 138, title: 'Validar tokens OAuth', from: 'feature/auth', to: 'develop', state: 'merged' }];

  const D = {
    home: ['Inicio', 'Home'], repos: ['Repositorios', 'Repositories'], workspaces: ['Workspaces', 'Workspaces'], accounts: ['Cuentas', 'Accounts'],
    newWs: ['Nuevo workspace', 'New workspace'], reposSub: ['Repositorios abiertos recientemente y vinculados a tus cuentas.', 'Recently opened repositories linked to your accounts.'],
    openRepo: ['Abrir', 'Open'], init: ['Iniciar', 'Init'], clone: ['Clonar', 'Clone'],
    search: ['Buscar por nombre, alias, ruta o cuenta…', 'Search by name, alias, path or account…'],
    kAll: ['Todos', 'All'], kLocal: ['Local', 'Local'], kRemote: ['Remoto', 'Remote'],
    anyAcc: ['Cualquier cuenta', 'Any account'], anyWs: ['Cualquier workspace', 'Any workspace'], noWs: ['Sin workspace', 'No workspace'],
    groupWs: ['Agrupar por workspace', 'Group by workspace'], clear: ['Limpiar filtros', 'Clear filters'], results: ['{n} repositorios', '{n} repositories'],
    noResults: ['Ningún repositorio coincide con los filtros.', 'No repositories match the filters.'],
    localOnly: ['Solo local', 'Local only'], noRemote: ['Sin remoto configurado', 'No remote configured'], changes: ['{n} cambios', '{n} changes'],
    wsRepos: ['Repositorios', 'Repositories'], wsSub: ['{n} repositorios · {p} PRs abiertos', '{n} repositories · {p} open PRs'],
    prsTitle: ['Pull requests', 'Pull requests'], fetchAll: ['Fetch de todos', 'Fetch all'], openAll: ['Abrir todos', 'Open all'],
    colRepo: ['Repositorio', 'Repository'], colPr: ['Pull request', 'Pull request'], colAuthor: ['Autor', 'Author'], colState: ['Estado', 'State'],
    colChecks: ['Checks', 'Checks'], colReviews: ['Revisiones', 'Reviews'], colUpdated: ['Actualizado', 'Updated'],
    noPrs: ['No hay pull requests con este estado.', 'No pull requests with this state.'], prAll: ['Todos', 'All'], openPrs: ['{n} PR abiertos', '{n} open PRs'],
    st_open: ['Abierto', 'Open'], st_draft: ['Borrador', 'Draft'], st_approved: ['Aprobado', 'Approved'], st_changes: ['Cambios solicitados', 'Changes requested'], st_merged: ['Fusionado', 'Merged'], st_closed: ['Cerrado', 'Closed'],
    ck_pass: ['✓ Correctos', '✓ Passing'], ck_fail: ['× Fallidos', '× Failing'], ck_pending: ['● En curso', '● Running'],
    accTitle: ['Cuentas', 'Accounts'], accSub: ['Conecta varias cuentas por plataforma. Cada repositorio remoto usa una de ellas para fetch, push y pull requests.', 'Connect several accounts per platform. Each remote repository uses one of them for fetch, push and pull requests.'],
    addAccount: ['Añadir cuenta', 'Add account'], isDefault: ['Predeterminada', 'Default'], makeDefault: ['Hacer predeterminada', 'Make default'],
    method: ['Autenticación', 'Authentication'], scopes: ['Permisos', 'Scopes'], usedIn: ['Usada en {n} repos', 'Used in {n} repos'], addTo: ['Añadir cuenta de {p}', 'Add {p} account'], noAccounts: ['Sin cuentas conectadas', 'No connected accounts'],
    undo: ['Deshacer', 'Undo'], redo: ['Rehacer', 'Redo'], fetch: ['Fetch', 'Fetch'], pull: ['Pull', 'Pull'], push: ['Push', 'Push'], branch: ['Rama', 'Branch'], stash: ['Stash', 'Stash'], pop: ['Pop', 'Pop'], pr: ['PR', 'PR'],
    searchCommits: ['Buscar commits…', 'Search commits…'], filterSb: ['Filtrar ramas, tags…', 'Filter branches, tags…'],
    sLocal: ['Local', 'Local'], sRemote: ['Remotos', 'Remotes'], sPrs: ['Pull requests', 'Pull requests'], sStash: ['Stashes', 'Stashes'], sTags: ['Tags', 'Tags'], sSubs: ['Submódulos', 'Submodules'],
    colRefs: ['Rama / tag', 'Branch / tag'], colGraph: ['Grafo', 'Graph'], colMsg: ['Mensaje', 'Message'], colDate: ['Fecha', 'Date'], colSha: ['SHA', 'SHA'],
    wipText: ['// WIP · {n} archivos con cambios', '// WIP · {n} changed files'], wipMerge: ['// Merge en curso · resuelve los conflictos', '// Merge in progress · resolve conflicts'],
    backGraph: ['Grafo', 'Graph'], split: ['Lado a lado', 'Side by side'], unified: ['Unificada', 'Unified'], stageFile: ['Preparar archivo', 'Stage file'], unstageFile: ['Quitar de preparados', 'Unstage file'],
    output: ['Resultado', 'Output'], takeAllA: ['Todo de A', 'All from A'], takeAllB: ['Todo de B', 'All from B'], conflictTitle: ['Conflicto en {f}', 'Conflict in {f}'],
    hunk: ['Conflicto {n}', 'Conflict {n}'], use: ['Usar', 'Use'], using: ['Usando', 'Using'], resolved: ['Resuelto', 'Resolved'], conflicted: ['En conflicto', 'Conflicted'],
    mergeInProgress: ['Merge en curso', 'Merge in progress'], cfProgress: ['{n} de {m} archivos resueltos', '{n} of {m} files resolved'], completeMerge: ['Completar merge', 'Complete merge'], abortMerge: ['Abortar merge', 'Abort merge'],
    rbTitle: ['Rebase interactivo', 'Interactive rebase'], rbOrder: ['Los commits se aplican de arriba a abajo. Arrastra para reordenar.', 'Commits are applied top to bottom. Drag to reorder.'],
    cancel: ['Cancelar', 'Cancel'], startRebase: ['Iniciar rebase', 'Start rebase'], rbSummary: ['{p} pick · {s} squash/fixup · {d} drop', '{p} pick · {s} squash/fixup · {d} drop'],
    act_pick: ['pick', 'pick'], act_reword: ['reword', 'reword'], act_edit: ['edit', 'edit'], act_squash: ['squash', 'squash'], act_fixup: ['fixup', 'fixup'], act_drop: ['drop', 'drop'],
    unstaged: ['Sin preparar', 'Unstaged'], staged: ['Preparados', 'Staged'], stageAll: ['Preparar todo', 'Stage all'], unstageAll: ['Quitar todo', 'Unstage all'],
    summary: ['Resumen del commit', 'Commit summary'], description: ['Descripción (opcional)', 'Description (optional)'], amend: ['Modificar el último commit (amend)', 'Amend previous commit'],
    commitBtn: ['Commit de {n} archivos en {b}', 'Commit {n} files to {b}'], amendBtn: ['Modificar último commit', 'Amend last commit'], stgTitle: ['{n} cambios', '{n} changes'], stgSub: ['en {b}', 'on {b}'],
    noChanges: ['Sin cambios', 'No changes'], copy: ['Copiar', 'Copy'], parents: ['Padres', 'Parents'], authorL: ['Autor', 'Author'],
    cherryBtn: ['Cherry-pick', 'Cherry-pick'], revertBtn: ['Revert', 'Revert'], branchBtn: ['Crear rama', 'Create branch'], filesChanged: ['{n} archivos', '{n} files'],
    dark: ['Oscuro', 'Dark'], light: ['Claro', 'Light'],
    mOpen: ['Abrir', 'Open'], mAlias: ['Asignar alias…', 'Set alias…'], mRename: ['Renombrar…', 'Rename…'], mMoveWs: ['Workspaces', 'Workspaces'], mNoWsAll: ['Quitar de todos', 'Remove from all'], mFetch: ['Fetch', 'Fetch'],
    mReveal: ['Mostrar en la carpeta', 'Reveal in folder'], mCopyPath: ['Copiar ruta', 'Copy path'], mRemove: ['Quitar de la lista…', 'Remove from list…'], mTerminal: ['Abrir terminal aquí', 'Open terminal here'],
    mCheckout: ['Checkout', 'Checkout'], mCheckoutTrack: ['Checkout (crear rama local {b})', 'Checkout (create local {b})'], mBranchHere: ['Crear rama aquí…', 'Create branch here…'], mTagHere: ['Crear tag aquí…', 'Create tag here…'],
    mCherry: ['Cherry-pick en {b}…', 'Cherry-pick onto {b}…'], mRevert: ['Revertir commit', 'Revert commit'], mReset: ['Reset de {b} a este commit', 'Reset {b} to this commit'],
    rSoft: ['Soft · conserva todo preparado', 'Soft · keep all staged'], rMixed: ['Mixed · conserva cambios', 'Mixed · keep changes'], rHard: ['Hard · descarta todo…', 'Hard · discard all…'],
    mRebaseI: ['Rebase interactivo desde aquí…', 'Interactive rebase from here…'], mCopySha: ['Copiar SHA', 'Copy SHA'], mCopyMsg: ['Copiar mensaje', 'Copy message'],
    mMerge: ['Merge de {a} en {b}…', 'Merge {a} into {b}…'], mRebaseOnto: ['Rebase de {a} sobre {b}…', 'Rebase {a} onto {b}…'], mRebaseIOnto: ['Rebase interactivo de {a} sobre {b}…', 'Interactive rebase {a} onto {b}…'],
    mPush: ['Push', 'Push'], mPull: ['Pull', 'Pull'], mUpstream: ['Configurar upstream…', 'Set upstream…'], mCreatePR: ['Crear pull request…', 'Create pull request…'], mCompare: ['Comparar con {b}', 'Compare with {b}'],
    mDelete: ['Eliminar rama…', 'Delete branch…'], mDeleteRemote: ['Eliminar rama remota…', 'Delete remote branch…'], mFetchRemote: ['Fetch {r}', 'Fetch {r}'], mEditRemote: ['Editar URL del remoto…', 'Edit remote URL…'],
    mStage: ['Preparar', 'Stage'], mUnstage: ['Quitar de preparados', 'Unstage'], mViewDiff: ['Ver diff', 'View diff'], mDiscard: ['Descartar cambios…', 'Discard changes…'], mHistory: ['Historial del archivo', 'File history'],
    mBlame: ['Blame', 'Blame'], mIgnore: ['Añadir a .gitignore', 'Add to .gitignore'], mEditor: ['Abrir en el editor', 'Open in editor'],
    mApply: ['Aplicar', 'Apply'], mPop: ['Pop (aplicar y eliminar)', 'Pop (apply and drop)'], mDrop: ['Eliminar stash…', 'Drop stash…'], mPushTag: ['Push del tag', 'Push tag'], mDeleteTag: ['Eliminar tag', 'Delete tag'],
    mSubUpdate: ['Actualizar submódulo', 'Update submodule'], mSubOpen: ['Abrir como repositorio', 'Open as repository'], mPrOpen: ['Abrir en {p}', 'Open on {p}'], mPrCheckout: ['Checkout de la rama', 'Checkout branch'], mCopyLink: ['Copiar enlace', 'Copy link'],
    mTest: ['Probar conexión', 'Test connection'], mEditToken: ['Actualizar credenciales…', 'Update credentials…'], mRemoveAcc: ['Desconectar cuenta…', 'Disconnect account…'],
    mRenameWs: ['Renombrar workspace…', 'Rename workspace…'], mDeleteWs: ['Eliminar workspace…', 'Delete workspace…'],
    mStashAll: ['Stash de todos los cambios…', 'Stash all changes…'], mDiscardAll: ['Descartar todos los cambios…', 'Discard all changes…'],
    mFetchAllR: ['Fetch de todos los remotos', 'Fetch all remotes'], mPullFF: ['Pull (fast-forward si es posible)', 'Pull (fast-forward if possible)'], mPullOnly: ['Pull (solo fast-forward)', 'Pull (fast-forward only)'], mPullRebase: ['Pull (rebase)', 'Pull (rebase)'],
    mForcePush: ['Force push (with lease)…', 'Force push (with lease)…'],
    save: ['Guardar', 'Save'], remove: ['Quitar', 'Remove'], create: ['Crear', 'Create'], del: ['Eliminar', 'Delete'], alias: ['Alias', 'Alias'],
    aliasHint: ['El alias solo cambia cómo se muestra en Ramal; la carpeta y el remoto no cambian.', 'The alias only changes how Ramal displays it; folder and remote stay the same.'],
    name: ['Nombre', 'Name'], renameInfo: ['Renombra la carpeta del repositorio en disco.', 'Renames the repository folder on disk.'],
    removeTitle: ['¿Quitar {n} de la lista?', 'Remove {n} from the list?'], removeInfo: ['El repositorio seguirá en tu disco y en el remoto.', 'The repository stays on your disk and remote.'], alsoDisk: ['Además, mover la carpeta a la papelera', 'Also move the folder to trash'],
    newWsTitle: ['Nuevo workspace', 'New workspace'], renameWsTitle: ['Renombrar workspace', 'Rename workspace'], deleteWsTitle: ['¿Eliminar el workspace {n}?', 'Delete workspace {n}?'], deleteWsInfo: ['Los repositorios no se eliminan; quedan sin workspace.', 'Repositories are kept, without a workspace.'],
    mergeTitle: ['Merge', 'Merge'], strategy: ['Estrategia', 'Strategy'], sFF: ['Fast-forward si es posible', 'Fast-forward if possible'], sNoFF: ['Crear commit de merge', 'Create merge commit'], sSquash: ['Squash', 'Squash'],
    editMsg: ['Editar el mensaje antes de confirmar', 'Edit message before committing'], rebaseTitle: ['Rebase', 'Rebase'], autostash: ['Autostash de cambios locales', 'Autostash local changes'], interactive: ['Rebase interactivo', 'Interactive rebase'],
    cherryTitle: ['Cherry-pick', 'Cherry-pick'], cherryX: ['Añadir "(cherry picked from…)" al mensaje', 'Append "(cherry picked from…)" to message'], noCommit: ['Aplicar sin crear commit', 'Apply without committing'],
    prTitleM: ['Crear pull request', 'Create pull request'], asAccount: ['Cuenta', 'Account'], fromB: ['Desde', 'From'], toB: ['Hacia', 'Into'], prTitle: ['Título', 'Title'], prDesc: ['Descripción', 'Description'],
    reviewers: ['Revisores', 'Reviewers'], reviewersPh: ['@usuario, @equipo', '@user, @team'], draft: ['Crear como borrador', 'Create as draft'],
    branchTitle: ['Crear rama', 'Create branch'], branchName: ['Nombre de la rama', 'Branch name'], checkoutAfter: ['Hacer checkout después de crearla', 'Checkout after creating'],
    tagTitle: ['Crear tag', 'Create tag'], tagName: ['Nombre del tag', 'Tag name'], tagMsg: ['Mensaje (tag anotado)', 'Message (annotated tag)'], pushTagToo: ['Hacer push del tag a origin', 'Push tag to origin'],
    stashTitle: ['Stash', 'Stash'], stashMsg: ['Mensaje', 'Message'], untracked: ['Incluir archivos sin seguimiento', 'Include untracked files'],
    accTitleM: ['Añadir cuenta', 'Add account'], platform: ['Plataforma', 'Platform'], host: ['Servidor', 'Host'], user: ['Usuario', 'Username'], token: ['Token de acceso', 'Access token'], label: ['Etiqueta', 'Label'], labelPh: ['Trabajo, Personal…', 'Work, Personal…'],
    resetHardTitle: ['¿Reset --hard a {s}?', 'Reset --hard to {s}?'], resetHardInfo: ['Se perderán los cambios sin commit y los commits posteriores en la rama actual.', 'Uncommitted changes and later commits on the current branch will be lost.'],
    discardTitle: ['¿Descartar cambios?', 'Discard changes?'], discardInfo: ['Esta acción no se puede deshacer.', 'This cannot be undone.'],
    deleteBranchTitle: ['¿Eliminar {b}?', 'Delete {b}?'], deleteBranchInfo: ['La rama tiene commits que no están en otra rama.', 'The branch may contain commits not on other branches.'], forceDelete: ['Forzar (-D)', 'Force (-D)'],
    cloneTitle: ['Clonar repositorio', 'Clone repository'], cloneUrl: ['URL o owner/repo', 'URL or owner/repo'], cloneDest: ['Carpeta de destino', 'Destination folder'],
    openTitle: ['Abrir repositorio local', 'Open local repository'], openPath: ['Ruta', 'Path'], initTitle: ['Iniciar repositorio', 'Initialize repository'], publish: ['Publicar en una cuenta remota', 'Publish to a remote account'],
    renameBranchTitle: ['Renombrar rama', 'Rename branch'], forcePushTitle: ['¿Force push a origin/{b}?', 'Force push to origin/{b}?'], forcePushInfo: ['Se usa --force-with-lease: falla si alguien ha hecho push desde tu último fetch.', 'Uses --force-with-lease: fails if someone pushed since your last fetch.'],
    saved: ['Cambios guardados', 'Changes saved'], removedT: ['{n} quitado de la lista', '{n} removed from list'], copied: ['Copiado: {x}', 'Copied: {x}'],
    fetchedT: ['Fetch completado', 'Fetch complete'], pushedT: ['Push a {b} completado', 'Pushed to {b}'], pulledT: ['{b} está al día', '{b} is up to date'], committedT: ['Commit {s} creado', 'Commit {s} created'],
    cherryT: ['Cherry-pick de {s} aplicado', 'Cherry-picked {s}'], revertT: ['Revert de {s} creado', 'Reverted {s}'], resetT: ['Reset {m} a {s}', 'Reset {m} to {s}'],
    mergedT: ['{a} fusionada en {b}', '{a} merged into {b}'], conflictsT: ['Merge con conflictos en 2 archivos', 'Merge has conflicts in 2 files'], rebaseT: ['{a} rebasada sobre {b}', '{a} rebased onto {b}'],
    abortedT: ['Merge abortado', 'Merge aborted'], stashedT: ['Cambios guardados en stash', 'Changes stashed'], poppedT: ['Stash aplicado', 'Stash applied'], droppedT: ['Stash eliminado', 'Stash dropped'],
    branchT: ['Rama {b} creada', 'Branch {b} created'], tagT: ['Tag {b} creado', 'Tag {b} created'], prT: ['PR #{n} creado en {p}', 'PR #{n} created on {p}'], checkoutT: ['Checkout de {b}', 'Checked out {b}'],
    accAddedT: ['Cuenta @{u} conectada', 'Account @{u} connected'], accRemovedT: ['Cuenta desconectada', 'Account disconnected'], defT: ['@{u} es ahora la cuenta predeterminada', '@{u} is now the default account'],
    wsCreatedT: ['Workspace {n} creado', 'Workspace {n} created'], movedT: ['Workspaces actualizados', 'Workspaces updated'], undoneT: ['Deshecho: {x}', 'Undone: {x}'], redoneT: ['Nada que rehacer', 'Nothing to redo'],
    noStagedT: ['Prepara al menos un archivo y escribe un resumen', 'Stage at least one file and write a summary'], nothingStashT: ['No hay stashes', 'No stashes'], clonedT: ['{n} clonado', '{n} cloned'],
    discardedT: ['Cambios descartados', 'Changes discarded'], deletedT: ['{b} eliminada', '{b} deleted'], doneT: ['{x}: listo', '{x}: done'], stillConflicts: ['Quedan conflictos sin resolver', 'Unresolved conflicts remain'],
    mergeDoneT: ['Merge completado', 'Merge completed'],
    fKind: ['Tipo', 'Type'], fPlat: ['Plataforma', 'Platform'], fAcc: ['Cuenta', 'Account'], fWs: ['Workspace', 'Workspace'], fGroup: ['Agrupar', 'Group'],
    gNone: ['Nada', 'None'], allPlat: ['Todas', 'All'], anyShort: ['Cualquiera', 'Any'], partial: ['Parcial', 'Partial'],
    yours: ['Tu rama (actual)', 'Yours (current)'], theirs: ['Entrante', 'Incoming'], takeBlock: ['Tomar bloque', 'Take block'], dropBlock: ['Quitar bloque', 'Drop block'],
    lgAdd: ['Añadir la línea al resultado', 'Add line to output'], lgRemove: ['Quitar la línea del resultado', 'Remove line from output'],
    gapBlock: ['Conflicto {n} sin resolver · añade líneas desde A o B con +', 'Conflict {n} unresolved · add lines from A or B with +'],
    outputSub: ['así quedará {f} al completar el merge', 'how {f} will look after the merge'], editManual: ['Editar a mano', 'Edit manually'], doneEdit: ['Listo', 'Done'], resetManual: ['Restablecer', 'Reset'], manualNote: ['editado a mano', 'edited manually'],
    settings: ['Configuración', 'Settings'], sgUser: ['Usuario', 'User'], sgConn: ['Conexiones', 'Connections'], sgApp: ['Aplicación', 'Application'],
    secProfile: ['Perfil', 'Profile'], secAccount: ['Cuenta y equipo', 'Account & team'], secIntegrations: ['Plataformas Git', 'Git platforms'], secAI: ['Inteligencia artificial', 'Artificial intelligence'], secGit: ['Git', 'Git'], secAppearance: ['Apariencia', 'Appearance'],
    profileSub: ['Tu identidad en Ramal. Es lo que ve tu equipo en invitaciones y actividad.', 'Your Ramal identity. It is what your team sees in invitations and activity.'],
    changePhoto: ['Cambiar foto', 'Change photo'], fullName: ['Nombre', 'Name'], username: ['Usuario', 'Username'], email: ['Email', 'Email'], timezone: ['Zona horaria', 'Time zone'], saveChanges: ['Guardar cambios', 'Save changes'],
    accountSub: ['Tu cuenta de Ramal, la suscripción y los miembros de tu organización.', 'Your Ramal account, subscription and organization members.'],
    plan: ['Plan', 'Plan'], planName: ['Ramal Pro', 'Ramal Pro'], planRenew: ['Se renueva el 12 de octubre de 2026 · 5 asientos', 'Renews on October 12, 2026 · 5 seats'], manageSub: ['Gestionar suscripción', 'Manage subscription'],
    session: ['Sesión', 'Session'], signedAs: ['Sesión iniciada como', 'Signed in as'], signOut: ['Cerrar sesión', 'Sign out'],
    members: ['{n} miembros', '{n} members'], invite: ['Invitar miembro', 'Invite member'], role: ['Rol', 'Role'], r_owner: ['Propietaria', 'Owner'], r_admin: ['Admin', 'Admin'], r_member: ['Miembro', 'Member'], pending: ['Invitación pendiente', 'Invitation pending'],
    sessions: ['Sesiones activas', 'Active sessions'], thisDevice: ['Este dispositivo', 'This device'], revoke: ['Cerrar', 'Revoke'],
    inviteTitle: ['Invitar a {o}', 'Invite to {o}'], invitedT: ['Invitación enviada a {e}', 'Invitation sent to {e}'], signedOutT: ['Sesión cerrada (simulado)', 'Signed out (simulated)'], revokedT: ['Sesión cerrada en {d}', 'Session revoked on {d}'],
    mRemoveMember: ['Quitar de la organización…', 'Remove from organization…'], mResend: ['Reenviar invitación', 'Resend invitation'],
    intSub: ['Conecta varias cuentas por plataforma. Cada repositorio remoto usa una de ellas para fetch, push y pull requests.', 'Connect several accounts per platform. Each remote repository uses one of them for fetch, push and pull requests.'],
    aiSub: ['Añade llaves de proveedores de IA para generar mensajes de commit y títulos y descripciones de PR. Las llaves se guardan cifradas en el llavero del sistema.', 'Add AI provider keys to generate commit messages and PR titles and descriptions. Keys are stored encrypted in the system keychain.'],
    addKey: ['Añadir llave', 'Add key'], provider: ['Proveedor', 'Provider'], model: ['Modelo', 'Model'], apiKey: ['API key', 'API key'], baseUrl: ['URL base', 'Base URL'], scope: ['Pertenece a', 'Belongs to'], sc_personal: ['Personal', 'Personal'], sc_org: ['Organización', 'Organization'], keyLabelPh: ['Personal, Equipo…', 'Personal, Team…'],
    aiKeysTitle: ['Llaves', 'Keys'], aiFeatures: ['Dónde usar la IA', 'Where to use AI'], aiFeaturesSub: ['Activa cada función y elige qué llave usa.', 'Turn each feature on and choose which key it uses.'],
    fCommit: ['Mensajes de commit', 'Commit messages'], fCommitD: ['Genera resumen y descripción a partir de los cambios preparados.', 'Generates summary and description from staged changes.'],
    fPR: ['Pull requests', 'Pull requests'], fPRD: ['Genera título y descripción al crear un PR.', 'Generates title and description when creating a PR.'],
    fSummary: ['Resumen de commits', 'Commit summaries'], fSummaryD: ['Explica qué hace un commit en el panel de detalle.', 'Explains what a commit does in the detail panel.'],
    useDefaultKey: ['Llave predeterminada', 'Default key'], aiPrefs: ['Preferencias de generación', 'Generation preferences'], aiLang: ['Idioma del texto generado', 'Generated text language'], langApp: ['Igual que la app', 'Same as app'],
    conventional: ['Usar Conventional Commits', 'Use Conventional Commits'], conventionalD: ['feat:, fix:, chore:… con ámbito opcional', 'feat:, fix:, chore:… with optional scope'],
    instructions: ['Instrucciones adicionales', 'Extra instructions'], instructionsPh: ['Ej. menciona el ticket de JIRA si aparece en el nombre de la rama', 'e.g. mention the JIRA ticket if it appears in the branch name'],
    excludeSens: ['Excluir archivos sensibles', 'Exclude sensitive files'], excludeSensD: ['Nunca envía .env, *.pem ni *.key al proveedor.', 'Never sends .env, *.pem or *.key to the provider.'],
    noKeys: ['Aún no hay llaves. Añade una para activar las funciones de IA.', 'No keys yet. Add one to enable AI features.'],
    keyAddedT: ['Llave de {p} añadida', '{p} key added'], keyRemovedT: ['Llave eliminada', 'Key removed'], keyDefT: ['{l} es ahora la llave predeterminada', '{l} is now the default key'], keyTestT: ['Conexión con {p} correcta · {m}', 'Connected to {p} · {m}'],
    mMakeDefKey: ['Usar como predeterminada', 'Use as default'], mEditKey: ['Editar…', 'Edit…'], mRemoveKey: ['Eliminar llave…', 'Delete key…'], editKeyTitle: ['Editar llave', 'Edit key'],
    aiGenerate: ['Generar con IA', 'Generate with AI'], aiGenerating: ['Generando…', 'Generating…'], aiPR: ['✦ Generar título y descripción con IA', '✦ Generate title and description with AI'],
    aiNoKey: ['Configura una llave de IA en Configuración › IA', 'Set up an AI key in Settings › AI'], aiDoneT: ['Generado con {p} · {m}', 'Generated with {p} · {m}'], aiDisabled: ['Esta función de IA está desactivada en Configuración', 'This AI feature is disabled in Settings'], aiNothing: ['Prepara algún archivo primero', 'Stage some files first'],
    gitSub: ['Opciones globales que Ramal aplica a todos los repositorios.', 'Global options Ramal applies to every repository.'],
    identity: ['Identidad', 'Identity'], identityD: ['Autor de tus commits (user.name / user.email).', 'Author of your commits (user.name / user.email).'], defBranch: ['Rama inicial por defecto', 'Default initial branch'], behavior: ['Comportamiento', 'Behavior'],
    signCommits: ['Firmar commits', 'Sign commits'], signCommitsD: ['Con tu llave SSH o GPG', 'With your SSH or GPG key'], autoFetch: ['Fetch automático', 'Auto fetch'], off: ['Desactivado', 'Off'], everyN: ['Cada {n} min', 'Every {n} min'],
    pullDefault: ['Pull por defecto', 'Default pull'], prune: ['Podar ramas remotas al hacer fetch', 'Prune remote branches on fetch'], sshKeys: ['Llaves SSH', 'SSH keys'], genKey: ['Generar llave SSH', 'Generate SSH key'], usedBy: ['Usada por {u}', 'Used by {u}'],
    appearanceSub: ['Tema, idioma y visualización del grafo.', 'Theme, language and graph display.'], themeL: ['Tema', 'Theme'], language: ['Idioma', 'Language'], graphStyle: ['Nodos del grafo', 'Graph nodes'], nAvatar: ['Iniciales', 'Initials'], nDot: ['Punto', 'Dot'], display: ['Visualización', 'Display'],
    vChanges: ['Cambios', 'Changes'], vMerge: ['Cambios de merge', 'Merge Changes'], vStaged: ['Cambios preparados', 'Staged Changes'], vBranches: ['Ramas', 'Branches'], vRemotes: ['Remotos', 'Remotes'],
    msgPh: ['Mensaje (⌘Enter para commit en "{b}")', 'Message (⌘Enter to commit on "{b}")'], commitPush: ['Commit y push', 'Commit & Push'], commitAmend: ['Commit (amend)', 'Commit (Amend)'],
    tabGraph: ['Grafo de Ramal', 'Ramal Graph'], tabSettings: ['Configuración de Ramal', 'Ramal Settings'], workingTree: ['(Árbol de trabajo)', '(Working Tree)'], headV: ['(HEAD)', '(HEAD)'],
    followsVs: ['El tema y el idioma siguen la configuración de VS Code.', 'Theme and language follow your VS Code settings.'],
    syncSettings: ['Sincronizar configuración desde la nube', 'Sync settings from the cloud'], syncD: ['Usa las cuentas vinculadas, llaves de IA y preferencias de tu cuenta de Ramal.', 'Use the linked accounts, AI keys and preferences from your Ramal account.'],
    openDashboard: ['Abrir panel web', 'Open web dashboard'], openDesktop: ['Abrir en Ramal Desktop', 'Open in Ramal Desktop'], cmdPh: ['Escribe un comando de Ramal', 'Type a Ramal command'],
    sAccount: ['Cuenta de Ramal', 'Ramal Account'], sPlatforms: ['Plataformas Git', 'Git Platforms'], sAI: ['IA', 'AI'], sGit: ['Git', 'Git'], addAccountV: ['Añadir cuenta…', 'Add Account…'], addKeyV: ['Añadir llave…', 'Add Key…'],
    aiKeyV: ['Llave usada por las funciones de IA.', 'Key used by AI features.'], aiLangV: ['Idioma del texto generado.', 'Language of generated text.'], matchVs: ['Igual que VS Code', 'Match VS Code'],
    noConflicts: ['No quedan conflictos', 'No conflicts left'], conflictsN: ['{n} conflictos', '{n} conflicts'], sourceRamal: ['Origen: Ramal', 'Source: Ramal'], ok: ['Aceptar', 'OK'], selectCommit: ['Selecciona un commit para ver sus detalles.', 'Select a commit to see its details.'],
    cfHint: ['Marca las líneas que quieres conservar de cada lado', 'Tick the lines you want to keep from each side'], lastCommit: ['último commit', 'last commit']
  };
  return { AIPROV, AIKEYS, MEMBERS, SESSIONS, SSH, PLAT, LANE, AUTH, mkCommits, ACCOUNTS, WS, REPOS, PRS, UNSTAGED, STAGED, FILESETS, DIFF, CF, SUBS, STASHES, REPO_PRS, D };
})();
