/**
 * Convencion de commits del monorepo. Los scopes son los ambitos reales del repo,
 * asi el historial dice de un vistazo que parte cambio.
 */
export default {
  extends: ['@commitlint/config-conventional'],
  rules: {
    'scope-enum': [
      2,
      'always',
      [
        'api',
        'engine',
        'desktop',
        'vscode',
        'web',
        'domain',
        'i18n',
        'ui',
        'repo-view',
        'bridge',
        'api-client',
        'packages',
        'tools',
        'ci',
        'docs',
        'deps',
        'repo',
        'release',
      ],
    ],
    'header-max-length': [2, 'always', 100],
    'body-max-line-length': [2, 'always', 100],
  },
};
