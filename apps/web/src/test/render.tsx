import type { DetectedPlatform } from '@ramal/domain';
import { type RenderResult, render } from '@testing-library/react';
import type { ReactElement } from 'react';
import { MemoryRouter } from 'react-router';
import { vi } from 'vitest';

import { AppProviders } from '../app/app-providers';
import type { Clipboard } from '../application/ports/clipboard';
import type { VisitorEnvironment, VisitorSnapshot } from '../application/ports/visitor-environment';
import type { Language } from '../domain/preferences/preferences';
import { createUnavailableAuthGateway } from '../infrastructure/auth/unavailable-auth-gateway';
import { createLocalPreferencesStore } from '../infrastructure/storage/local-preferences-store';
import type { UiPorts } from '../ui/ports/ui-ports';

/** Un navegador fijo: el sistema que se quiera probar, sin leer el de verdad. */
export function fixedVisitor(platform: DetectedPlatform): VisitorEnvironment {
  const snapshot: VisitorSnapshot = { platform, languageTag: null, systemTheme: null };
  return { snapshot: () => snapshot };
}

export function recordingClipboard(): Clipboard & { readonly copied: string[] } {
  const copied: string[] = [];
  return {
    copied,
    copy: vi.fn((text: string) => {
      copied.push(text);
      return Promise.resolve(true);
    }),
  };
}

/** Los puertos de un test: los adaptadores en memoria del primer corte. */
export function testPorts(overrides: Partial<UiPorts> = {}): UiPorts {
  const items = new Map<string, string>();
  const storage = {
    getItem: (key: string) => items.get(key) ?? null,
    setItem: (key: string, value: string) => {
      items.set(key, value);
    },
  };
  return {
    auth: createUnavailableAuthGateway(),
    preferences: createLocalPreferencesStore(() => storage),
    visitor: fixedVisitor({ os: 'unknown', arch: 'unknown' }),
    clipboard: recordingClipboard(),
    ...overrides,
  };
}

export interface ScreenOptions {
  readonly language?: Language;
  readonly path?: string;
  readonly ports?: UiPorts;
}

/** Monta una pantalla con los mismos providers que root.tsx y un router en memoria. */
export function renderScreen(ui: ReactElement, options: ScreenOptions = {}): RenderResult {
  const { path = '/' } = options;
  return renderWithProviders(<MemoryRouter initialEntries={[path]}>{ui}</MemoryRouter>, options);
}

/** Solo los providers, para montar un stub de rutas que ya trae su propio router. */
export function renderWithProviders(ui: ReactElement, options: ScreenOptions = {}): RenderResult {
  const { language = 'es', ports = testPorts() } = options;
  return render(
    <AppProviders ports={ports} language={language}>
      {ui}
    </AppProviders>,
  );
}
