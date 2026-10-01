import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { HealthProbe } from '../../application/use-cases/check-api-health';
import { createHealthCheck, createHealthReport } from '../../domain/health/health-report';
import { renderScreen } from '../../test/render';

import { StatusScreen } from './status-screen';

const CHECKED_AT = new Date('2026-09-30T10:00:00Z');

function reachable(status: 'ok' | 'error', name = 'database'): HealthProbe {
  return {
    outcome: 'reachable',
    report: createHealthReport({
      checks: [createHealthCheck(name, status, 3)],
      version: '0.1.0',
      checkedAt: CHECKED_AT,
    }),
  };
}

describe('StatusScreen', () => {
  it('ensena el servicio operativo con sus componentes y su version', () => {
    renderScreen(<StatusScreen probe={reachable('ok')} retrying={false} onRetry={vi.fn()} />);

    expect(screen.getByTestId('health-status')).toHaveAttribute('data-status', 'ok');
    expect(screen.getByTestId('health-status')).toHaveTextContent('Operativo');
    expect(screen.getByTestId('health-check')).toHaveTextContent('Base de datos');
    expect(screen.getByTestId('health-check')).toHaveTextContent('Responde');
    expect(screen.getByText(/Versión 0\.1\.0/u)).toBeInTheDocument();
  });

  it('ensena el servicio degradado y el componente que no responde', () => {
    renderScreen(<StatusScreen probe={reachable('error')} retrying={false} onRetry={vi.fn()} />);

    expect(screen.getByTestId('health-status')).toHaveAttribute('data-status', 'degraded');
    expect(screen.getByTestId('health-check')).toHaveAttribute('data-status', 'error');
    expect(screen.getByTestId('health-check')).toHaveTextContent('No responde');
  });

  it('nombra con su nombre tecnico un componente que la web no conoce', () => {
    renderScreen(
      <StatusScreen probe={reachable('ok', 'cache')} retrying={false} onRetry={vi.fn()} />,
      { language: 'en' },
    );

    expect(screen.getByTestId('health-check')).toHaveTextContent('cache');
    expect(screen.getByTestId('health-status')).toHaveTextContent('Operational');
  });

  it('sin respuesta de la API lo dice y deja volver a comprobar', async () => {
    const user = userEvent.setup();
    const onRetry = vi.fn();
    renderScreen(
      <StatusScreen probe={{ outcome: 'unreachable' }} retrying={false} onRetry={onRetry} />,
    );

    expect(screen.getByTestId('health-status')).toHaveAttribute('data-status', 'unreachable');
    expect(screen.queryByTestId('health-check')).not.toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Volver a comprobar' }));
    expect(onRetry).toHaveBeenCalledTimes(1);
  });
});
