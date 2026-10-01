import { PLATFORMS } from '@ramal/domain';
import { Button, PlatformBadge, useT } from '@ramal/ui';
import type { ReactElement, ReactNode, SubmitEvent } from 'react';
import { Link } from 'react-router';

import type { OAuthProvider } from '../../application/ports/auth-gateway';
import { PRODUCT_NAME, moduleClasses } from '../shared/module-classes';

import styles from './auth.module.css';
import type { AuthSubmit } from './use-auth-submit';

const css = moduleClasses(styles);

/** Los proveedores de identidad del diseno; sus nombres son marcas y no se traducen. */
const PROVIDERS: readonly { readonly id: OAuthProvider; readonly name: string }[] = [
  { id: 'github', name: PLATFORMS.github.label },
  { id: 'gitlab', name: PLATFORMS.gitlab.label },
  { id: 'google', name: 'Google' },
];

/** La letra del distintivo de Google en el prototipo; es una marca y no se traduce. */
const GOOGLE_ABBR = 'G';

/**
 * El distintivo de 20 px de cada proveedor. Es decorativo: el nombre ya va en el
 * texto del boton, y repetirlo haria que el lector de pantalla lo dijera dos veces.
 */
function ProviderBadge({ provider }: { readonly provider: OAuthProvider }): ReactElement {
  if (provider === 'google') {
    return (
      <span className={css('google-badge')} aria-hidden="true">
        {GOOGLE_ABBR}
      </span>
    );
  }
  const platform = PLATFORMS[provider];
  return (
    <span className={css('badge-slot')} aria-hidden="true">
      <PlatformBadge platform={provider} abbr={platform.abbr} name={platform.label} size={20} />
    </span>
  );
}

interface ProviderButtonsProps {
  readonly pending: boolean;
  readonly onProvider: (provider: OAuthProvider) => void;
}

/** Los botones de proveedor del prototipo: fondo de la pagina, distintivo y borde de acento al pasar. */
function ProviderButtons({ pending, onProvider }: ProviderButtonsProps): ReactElement {
  const t = useT();
  return (
    <div className={css('providers')}>
      {PROVIDERS.map((provider) => (
        <Button
          key={provider.id}
          variant="outline"
          size={42}
          block
          isDisabled={pending}
          onPress={() => {
            onProvider(provider.id);
          }}
        >
          <ProviderBadge provider={provider.id} />
          {t('web.auth.continueWith', { provider: provider.name })}
        </Button>
      ))}
    </div>
  );
}

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
        <ProviderButtons pending={submit.pending} onProvider={props.onProvider} />
        <div className={css('divider')}>{t('web.auth.orEmail')}</div>
        <form className={css('form')} noValidate onSubmit={send}>
          {props.children}
          {submit.message === null ? null : (
            <div className={css('error')} role="alert" data-testid="auth-error">
              {t(submit.message)}
            </div>
          )}
          <Button type="submit" variant="primary" size={44} block isDisabled={submit.pending}>
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
