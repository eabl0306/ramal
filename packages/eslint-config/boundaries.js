import path from 'node:path';

import boundaries from 'eslint-plugin-boundaries';

/**
 * Arquitectura forzada por el linter. Las capas se declaran como tipos de
 * elemento, los ficheros de test como categoria aparte, y la direccion de
 * dependencias se escribe una sola vez, aqui.
 *
 * Esto vigila las capas DENTRO de un paquete. Que paquete puede depender de cual
 * lo vigila tools/checks/package-graph.mjs leyendo los package.json.
 *
 * `rootPath` es obligatorio y absoluto: el directorio del paquete. Sin el, el
 * plugin resuelve `include` y los patrones de las capas desde el directorio
 * donde se lanza ESLint. El hook de pre-commit lo lanza desde la raiz del repo,
 * y alli `src/domain` no casa con nada: todas las reglas de capas se apagaban en
 * silencio y un import prohibido entraba en el commit.
 *
 * @param {{
 *   rootPath: string,
 *   files?: string[],
 *   elements: Array<Record<string, unknown>>,
 *   fileCategories?: Array<Record<string, unknown>>,
 *   policies: Array<Record<string, unknown>>,
 *   ignore?: string[],
 *   include?: string[],
 *   extensions?: string[],
 * }} options
 * @returns {import('eslint').Linter.Config[]}
 */
export function boundariesConfig({
  rootPath,
  files = ['**/*.{ts,tsx}'],
  elements,
  fileCategories = [],
  policies,
  ignore = [],
  include = ['src/**/*'],
  extensions = ['.ts', '.tsx', '.js', '.jsx', '.mjs', '.css'],
}) {
  if (typeof rootPath !== 'string' || !path.isAbsolute(rootPath)) {
    throw new Error(
      'boundariesConfig necesita rootPath: la ruta absoluta del paquete ' +
        "(fileURLToPath(new URL('.', import.meta.url))).",
    );
  }
  return [
    {
      files,
      plugins: { boundaries },
      settings: {
        'boundaries/root-path': rootPath,
        'boundaries/elements': elements,
        'boundaries/files': fileCategories,
        'boundaries/include': include,
        'boundaries/ignore': ignore,
        // El plugin resuelve los imports con eslint-module-utils, que lee esta
        // clave. Sin las extensiones de TypeScript no encuentra los ficheros.
        'import/resolver': { node: { extensions } },
      },
      rules: {
        // checkAllOrigins hace que la regla mire tambien los paquetes de
        // node_modules, no solo los ficheros del propio proyecto.
        'boundaries/dependencies': [
          'error',
          { default: 'disallow', checkAllOrigins: true, policies },
        ],
        'boundaries/no-unknown-dependencies': 'error',
        'boundaries/no-unknown-files': 'error',
      },
    },
  ];
}
