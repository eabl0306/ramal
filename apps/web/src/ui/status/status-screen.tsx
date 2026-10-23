import { type PlainKey, formatDate } from '@git-rudder/i18n';
import { Button, useLocale, useT } from '@git-rudder/ui';
import type { ReactElement } from 'react';

import type { HealthProbe } from '../../application/use-cases/check-api-health';
import type { HealthCheck, HealthReport } from '../../domain/health/health-report';
import { PRODUCT_NAME, moduleClasses } from '../shared/module-classes';

import styles from './status.module.css';

const css = moduleClasses(styles);

type ServiceState = 'ok' | 'degraded' | 'unreachable';

const STATE_LABELS: Readonly<Record<ServiceState, PlainKey>> = {
  ok: 'web.status.ok',
  degraded: 'web.status.degraded',
  unreachable: 'web.status.unreachable',
};

/** Los componentes que la web sabe nombrar; uno nuevo se ensena con su nombre tecnico. */
const COMPONENT_LABELS: Readonly<Record<string, PlainKey>> = {
  database: 'web.status.component.database',
};

function CheckRow({ check }: { readonly check: HealthCheck }): ReactElement {
  const t = useT();
  const label = COMPONENT_LABELS[check.name];
  return (
    <li className={css('check')} data-testid="health-check" data-status={check.status}>
      <span>{label === undefined ? check.name : t(label)}</span>
      <span className={css('check-state')}>
        {t(check.status === 'ok' ? 'web.status.checkOk' : 'web.status.checkError')}
      </span>
      <span className={css('latency')}>{t('web.status.latency', { ms: check.latencyMs })}</span>
    </li>
  );
}

function ReportDetails({ report }: { readonly report: HealthReport }): ReactElement {
  const t = useT();
  const locale = useLocale();
  const time = formatDate(report.checkedAt, locale, { style: 'dateTime' });
  return (
    <>
      {report.checks.length === 0 ? null : (
        <>
          <h2 className={css('section-title')}>{t('web.status.checks')}</h2>
          <ul className={css('checks')}>
            {report.checks.map((check) => (
              <CheckRow key={check.name} check={check} />
            ))}
          </ul>
        </>
      )}
      <p className={css('meta')}>
        {t('web.status.version', { version: report.version })}
        {' · '}
        {t('web.status.checkedAt', { time })}
      </p>
    </>
  );
}

export interface StatusScreenProps {
  readonly probe: HealthProbe;
  readonly onRetry: () => void;
  readonly retrying: boolean;
}

/**
 * Lo que responde /health/ready a traves del gateway: operativo, degradado (503
 * con su informe) o sin conexion. Es la pantalla que comprueban los e2e de la
 * pila y la que demuestra que la web llega a la API por el mismo origen.
 */
export function StatusScreen({ probe, onRetry, retrying }: StatusScreenProps): ReactElement {
  const t = useT();
  const state: ServiceState = probe.outcome === 'unreachable' ? 'unreachable' : probe.report.status;
  return (
    <main className={css('page')}>
      <title>{`${t('web.status.title')} · ${PRODUCT_NAME}`}</title>
      <section className={css('card')}>
        <div>
          <h1 className={css('title')}>{t('web.status.title')}</h1>
          <p className={css('subtitle')}>{t('web.status.subtitle')}</p>
        </div>
        <div className={css('summary')} data-testid="health-status" data-status={state}>
          <span className={css('dot')} aria-hidden="true" />
          <span>{t(STATE_LABELS[state])}</span>
        </div>
        {probe.outcome === 'unreachable' ? (
          <p className={css('note')}>{t('web.status.unreachableText')}</p>
        ) : (
          <ReportDetails report={probe.report} />
        )}
        <div>
          <Button icon="sync" isDisabled={retrying} onPress={onRetry}>
            {t('web.status.retry')}
          </Button>
        </div>
      </section>
    </main>
  );
}
