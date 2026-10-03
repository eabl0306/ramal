/**
 * La unica tabla de plataformas Git (gaps B-38: el prototipo tenia hosts por
 * defecto distintos en cada producto). El nombre es una marca y no se traduce.
 * El color del distintivo no vive aqui sino en los tokens de @git-rudder/ui.
 */

export const PLATFORM_IDS = ['github', 'gitlab', 'bitbucket', 'gitea', 'azure'] as const;

export type PlatformId = (typeof PLATFORM_IDS)[number];

export interface PlatformInfo {
  readonly id: PlatformId;
  readonly label: string;
  /** Las dos letras del distintivo. */
  readonly abbr: string;
  readonly defaultHost: string;
}

export const PLATFORMS: Readonly<Record<PlatformId, PlatformInfo>> = {
  github: { id: 'github', label: 'GitHub', abbr: 'GH', defaultHost: 'github.com' },
  gitlab: { id: 'gitlab', label: 'GitLab', abbr: 'GL', defaultHost: 'gitlab.com' },
  bitbucket: { id: 'bitbucket', label: 'Bitbucket', abbr: 'BB', defaultHost: 'bitbucket.org' },
  gitea: { id: 'gitea', label: 'Gitea', abbr: 'GT', defaultHost: 'gitea.com' },
  azure: { id: 'azure', label: 'Azure DevOps', abbr: 'AZ', defaultHost: 'dev.azure.com' },
};

/** GitLab numera sus merge requests con "!"; el resto, sus pull requests con "#". */
export function prNumberLabel(platform: PlatformId, number: number): string {
  return `${platform === 'gitlab' ? '!' : '#'}${String(number)}`;
}
