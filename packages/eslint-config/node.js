import globals from 'globals';

/**
 * Capa de Node: el host de la extension de VS Code, los scripts de tools/ y los
 * ficheros de configuracion. Declara el entorno de Node y nada del navegador, asi
 * que un `window` o un `document` en el host de la extension salta aqui (en
 * JavaScript, por no-undef) o en el typecheck (en TypeScript, porque node.json no
 * trae la lib DOM).
 *
 * @param {{ files?: string[] }} [options]
 * @returns {import('eslint').Linter.Config[]}
 */
export function nodeConfig({ files = ['**/*.{js,mjs,cjs,ts,mts,cts}'] } = {}) {
  return [
    {
      files,
      languageOptions: {
        ecmaVersion: 'latest',
        sourceType: 'module',
        globals: { ...globals.node },
      },
    },
  ];
}
