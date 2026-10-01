import type { PlainKey } from '@ramal/i18n';
import { useLocale, useT, useToast } from '@ramal/ui';
import { useState } from 'react';
import { useNavigate } from 'react-router';

import type { AuthAttempt } from '../../application/use-cases/authenticate';
import { landingPath } from '../../domain/preferences/page-preferences';

import { FAILURE_MESSAGES, firstFieldMessage } from './auth-messages';

export interface AuthSubmit {
  /** La clave del texto de la caja de error, o null si no hay error. */
  readonly message: PlainKey | null;
  /** Mientras espera a la API el boton se desactiva (gaps W-07). */
  readonly pending: boolean;
  readonly run: (attempt: () => Promise<AuthAttempt>) => void;
  /** Escribir en cualquier campo borra el error, como en el diseno. */
  readonly clear: () => void;
}

/**
 * Envio comun de entrar, crear cuenta y los botones de proveedor: lanza el caso
 * de uso y traduce su resultado a la caja de error o, si entra, a un aviso y la
 * vuelta a la landing (todavia no hay panel al que ir).
 */
export function useAuthSubmit(): AuthSubmit {
  const t = useT();
  const locale = useLocale();
  const toast = useToast();
  const navigate = useNavigate();
  const [message, setMessage] = useState<PlainKey | null>(null);
  const [pending, setPending] = useState(false);

  const settle = (attempt: AuthAttempt): void => {
    switch (attempt.outcome) {
      case 'invalid':
        setMessage(firstFieldMessage(attempt.errors));
        return;
      case 'rejected':
        setMessage(FAILURE_MESSAGES[attempt.failure]);
        return;
      case 'signed-in':
        setMessage(null);
        toast.show(t('web.auth.welcome', { email: attempt.session.email }));
        void navigate(landingPath(locale));
    }
  };

  const run = (attempt: () => Promise<AuthAttempt>): void => {
    setPending(true);
    void attempt()
      .then(settle)
      .finally(() => {
        setPending(false);
      });
  };

  return {
    message,
    pending,
    run,
    clear: () => {
      setMessage(null);
    },
  };
}
