/**
 * Deteccion del sistema y la arquitectura para la pagina de descargas.
 *
 * Recibe lo que el navegador sabe (user agent y, si existen, las pistas de
 * navigator.userAgentData) en vez de leer `navigator`: asi es pura y se prueba
 * con cadenas reales. El prototipo daba macOS para Android, iOS y cualquier
 * sistema desconocido (gaps B-55); aqui esos casos tienen su propio valor.
 */

export type DesktopOs = 'mac' | 'windows' | 'linux';
export type DetectedOs = DesktopOs | 'android' | 'ios' | 'unknown';
export type CpuArch = 'x64' | 'arm64' | 'unknown';

export interface PlatformHints {
  readonly userAgent: string;
  /** navigator.maxTouchPoints: el iPad moderno se anuncia como un Mac con pantalla tactil. */
  readonly maxTouchPoints?: number;
  /** navigator.userAgentData.platform, cuando el navegador lo da. */
  readonly uaPlatform?: string;
  /** getHighEntropyValues(['architecture']).architecture: "arm" o "x86". */
  readonly architecture?: string;
  /** getHighEntropyValues(['bitness']).bitness: "64" o "32". */
  readonly bitness?: string;
}

export interface DetectedPlatform {
  readonly os: DetectedOs;
  readonly arch: CpuArch;
}

const UA_PLATFORMS: Readonly<Record<string, DetectedOs>> = {
  windows: 'windows',
  macos: 'mac',
  linux: 'linux',
  android: 'android',
  ios: 'ios',
};

function osFromUaPlatform(uaPlatform: string | undefined): DetectedOs | null {
  if (uaPlatform === undefined) return null;
  return UA_PLATFORMS[uaPlatform.trim().toLowerCase()] ?? null;
}

const MAC_PATTERN = /Macintosh|Mac OS X/u;

/** El orden importa: Android dice "Linux" y el iPad dice "Mac OS X". */
function osFromUserAgent(userAgent: string, maxTouchPoints: number): DetectedOs {
  if (/Android/iu.test(userAgent)) return 'android';
  if (/iPhone|iPad|iPod/u.test(userAgent)) return 'ios';
  if (MAC_PATTERN.test(userAgent) && maxTouchPoints > 1) return 'ios';
  if (/Windows|Win64|Win32|WOW64/u.test(userAgent)) return 'windows';
  if (MAC_PATTERN.test(userAgent)) return 'mac';
  if (userAgent.includes('CrOS')) return 'unknown';
  if (/Linux|X11/u.test(userAgent)) return 'linux';
  return 'unknown';
}

export function detectOs(hints: PlatformHints): DetectedOs {
  return (
    osFromUaPlatform(hints.uaPlatform) ??
    osFromUserAgent(hints.userAgent, hints.maxTouchPoints ?? 0)
  );
}

function archFromHints(architecture: string | undefined, bitness: string | undefined): CpuArch {
  if (architecture === undefined || bitness === '32') return 'unknown';
  const normalized = architecture.toLowerCase();
  if (normalized === 'arm') return 'arm64';
  if (normalized === 'x86') return 'x64';
  return 'unknown';
}

/**
 * Safari y Chrome en un Mac con Apple Silicon siguen diciendo "Intel Mac OS X",
 * asi que en macOS el user agent no sirve: sin pistas, la arquitectura es
 * desconocida y quien recomienda decide.
 */
function archFromUserAgent(userAgent: string, os: DetectedOs): CpuArch {
  if (os === 'mac') return 'unknown';
  if (/aarch64|arm64/iu.test(userAgent)) return 'arm64';
  if (/x86_64|x64|Win64|WOW64|amd64/iu.test(userAgent)) return 'x64';
  return 'unknown';
}

export function detectPlatform(hints: PlatformHints): DetectedPlatform {
  const os = detectOs(hints);
  const fromHints = archFromHints(hints.architecture, hints.bitness);
  const arch = fromHints === 'unknown' ? archFromUserAgent(hints.userAgent, os) : fromHints;
  return { os, arch };
}

export function isDesktopOs(os: DetectedOs): os is DesktopOs {
  return os === 'mac' || os === 'windows' || os === 'linux';
}
