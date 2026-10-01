/**
 * Nombres de rama y de tag segun las reglas de `git check-ref-format`, para
 * validar en el momento lo que se escribe en un dialogo (gaps B-31: el
 * prototipo solo cambiaba espacios por guiones). El motor vuelve a validar.
 *
 * Tambien es el validador declarativo de las peticiones de texto del puerto
 * RepoViewHost: una funcion no cruza postMessage, un nombre de regla si.
 */

export type RefNameIssue =
  | 'empty'
  | 'forbidden-character'
  | 'double-dot'
  | 'leading-dash'
  | 'slash-position'
  | 'dot-component'
  | 'lock-suffix'
  | 'trailing-dot'
  | 'at-brace'
  | 'only-at';

/** Ademas de los caracteres de control: espacio, ~ ^ : ? * [ y la barra invertida. */
const FORBIDDEN_SYMBOLS = new Set([' ', '~', '^', ':', '?', '*', '[', '\\']);
const FIRST_PRINTABLE = 0x20;
const DELETE = 0x7f;

function hasForbiddenCharacter(name: string): boolean {
  for (const char of name) {
    const code = char.codePointAt(0) ?? 0;
    if (code < FIRST_PRINTABLE || code === DELETE || FORBIDDEN_SYMBOLS.has(char)) return true;
  }
  return false;
}

function componentIssues(name: string): RefNameIssue[] {
  const issues: RefNameIssue[] = [];
  const components = name.split('/');
  if (components.some((component) => component.startsWith('.'))) issues.push('dot-component');
  if (components.some((component) => component.endsWith('.lock'))) issues.push('lock-suffix');
  return issues;
}

function shapeIssues(name: string): RefNameIssue[] {
  const checks: readonly (readonly [boolean, RefNameIssue])[] = [
    [hasForbiddenCharacter(name), 'forbidden-character'],
    [name.includes('..'), 'double-dot'],
    [name.startsWith('-'), 'leading-dash'],
    [name.startsWith('/') || name.endsWith('/') || name.includes('//'), 'slash-position'],
    [name.endsWith('.'), 'trailing-dot'],
    [name.includes('@{'), 'at-brace'],
    [name === '@', 'only-at'],
  ];
  return checks.filter(([failed]) => failed).map(([, issue]) => issue);
}

/** Todos los problemas del nombre, para poder explicarlos a la vez. */
export function refNameIssues(name: string): RefNameIssue[] {
  if (name === '') return ['empty'];
  return [...shapeIssues(name), ...componentIssues(name)];
}

export function isValidRefName(name: string): boolean {
  return refNameIssues(name).length === 0;
}

/** Lo que el prototipo hacia como unica regla, ahora solo como sugerencia al escribir. */
export function suggestBranchName(input: string): string {
  return input.trim().replace(/\s+/gu, '-');
}

export type PromptValidation = 'branch-name' | 'tag-name' | 'non-empty' | 'none';

export type PromptIssue = RefNameIssue;

/** El validador que aplica cada host a una peticion de texto de RepoViewHost. */
export function validatePromptValue(validation: PromptValidation, value: string): PromptIssue[] {
  switch (validation) {
    case 'branch-name':
    case 'tag-name':
      return refNameIssues(value.trim());
    case 'non-empty':
      return value.trim() === '' ? ['empty'] : [];
    case 'none':
      return [];
  }
}
