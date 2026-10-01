import type { ReactElement } from 'react';

import { ProblemScreen } from '../../ui/feedback/feedback-screens';

/** Cualquier ruta desconocida, con la barra publica encima. */
export default function NotFoundRoute(): ReactElement {
  return <ProblemScreen notFound />;
}
