import { createContext, use } from 'react';

/** `info` usa el color de informacion, no el de acento (gaps B-61). */
export type ToastKind = 'ok' | 'warn' | 'error' | 'info';

export interface ToastMessage {
  readonly id: number;
  readonly text: string;
  readonly kind: ToastKind;
}

export interface ToastApi {
  /** Ensena un aviso que se va solo; devuelve su id por si hay que quitarlo antes. */
  readonly show: (text: string, kind?: ToastKind) => number;
  readonly dismiss: (id: number) => void;
}

export const ToastContext = createContext<ToastApi | null>(null);

/** `const toast = useToast(); toast.show(t('toast.fetched'))`. */
export function useToast(): ToastApi {
  const api = use(ToastContext);
  if (api === null) {
    throw new Error('useToast necesita un <ToastProvider> por encima en el arbol.');
  }
  return api;
}
