import { useT } from '@ramal/ui';
import { type ReactElement, useState } from 'react';

import { signIn, signInWithProvider } from '../../application/use-cases/authenticate';
import { useUiPorts } from '../ports/ui-ports';

import { AuthCard } from './auth-card';
import { AuthField } from './auth-field';
import { useAuthSubmit } from './use-auth-submit';

/**
 * Entrar con email y contrasena. Al entrar solo se pide que la contrasena no
 * este vacia: el minimo de 8 caracteres es del registro (gaps B-53).
 */
export function LoginScreen(): ReactElement {
  const t = useT();
  const { auth } = useUiPorts();
  const submit = useAuthSubmit();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');

  const edit =
    (setter: (value: string) => void) =>
    (value: string): void => {
      setter(value);
      submit.clear();
    };

  return (
    <AuthCard
      title={t('web.auth.loginTitle')}
      subtitle={t('web.auth.loginSubtitle')}
      submitLabel={t('web.nav.login')}
      submit={submit}
      onSubmit={() => {
        submit.run(() => signIn(auth, { email, password }));
      }}
      onProvider={(provider) => {
        submit.run(() => signInWithProvider(auth, provider));
      }}
      switchText={t('web.auth.noAccount')}
      switchLink={{ to: '/signup', label: t('web.nav.signup') }}
    >
      <AuthField
        label={t('web.auth.email')}
        type="email"
        name="email"
        autoComplete="email"
        value={email}
        onChange={edit(setEmail)}
      />
      <AuthField
        label={t('web.auth.password')}
        type="password"
        name="password"
        autoComplete="current-password"
        value={password}
        onChange={edit(setPassword)}
      />
    </AuthCard>
  );
}
