import { describe, expect, it } from 'vitest';

import { HealthGatewayError } from '../../application/ports/health-gateway';

import { createHttpHealthGateway } from './http-health-gateway';

const BASE_URL = 'http://ramal.test';

const READY = {
  status: 'ok',
  checks: [{ name: 'database', status: 'ok', latencyMs: 3 }],
  version: '0.1.0',
  checkedAt: '2026-09-30T10:00:00.123Z',
};

function respond(body: string, status: number, type = 'application/json') {
  const requests: Request[] = [];
  const fetch = (input: Request): Promise<Response> => {
    requests.push(input);
    return Promise.resolve(new Response(body, { status, headers: { 'content-type': type } }));
  };
  return { fetch, requests };
}

function gatewayFor(body: unknown, status: number, type?: string) {
  const fake = respond(typeof body === 'string' ? body : JSON.stringify(body), status, type);
  return { gateway: createHttpHealthGateway({ baseUrl: BASE_URL, fetch: fake.fetch }), fake };
}

describe('createHttpHealthGateway', () => {
  it('lee /health/ready del mismo cliente y lo pasa al dominio', async () => {
    const { gateway, fake } = gatewayFor(READY, 200);

    const report = await gateway.readReadiness();

    expect(fake.requests[0]?.url).toBe(`${BASE_URL}/health/ready`);
    expect(report.status).toBe('ok');
    expect(report.version).toBe('0.1.0');
    expect(report.checks).toEqual([{ name: 'database', status: 'ok', latencyMs: 3 }]);
    expect(report.checkedAt.toISOString()).toBe('2026-09-30T10:00:00.123Z');
  });

  it('lee el cuerpo de un 503 como informe degradado', async () => {
    const degraded = {
      ...READY,
      status: 'degraded',
      checks: [{ name: 'database', status: 'error', latencyMs: 2000 }],
    };
    const { gateway } = gatewayFor(degraded, 503);

    const report = await gateway.readReadiness();

    expect(report.status).toBe('degraded');
    expect(report.checks[0]?.status).toBe('error');
  });

  it('falla con otro estado, como el 404 problem+json de una ruta mal enrutada', async () => {
    const problem = { type: 'x', title: 'No encontrado', status: 404, detail: 'x', code: 'x' };
    const { gateway } = gatewayFor(problem, 404, 'application/problem+json');

    await expect(gateway.readReadiness()).rejects.toThrow('estado 404');
  });

  it('falla si un 200 trae HTML: el gateway mando la ruta a la web', async () => {
    const { gateway } = gatewayFor('<!doctype html><html></html>', 200, 'text/html');

    await expect(gateway.readReadiness()).rejects.toBeInstanceOf(HealthGatewayError);
  });

  it('falla si el JSON no cumple el contrato', async () => {
    const { gateway } = gatewayFor({ ...READY, checkedAt: 'ayer' }, 200);

    await expect(gateway.readReadiness()).rejects.toThrow('no cumple el contrato');
  });

  it('falla si no hay red', async () => {
    const gateway = createHttpHealthGateway({
      baseUrl: BASE_URL,
      timeoutMs: 50,
      fetch: () => Promise.reject(new TypeError('Failed to fetch')),
    });

    await expect(gateway.readReadiness()).rejects.toBeInstanceOf(HealthGatewayError);
  });
});
