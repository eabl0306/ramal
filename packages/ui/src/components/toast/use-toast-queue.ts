import { useCallback, useEffect, useRef, useState } from 'react';

import type { ToastKind, ToastMessage } from './toast-context';

export interface ToastQueue {
  readonly toasts: readonly ToastMessage[];
  readonly show: (text: string, kind?: ToastKind) => number;
  readonly dismiss: (id: number) => void;
}

/**
 * La cola de avisos: cada uno se va solo pasado `duration`. Los temporizadores
 * se limpian al quitar un aviso a mano y al desmontar, para no tocar estado de
 * un componente que ya no existe.
 */
export function useToastQueue(duration: number): ToastQueue {
  const [toasts, setToasts] = useState<readonly ToastMessage[]>([]);
  const nextIdRef = useRef(1);
  const timersRef = useRef(new Map<number, ReturnType<typeof setTimeout>>());

  const dismiss = useCallback((id: number) => {
    const timer = timersRef.current.get(id);
    if (timer !== undefined) clearTimeout(timer);
    timersRef.current.delete(id);
    setToasts((current) => current.filter((toast) => toast.id !== id));
  }, []);

  const show = useCallback(
    (text: string, kind: ToastKind = 'ok') => {
      const id = nextIdRef.current;
      nextIdRef.current += 1;
      setToasts((current) => [...current, { id, text, kind }]);
      const timer = setTimeout(() => {
        dismiss(id);
      }, duration);
      timersRef.current.set(id, timer);
      return id;
    },
    [dismiss, duration],
  );

  useEffect(() => {
    const timers = timersRef.current;
    return () => {
      for (const timer of timers.values()) clearTimeout(timer);
    };
  }, []);

  return { toasts, show, dismiss };
}
