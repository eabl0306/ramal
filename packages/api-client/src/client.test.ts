import { describe, expect, it } from 'vitest';

import { createApiClient, normalizeBaseUrl } from './client';

const READY_BODY = {
  status: 'ok',
  checks: [{ name: 'database', status: 'ok', latencyMs: 3 }],
  version: '0.1.0',
  checkedAt: '2026-09-30T10:00:00Z',
};

function jsonResponse(body: unknown, status: number, type = 'application/json'): Response {
  return new Response(JSON.stringify(body), { status, headers: { 'content-type': type } });
}

/** fetch falso que guarda las peticiones y contesta siempre lo mismo. */
function recordingFetch(response: () => Response): {
  fetch: (input: Request) => Promise<Response>;
  requests: Request[];
} {
  const requests: Request[] = [];
  return {
    requests,
    fetch: (input) => {
      requests.push(input);
      return Promise.resolve(response());
    },
  };
}

describe('normalizeBaseUrl', () => {
  it('quita las barras finales y los espacios', () => {
    expect(normalizeBaseUrl(' https://api.ramal.bahoque.com/ ')).toBe(
      'https://api.ramal.bahoque.com',
    );
    expect(normalizeBaseUrl('https://api.ramal.bahoque.com///')).toBe(
      'https://api.ramal.bahoque.com',
    );
  });

  it('deja vacio el mismo origen', () => {
    expect(normalizeBaseUrl('')).toBe('');
  });
});

describe('createApiClient', () => {
  it('pide la ruta tipada bajo el origen configurado y acepta JSON', async () => {
    const fake = recordingFetch(() => jsonResponse(READY_BODY, 200));
    const client = createApiClient({ baseUrl: 'http://ramal.test/', fetch: fake.fetch });

    const { data, error } = await client.GET('/health/ready');

    expect(error).toBeUndefined();
    expect(data?.status).toBe('ok');
    expect(fake.requests).toHaveLength(1);
    expect(fake.requests[0]?.url).toBe('http://ramal.test/health/ready');
    expect(fake.requests[0]?.headers.get('accept')).toBe('application/json');
  });

  it('sin opciones queda listo para el mismo origen y el fetch del navegador', () => {
    const client = createApiClient();

    expect(typeof client.GET).toBe('function');
  });

  it('entrega el cuerpo de un 503 como error, con la respuesta para leer el estado', async () => {
    const degraded = { ...READY_BODY, status: 'degraded' };
    const fake = recordingFetch(() => jsonResponse(degraded, 503));
    const client = createApiClient({ baseUrl: 'http://ramal.test', fetch: fake.fetch });

    const { data, error, response } = await client.GET('/health/ready');

    expect(data).toBeUndefined();
    expect(response.status).toBe(503);
    expect(error).toEqual(degraded);
  });
});
