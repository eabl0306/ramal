/**
 * Mensajes que git escribe por su cuenta y que la interfaz propone antes de
 * confirmar. Van en ingles a proposito: es el formato de git, no texto de
 * interfaz, y el historial tiene que parecerse al de cualquier otro cliente.
 */

/** git omite "into <rama>" cuando la rama destino es main o master. */
export function mergeMessage(source: string, target: string): string {
  const omitTarget = target === 'main' || target === 'master';
  return omitTarget ? `Merge branch '${source}'` : `Merge branch '${source}' into ${target}`;
}

export function revertMessage(subject: string, sha: string): string {
  return `Revert "${subject}"\n\nThis reverts commit ${sha}.`;
}

/**
 * La linea que anade `cherry-pick -x`, en una linea propia y con el sha completo
 * (gaps B-12: el prototipo la pegaba al final del asunto).
 */
export function cherryPickMessage(message: string, sha: string): string {
  return `${message.trimEnd()}\n\n(cherry picked from commit ${sha})`;
}
