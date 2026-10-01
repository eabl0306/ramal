import { passwordHint } from '@ramal/domain';
import { TextField, useT } from '@ramal/ui';
import { type ReactElement, useState } from 'react';

import { signInWithProvider, signUp } from '../../application/use-cases/authenticate';
import { useUiPorts } from '../ports/ui-ports';
import { moduleClasses } from '../shared/module-classes';

import { AuthCard } from './auth-card';
import styles from './auth.module.css';
import { useAuthSubmit } from './use-auth-submit';

const css = moduleClasses(styles);

interface SignupFields {
  readonly name: string;
  readonly email: string;
  readonly password: string;
  readonly organization: string;
}

const EMPTY: SignupFields = { name: '', email: '', password: '', organization: '' };

interface SignupFieldsetProps {
  readonly fields: SignupFields;
  readonly onEdit: (field: keyof SignupFields) => (value: string) => void;
  readonly termsAccepted: boolean;
  readonly onTerms: (accepted: boolean) => void;
}

function TermsField({
  accepted,
  onChange,
}: {
  readonly accepted: boolean;
  readonly onChange: (accepted: boolean) => void;
}): ReactElement {
  const t = useT();
  return (
    <label className={css('terms')}>
      <input
        type="checkbox"
        name="terms"
        checked={accepted}
        onChange={(event) => {
          onChange(event.target.checked);
        }}
      />
      <span>{t('web.auth.terms')}</span>
    </label>
  );
}

/** Los campos en el orden del diseno; la pista de la contrasena cambia mientras se escribe. */
function SignupFieldset({
  fields,
  onEdit,
  termsAccepted,
  onTerms,
}: SignupFieldsetProps): ReactElement {
  const t = useT();
  const hintOk = passwordHint(fields.password) === 'ok';
  return (
    <>
      <TextField
        label={t('web.auth.name')}
        name="name"
        autoComplete="name"
        value={fields.name}
        onChange={onEdit('name')}
      />
      <TextField
        label={t('web.auth.email')}
        type="email"
        name="email"
        autoComplete="email"
        value={fields.email}
        onChange={onEdit('email')}
      />
      <TextField
        label={t('web.auth.password')}
        type="password"
        name="password"
        autoComplete="new-password"
        value={fields.password}
        onChange={onEdit('password')}
      />
      <p className={css('hint')} data-ok={hintOk ? 'true' : 'false'}>
        {t(hintOk ? 'web.auth.passwordOk' : 'web.auth.passwordHint')}
      </p>
      <TextField
        label={t('web.auth.organization')}
        name="organization"
        autoComplete="organization"
        value={fields.organization}
        onChange={onEdit('organization')}
      />
      <TermsField accepted={termsAccepted} onChange={onTerms} />
    </>
  );
}

/**
 * Crear cuenta: nombre, email, contrasena, organizacion opcional y terminos. La
 * validacion es la de @ramal/domain y el envio va al AuthGateway del composition
 * root.
 */
export function SignupScreen(): ReactElement {
  const t = useT();
  const { auth } = useUiPorts();
  const submit = useAuthSubmit();
  const [fields, setFields] = useState<SignupFields>(EMPTY);
  const [termsAccepted, setTermsAccepted] = useState(false);

  return (
    <AuthCard
      title={t('web.auth.signupTitle')}
      subtitle={t('web.auth.signupSubtitle')}
      submitLabel={t('web.nav.signup')}
      submit={submit}
      onSubmit={() => {
        submit.run(() => signUp(auth, { ...fields, termsAccepted }));
      }}
      onProvider={(provider) => {
        submit.run(() => signInWithProvider(auth, provider));
      }}
      switchText={t('web.auth.haveAccount')}
      switchLink={{ to: '/login', label: t('web.nav.login') }}
    >
      <SignupFieldset
        fields={fields}
        termsAccepted={termsAccepted}
        onEdit={(field) => (value) => {
          setFields((current) => ({ ...current, [field]: value }));
          submit.clear();
        }}
        onTerms={(accepted) => {
          setTermsAccepted(accepted);
          submit.clear();
        }}
      />
    </AuthCard>
  );
}
