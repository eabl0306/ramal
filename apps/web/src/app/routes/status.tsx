import type { ReactElement } from 'react';
import { useLoaderData, useRevalidator } from 'react-router';

import { type HealthProbe, checkApiHealth } from '../../application/use-cases/check-api-health';
import { LoadingScreen } from '../../ui/feedback/feedback-screens';
import { StatusScreen } from '../../ui/status/status-screen';
import { container } from '../container';

/** Ruta SPA: pregunta a la API en el navegador, por el mismo origen (architecture.md 6.3). */
export function clientLoader(): Promise<HealthProbe> {
  return checkApiHealth(container.health);
}

export function HydrateFallback(): ReactElement {
  return <LoadingScreen />;
}

export default function StatusRoute(): ReactElement {
  const probe = useLoaderData<typeof clientLoader>();
  const revalidator = useRevalidator();
  return (
    <StatusScreen
      probe={probe}
      retrying={revalidator.state === 'loading'}
      onRetry={() => {
        void revalidator.revalidate();
      }}
    />
  );
}
