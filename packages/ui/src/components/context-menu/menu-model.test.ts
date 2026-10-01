import { describe, expect, it, vi } from 'vitest';

import { type MenuActionEntry, groupEntries, toggledEntry } from './menu-model';

describe('groupEntries', () => {
  it('junta las casillas consecutivas y deja sueltos el resto', () => {
    const groups = groupEntries([
      { id: 'a', label: 'A' },
      { id: 'w1', label: 'W1', checked: true },
      { id: 'w2', label: 'W2', checked: false },
      { kind: 'separator', id: 's' },
      { id: 'w3', label: 'W3', checked: false },
    ]);
    expect(groups.map((group) => group.kind)).toEqual(['item', 'checks', 'separator', 'checks']);
    const checks = groups[1];
    expect(checks?.kind === 'checks' ? checks.entries.map((entry) => entry.id) : []).toEqual([
      'w1',
      'w2',
    ]);
  });
});

describe('toggledEntry', () => {
  it('encuentra la casilla que cambio respecto a lo marcado', () => {
    const onAction = vi.fn();
    const entries: MenuActionEntry[] = [
      { id: 'w1', label: 'W1', checked: true },
      { id: 'w2', label: 'W2', checked: false, onAction },
    ];
    expect(toggledEntry(entries, new Set(['w1', 'w2']))?.id).toBe('w2');
    expect(toggledEntry(entries, new Set())?.id).toBe('w1');
    expect(toggledEntry(entries, new Set(['w1']))).toBeUndefined();
  });
});
