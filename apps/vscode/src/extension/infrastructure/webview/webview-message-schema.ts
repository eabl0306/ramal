import { Buffer } from 'node:buffer';

import { z } from 'zod';

import {
  type HostMethod,
  type HostMethods,
  SURFACES,
  type WebviewToHostMessage,
} from '../../../shared/messages';

/** Las peticiones de un webview son pequenas: lo que pase de 1 MiB se descarta. */
export const MAX_MESSAGE_BYTES = 1024 * 1024;

/** Id de JSON-RPC: entero no negativo menor que 2^31, como exige el motor. */
const requestId = z.int().min(0).max(2_147_483_647);

/** `params` es siempre un objeto JSON; nunca un array, un texto o `null`. */
const jsonObject = z.record(z.string(), z.json());

const webviewReady = z.strictObject({
  type: z.literal('webview/ready'),
  surface: z.enum(SURFACES),
});

const rpcRequest = z.strictObject({
  type: z.literal('rpc/request'),
  request: z.strictObject({
    jsonrpc: z.literal('2.0'),
    id: requestId,
    method: z.string().min(1).max(128),
    params: jsonObject,
  }),
});

/**
 * Un esquema por metodo nativo: sus parametros se validan antes de llegar a la
 * API de VS Code. Un metodo nuevo en `HostMethods` no compila sin el suyo.
 */
const HOST_PARAMS = {
  'clipboard/writeText': z.strictObject({ text: z.string().max(100_000) }),
} satisfies { readonly [M in HostMethod]: z.ZodType<HostMethods[M]['params']> };

function hostRequestOf<M extends HostMethod>(method: M) {
  return z.strictObject({
    type: z.literal('host/request'),
    id: requestId,
    method: z.literal(method),
    params: HOST_PARAMS[method],
  });
}

const hostRequest = z.discriminatedUnion('method', [hostRequestOf('clipboard/writeText')]);

/** Discriminada por `type`: un fallo dice que campo de que mensaje no encaja. */
const webviewMessage = z.discriminatedUnion('type', [webviewReady, rpcRequest, hostRequest]);

export type ParsedWebviewMessage =
  | { readonly ok: true; readonly message: WebviewToHostMessage }
  | { readonly ok: false; readonly reason: string };

/**
 * Pasos 2 y 3 de architecture.md 6.7: tamano y forma. Un webview pinta contenido
 * que no es de fiar, asi que lo que llega se trata como hostil: lo que no se
 * puede serializar, pasa de 1 MiB o no encaja exactamente en la union de
 * mensajes (campos de mas incluidos) se descarta. Nunca lanza.
 */
export function parseWebviewMessage(raw: unknown): ParsedWebviewMessage {
  const size = serializedSize(raw);
  if (size === null) return { ok: false, reason: 'the message is not serializable' };
  if (size > MAX_MESSAGE_BYTES) {
    return { ok: false, reason: `the message has ${String(size)} bytes` };
  }
  try {
    const parsed = webviewMessage.safeParse(raw);
    if (parsed.success) return { ok: true, message: parsed.data };
    return { ok: false, reason: `invalid shape: ${firstIssue(parsed.error)}` };
  } catch {
    // Un objeto anidado a lo bestia puede agotar la pila al recorrerlo.
    return { ok: false, reason: 'the message could not be validated' };
  }
}

function serializedSize(raw: unknown): number | null {
  try {
    const text = JSON.stringify(raw) as string | undefined;
    return text === undefined ? null : Buffer.byteLength(text, 'utf8');
  } catch {
    return null;
  }
}

function firstIssue(error: z.ZodError): string {
  const [issue] = error.issues;
  if (issue === undefined) return 'unknown';
  const where = issue.path.length === 0 ? '(root)' : issue.path.join('.');
  return `${where}: ${issue.message}`;
}
