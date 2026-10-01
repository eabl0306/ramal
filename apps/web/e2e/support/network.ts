import type { Page } from '@playwright/test';

/**
 * Los e2e no dependen de que la API este levantada: se intercepta la llamada.
 * Con E2E_USE_REAL_API=1 no se intercepta nada y se prueba contra la pila real.
 */
export const USE_REAL_API = process.env['E2E_USE_REAL_API'] === '1';

const READINESS_PATTERN = '**/health/ready';

export const READY_BODY = {
  status: 'ok',
  checks: [{ name: 'database', status: 'ok', latencyMs: 3 }],
  version: '0.1.0',
  checkedAt: '2026-09-30T10:00:00Z',
};

export const DEGRADED_BODY = {
  ...READY_BODY,
  status: 'degraded',
  checks: [{ name: 'database', status: 'error', latencyMs: 2000 }],
};

export async function mockReadiness(page: Page, body: unknown, status: number): Promise<void> {
  await page.route(READINESS_PATTERN, async (route) => {
    await route.fulfill({
      status,
      contentType: 'application/json',
      body: JSON.stringify(body),
    });
  });
}

export async function breakReadiness(page: Page): Promise<void> {
  await page.route(READINESS_PATTERN, async (route) => {
    await route.abort('connectionrefused');
  });
}
