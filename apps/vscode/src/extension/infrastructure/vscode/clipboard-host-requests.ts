import * as vscode from 'vscode';

import type { HostMethod, HostMethods } from '../../../shared/messages';
import type { HostRequests } from '../../application/ports/host-requests';

type Handlers = {
  readonly [M in HostMethod]: (
    params: HostMethods[M]['params'],
  ) => Promise<HostMethods[M]['result']>;
};

/** Un manejador por metodo nativo; uno nuevo en `HostMethods` no compila sin el suyo. */
const HANDLERS: Handlers = {
  'clipboard/writeText': async ({ text }) => {
    await vscode.env.clipboard.writeText(text);
    return null;
  },
};

/** Las peticiones nativas de los webviews, resueltas con la API de VS Code. */
export const vscodeHostRequests: HostRequests = {
  handle(method, params) {
    return HANDLERS[method](params);
  },
};
