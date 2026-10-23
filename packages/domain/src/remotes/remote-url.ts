/**
 * Saca el host, el propietario y el nombre de la URL de un remoto.
 *
 * Existe por un fallo concreto (gaps B-30): el nombre que se propone al clonar
 * se sacaba recortando la URL, y con `git@host:repo.git` o con una barra final
 * salia mal. Lo usan el dialogo de clonar, las tarjetas de repositorio y
 * cualquier sitio que tenga que ensenar "de donde viene" un repositorio.
 *
 * **Esto no es la identidad del repositorio.** La `remoteKey` que guarda la nube
 * la calcula siempre el servidor a partir de `remoteUrl`, y vive en un solo sitio
 * (`git-rudder-core-domain`, en Rust): dos implementaciones que normalizaran
 * distinto partirian un repositorio en dos
 * ([ADR 0019](../../../../docs/adr/0019-sincronizacion-incremental.md)). Esta
 * funcion es para ensenar, y sus tests usan los mismos vectores que aquella tabla
 * para que las dos lean las mismas URL de la misma forma.
 */

export interface RemoteRef {
  /** Host en minusculas, con el puerto solo si no es el de por defecto del esquema. */
  readonly host: string;
  /** Lo que va antes del nombre. En Azure DevOps son dos segmentos: `organizacion/proyecto`. */
  readonly owner: string;
  /** Nombre del repositorio, sin `.git` ni barra final. */
  readonly name: string;
}

/**
 * Puertos que no se escriben porque son los de por defecto de su esquema.
 *
 * El analisis se hace a mano y no con `URL`: `@git-rudder/domain` compila sin los
 * tipos de DOM ni los de Node (`@git-rudder/tsconfig/library.json`), asi que `URL`
 * no existe aqui. Tampoco se echa de menos, porque `git@host:ruta` no es una URL
 * valida y habria que tratarla aparte de todas formas.
 */
const DEFAULT_PORTS: Readonly<Record<string, string>> = {
  https: '443',
  http: '80',
  ssh: '22',
  git: '9418',
};

/** `esquema://usuario@host:puerto/ruta`, con el usuario y el puerto opcionales. */
const URL_LIKE =
  /^(?<scheme>[A-Za-z][A-Za-z0-9+.-]*):\/\/(?:[^@/]*@)?(?<host>\[[^\]]+\]|[^/:]+)(?::(?<port>\d+))?(?<path>\/.*)?$/u;

/** `usuario@host:ruta`, la forma corta de SSH que no es una URL valida. */
const SCP_LIKE = /^(?:(?<user>[^@/]+)@)?(?<host>[^:/]+):(?<path>.+)$/u;

/**
 * `propietario/repo`, lo que se escribe a mano en el dialogo de clonar. Los dos
 * segmentos tienen que empezar por letra, numero o subrayado: sin eso, `./repo` y
 * `../repo` pasarian por atajos y darian un propietario `.` en vez de null.
 */
const SHORTHAND = /^[A-Za-z0-9_][\w.-]*\/[A-Za-z0-9_][\w.-]*$/u;

function trimPath(path: string): string {
  return path.replace(/^\/+/u, '').replace(/\/+$/u, '');
}

function stripGitSuffix(segment: string): string {
  return segment.endsWith('.git') ? segment.slice(0, -'.git'.length) : segment;
}

/**
 * Azure DevOps escribe la misma ruta de tres formas: `org/proyecto/_git/repo` por
 * HTTPS, `v3/org/proyecto/repo` por SSH y con el host `ssh.dev.azure.com`. Las
 * tres apuntan al mismo repositorio, asi que se dejan en una.
 */
function normalizeAzure(host: string, segments: readonly string[]): readonly string[] {
  if (!host.endsWith('dev.azure.com') && !host.endsWith('visualstudio.com')) return segments;
  const withoutV3 = segments[0] === 'v3' ? segments.slice(1) : segments;
  return withoutV3.filter((segment) => segment !== '_git');
}

function azureHost(host: string): string {
  return host === 'ssh.dev.azure.com' ? 'dev.azure.com' : host;
}

/** Del host y los segmentos de la ruta al resultado; sin propietario no hay referencia. */
function refFrom(rawHost: string, port: string, path: string): RemoteRef | null {
  const host = azureHost(rawHost.toLowerCase());
  const segments = normalizeAzure(
    host,
    trimPath(path)
      .split('/')
      .filter((segment) => segment !== ''),
  );
  const name = segments.at(-1);
  if (name === undefined || segments.length < 2) return null;
  return {
    host: port === '' ? host : `${host}:${port}`,
    owner: segments.slice(0, -1).join('/'),
    name: stripGitSuffix(name),
  };
}

function fromUrlLike(raw: string): RemoteRef | null {
  const groups = URL_LIKE.exec(raw)?.groups;
  if (groups?.['host'] === undefined) return null;
  const scheme = (groups['scheme'] ?? '').toLowerCase();
  const rawPort = groups['port'] ?? '';
  const port = rawPort === DEFAULT_PORTS[scheme] ? '' : rawPort;
  return refFrom(groups['host'], port, groups['path'] ?? '');
}

function fromScpLike(raw: string): RemoteRef | null {
  const groups = SCP_LIKE.exec(raw)?.groups;
  if (groups?.['host'] === undefined || groups['path'] === undefined) return null;
  // `host:1234/ruta` es un puerto, no el primer segmento: es como lo escribe
  // `ssh://`, y en la forma corta de scp no se puede usar.
  if (/^\d+\//u.test(groups['path'])) return null;
  return refFrom(groups['host'], '', groups['path']);
}

/**
 * Lee cualquiera de las formas que acepta git: `https://`, `http://`, `git://`,
 * `ssh://`, la forma corta `usuario@host:ruta` y el atajo `propietario/repo` que
 * se escribe a mano. Devuelve `null` si no se puede sacar propietario y nombre
 * (una ruta local, por ejemplo: un repositorio sin remoto no tiene identidad
 * global).
 */
export function parseRemoteUrl(remoteUrl: string): RemoteRef | null {
  const raw = remoteUrl.trim();
  if (raw === '') return null;
  if (SHORTHAND.test(raw)) {
    const [owner = '', name = ''] = raw.split('/');
    return { host: '', owner, name: stripGitSuffix(name) };
  }
  if (raw.includes('://')) return fromUrlLike(raw);
  return fromScpLike(raw);
}

/**
 * El nombre que se propone al clonar. Si la URL no se entiende se devuelve `''`
 * y el dialogo deja el campo vacio, que es mejor que proponer algo inventado.
 */
export function remoteRepoName(remoteUrl: string): string {
  return parseRemoteUrl(remoteUrl)?.name ?? '';
}
