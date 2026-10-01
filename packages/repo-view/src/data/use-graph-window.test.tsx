import { act, renderHook, waitFor } from '@testing-library/react';
import type { ReactElement, ReactNode } from 'react';
import { describe, expect, it } from 'vitest';

import { FakeEngine, fakeClient } from '../test/fake-engine';
import { graphRow, linearRows } from '../test/fixtures';
import { testQueryClient } from '../test/render';
import { RepoViewProvider } from '../view/repo-view-provider';

import { type GraphWindowState, useGraphWindow } from './use-graph-window';

function mountHook(engine: FakeEngine, pageSize: number): { current: GraphWindowState } {
  const client = fakeClient(engine);
  const queryClient = testQueryClient();
  function Wrapper({ children }: { readonly children: ReactNode }): ReactElement {
    return <RepoViewProvider queryClient={queryClient}>{children}</RepoViewProvider>;
  }
  return renderHook(() => useGraphWindow(client, 'repo-1', { pageSize }), { wrapper: Wrapper })
    .result;
}

function shas(state: GraphWindowState): string[] {
  return state.rows.map((row) => row.sha);
}

describe('useGraphWindow: el repositorio cambia entre dos paginas', () => {
  it('pide la pagina siguiente con la huella de la primera', async () => {
    const engine = new FakeEngine(linearRows(6));
    const result = mountHook(engine, 3);
    await waitFor(() => {
      expect(result.current.rows).toHaveLength(3);
    });

    act(() => {
      result.current.loadMore();
    });

    await waitFor(() => {
      expect(result.current.rows).toHaveLength(6);
    });
    expect(engine.calls.map((call) => [call.offset, call.snapshot])).toEqual([
      [0, undefined],
      [3, 'snap-1'],
    ]);
  });

  it('vuelve a empezar desde la fila 0 en vez de mezclar dos historiales', async () => {
    // El fallo real: commit desde la terminal con el grafo abierto y luego bajar.
    // Sin huella se unian c0 c1 c2 + c2 c3 c4, con c2 repetido.
    const engine = new FakeEngine(linearRows(6));
    const result = mountHook(engine, 3);
    await waitFor(() => {
      expect(result.current.rows).toHaveLength(3);
    });
    engine.rewrite([graphRow('nuevo'), ...linearRows(6)]);

    act(() => {
      result.current.loadMore();
    });

    await waitFor(() => {
      expect(shas(result.current)).toEqual(['nuevo', 'c0', 'c1']);
    });
    expect(result.current.loadMoreError).toBeNull();
    expect(result.current.status).toBe('success');
    expect(engine.calls.map((call) => [call.offset, call.snapshot])).toEqual([
      [0, undefined],
      [3, 'snap-1'],
      [0, undefined],
    ]);

    act(() => {
      result.current.loadMore();
    });

    await waitFor(() => {
      expect(result.current.rows).toHaveLength(6);
    });
    expect(shas(result.current)).toEqual(['nuevo', 'c0', 'c1', 'c2', 'c3', 'c4']);
    expect(engine.calls.at(-1)).toMatchObject({ offset: 3, snapshot: 'snap-2' });
  });
});
