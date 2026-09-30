/**
 * Stylelint de todo el monorepo: una sola configuracion para el CSS de las apps y
 * de los paquetes, porque el sistema de diseno es uno solo (@ramal/ui).
 *
 * La regla que importa: fuera de packages/ui/src/themes ningun CSS escribe un
 * color. Ramal tiene tema claro y oscuro, y un color a mano no cambia con el tema.
 * Las propiedades de color solo aceptan var(--...) (declaration-strict-value) y,
 * por si el color se cuela en otra propiedad (una sombra, un degradado), se
 * prohiben tambien los hexadecimales, los nombres de color y las funciones de color.
 * Es la misma regla que ramal/no-raw-colors aplica al TypeScript.
 */
const COLOR_FUNCTIONS = ['rgb', 'rgba', 'hsl', 'hsla', 'hwb', 'lab', 'lch', 'oklab', 'oklch'];

/** Valores que no son un color concreto y que un token no puede sustituir. */
const COLOR_KEYWORDS = [
  'currentcolor',
  'transparent',
  'inherit',
  'initial',
  'unset',
  'revert',
  'none',
];

export default {
  extends: ['stylelint-config-standard'],
  plugins: ['stylelint-declaration-strict-value'],
  ignoreFiles: [
    '**/node_modules/**',
    '**/dist/**',
    '**/build/**',
    '**/out/**',
    '**/coverage/**',
    '**/target/**',
    'docs/**',
  ],
  rules: {
    'color-no-hex': true,
    'color-named': 'never',
    'function-disallowed-list': COLOR_FUNCTIONS,
    'scale-unlimited/declaration-strict-value': [
      ['/color$/', 'fill', 'stroke'],
      {
        ignoreValues: COLOR_KEYWORDS,
        ignoreFunctions: false,
        expandShorthand: true,
        disableFix: true,
        message: 'Los colores salen de los tokens de @ramal/ui: usa var(--...).',
      },
    ],
  },
  overrides: [
    {
      // Los temas son el unico sitio donde se escriben los valores de los tokens.
      files: ['packages/ui/src/themes/**/*.css'],
      rules: {
        'color-no-hex': null,
        'color-named': null,
        'function-disallowed-list': null,
        'scale-unlimited/declaration-strict-value': null,
      },
    },
  ],
};
