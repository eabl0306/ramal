# @git-rudder/i18n

Unica fuente de textos de interfaz de Git Rudder, en castellano e ingles. Cero dependencias.

```ts
const t = createTranslator(resolveLocale(navigator.language));
t('toast.pushed', { branch: 'main' }); // "Push a main completado"
t('repos.results', { count: 1 }); // "1 repositorio"
```

- Los textos viven en `src/locales/<idioma>/<espacio>.json`. El castellano es la fuente de
  verdad: de su forma salen las claves tipadas, y si al ingles le falta o le sobra una clave,
  no compila.
- Los parametros tienen nombre (`{branch}`, `{count}`...). Como TypeScript no ve dentro de los
  textos de un JSON, `scripts/generate-params.mjs` escribe sus tipos en `src/generated/params.ts`.
- Plurales con sufijos `_one` y `_other` (y `_zero` si hace falta), resueltos con
  `Intl.PluralRules` sobre `count`.
- Nada de glifos ni atajos dentro de los textos: los glifos los pinta el componente y los atajos
  llegan como parametro desde `formatShortcut`.
- En React se usa a traves de `I18nProvider` y `useT` de `@git-rudder/ui`.

## Anadir un texto

1. La clave en `src/locales/es/<espacio>.json` y su traduccion en `src/locales/en/`.
2. Si lleva parametros: `corepack pnpm --filter @git-rudder/i18n generate`.
3. `corepack pnpm --filter @git-rudder/i18n test` comprueba claves, parametros, plurales y glifos.
