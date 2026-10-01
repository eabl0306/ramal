import { Button, useT } from '@ramal/ui';
import type { ReactElement, ReactNode, SubmitEvent } from 'react';
import { Link } from 'react-router';

import type { OAuthProvider } from '../../application/ports/auth-gateway';
import { PRODUCT_NAME, moduleClasses } from '../shared/module-classes';

import styles from './auth.module.css';
import type { AuthSubmit } from './use-auth-submit';

const css = moduleClasses(styles);

/** Los proveedores de identidad del diseno; sus nombres son marcas y no se traducen. */
const PROVIDERS: readonly { readonly id: OAuthProvider; readonly name: string }[] = [
  { id: 'github', name: 'GitHub' },
  { id: 'gitlab', name: 'GitLab' },
  { id: 'google', name: 'Google' },
];

export interface AuthCardProps {
  readonly title: string;
  readonly subtitle: string;
  readonly submitLabel: string;
  readonly submit: AuthSubmit;
  readonly onSubmit: () => void;
  readonly onProvider: (provider: OAuthProvider) => void;
  /** "No tienes cuenta? Crear cuenta" o al reves. */
  readonly switchText: string;
  readonly switchLink: { readonly to: string; readonly label: string };
  readonly children: ReactNode;
}

/**
 * La tarjeta comun de entrar y crear cuenta: proveedores, campos, la caja de
 * error y el envio. Es un <form> de verdad, asi Enter envia (el prototipo no
 * tenia formulario), y noValidate deja los mensajes a @ramal/domain.
 */
export function AuthCard(props: AuthCardProps): ReactElement {
  const t = useT();
  const { submit } = props;
  const send = (event: SubmitEvent<HTMLFormElement>): void => {
    event.preventDefault();
    props.onSubmit();
  };

  return (
    <main className={css('page')}>
      <title>{`${props.title} · ${PRODUCT_NAME}`}</title>
      <div className={css('card')}>
        <div>
          <h1 className={css('title')}>{props.title}</h1>
          <p className={css('subtitle')}>{props.subtitle}</p>
        </div>
        <div className={css('providers')}>
          {PROVIDERS.map((provider) => (
            <Button
              key={provider.id}
              size={42}
              block
              isDisabled={submit.pending}
              onPress={() => {
                props.onProvider(provider.id);
              }}
            >
              {t('web.auth.continueWith', { provider: provider.name })}
            </Button>
          ))}
        </div>
        <div className={css('divider')}>{t('web.auth.orEmail')}</div>
        <form className={css('form')} noValidate onSubmit={send}>
          {props.children}
          {submit.message === null ? null : (
            <div className={css('error')} role="alert" data-testid="auth-error">
              {t(submit.message)}
            </div>
          )}
          <Button type="submit" variant="primary" size={42} block isDisabled={submit.pending}>
            {props.submitLabel}
          </Button>
        </form>
        <p className={css('switch')}>
          {props.switchText} <Link to={props.switchLink.to}>{props.switchLink.label}</Link>
        </p>
      </div>
    </main>
  );
}
