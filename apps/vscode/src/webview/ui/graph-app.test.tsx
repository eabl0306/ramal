import type { BridgeClient, EngineMethod, GraphWindow } from '@ramal/bridge';
import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { GraphApp } from './graph-app';

const WINDOW: GraphWindow = {
  repoId: 'repo-1',
  offset: 0,
  hasMore: false,
  laneCount: 1,
  rows: [
    {
      sha: 'a'.repeat(40),
      shortSha: 'aaaaaaa',
      parents: [],
      lane: 0,
      edges: [],
      summary: 'Primer commit',
      authorName: 'Ana Ruiz',
      authorEmail: 'ana@example.com',
      authoredAt: '2026-04-12T10:24:00+02:00',
      refs: [{ name: 'main', fullName: 'refs/heads/main', kind: 'localBranch', isHead: true }],
      isHead: true,
    },
  ],
};

/** Cliente falso: contesta la ventana del grafo y guarda lo que se pidio. */
function fakeClient(): BridgeClient & { readonly calls: EngineMethod[] } {
  const calls: EngineMethod[] = [];
  return {
    calls,
    call: (method) => {
      calls.push(method);
      return Promise.resolve(WINDOW);
    },
    onNotification: () => () => undefined,
  };
}

describe('superficie del grafo en VS Code', () => {
  it('pinta el grafo del repo con densidad compacta y en el idioma de VS Code', async () => {
    const client = fakeClient();
    const { container } = render(<GraphApp client={client} locale="es" repoId="repo-1" />);
    expect(await screen.findByText('Primer commit')).toBeInTheDocument();
    expect(container.querySelector('[data-density="compact"]')).not.toBeNull();
    expect(screen.getByRole('grid', { name: 'Historial de commits' })).toBeInTheDocument();
    expect(client.calls).toEqual(['graph/window']);
  });
});
