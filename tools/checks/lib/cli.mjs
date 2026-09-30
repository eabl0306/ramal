import { repoRoot } from './scan.mjs';

/** Lee las opciones comunes: --staged para el hook y --root para los tests. */
export function parseArgs(argv) {
  const options = { staged: false, root: repoRoot() };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--staged') options.staged = true;
    if (arg === '--root') {
      index += 1;
      const value = argv[index];
      if (value !== undefined) options.root = value;
    }
  }
  return options;
}

/**
 * Imprime el informe y decide el codigo de salida. Una violacion por linea,
 * con ruta, posicion y motivo, para que se pueda pinchar desde el editor.
 */
export function report({ name, scanned, violations }) {
  const total = violations.length;
  if (total === 0) {
    process.stdout.write(`${name}: ${String(scanned)} ficheros revisados, todo en orden\n`);
    return 0;
  }
  process.stderr.write(`${name}: ${String(total)} violacion(es) en ${String(scanned)} ficheros\n`);
  for (const violation of violations) {
    const where =
      violation.line === undefined ? violation.file : `${violation.file}:${String(violation.line)}`;
    process.stderr.write(`  ${where}  ${violation.message}\n`);
  }
  process.stderr.write(`\n${name}: arregla lo de arriba antes de commitear.\n`);
  return 1;
}

export function run(main) {
  process.exitCode = main(parseArgs(process.argv.slice(2)));
}
