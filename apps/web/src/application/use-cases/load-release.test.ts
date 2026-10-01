import { describe, expect, it } from 'vitest';

import type { ReleaseInfo } from '../../domain/downloads/release';

import { loadRelease } from './load-release';

const RELEASE: ReleaseInfo = { version: '2.4.0', releasedAt: '2026-09-30', cards: [] };

describe('loadRelease', () => {
  it('devuelve la publicacion del catalogo', async () => {
    const loaded = await loadRelease({ latest: () => Promise.resolve(RELEASE) });

    expect(loaded).toEqual({ outcome: 'loaded', release: RELEASE });
  });

  it('sin catalogo la landing sigue: el resultado es no disponible, no una excepcion', async () => {
    const loaded = await loadRelease({ latest: () => Promise.reject(new Error('caida')) });

    expect(loaded).toEqual({ outcome: 'unavailable' });
  });
});
