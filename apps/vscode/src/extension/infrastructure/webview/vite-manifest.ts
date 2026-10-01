import { z } from 'zod';

/**
 * El manifiesto que escribe `vite build` (`build.manifest`): por cada entrada,
 * su JS, sus CSS y los trozos que importa. El host lo lee para saber que
 * ficheros cargar en cada webview sin escribir nombres con hash a mano.
 */
const chunkSchema = z.object({
  file: z.string(),
  src: z.string().optional(),
  isEntry: z.boolean().optional(),
  css: z.array(z.string()).optional(),
  imports: z.array(z.string()).optional(),
});

const manifestSchema = z.record(z.string(), chunkSchema);

export type ViteManifest = z.infer<typeof manifestSchema>;

/** Lo que hay que cargar para una entrada: un modulo y sus hojas de estilo. */
export interface EntryAssets {
  readonly script: string;
  readonly styles: readonly string[];
}

/** Valida el JSON del manifiesto. Lanza si no tiene la forma de Vite. */
export function parseViteManifest(json: unknown): ViteManifest {
  return manifestSchema.parse(json);
}

/**
 * El JS de una entrada y todos los CSS que necesita, los suyos y los de los
 * trozos que importa, sin repetir y en orden de dependencia.
 */
export function entryAssets(manifest: ViteManifest, entry: string): EntryAssets {
  const chunk = manifest[entry];
  if (chunk === undefined) throw new Error(`the webview build has no entry ${entry}`);
  const styles = new Set<string>();
  const visited = new Set<string>();
  const visit = (key: string): void => {
    const current = manifest[key];
    if (current === undefined || visited.has(key)) return;
    visited.add(key);
    for (const imported of current.imports ?? []) visit(imported);
    for (const style of current.css ?? []) styles.add(style);
  };
  visit(entry);
  return { script: chunk.file, styles: [...styles] };
}
