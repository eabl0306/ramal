import type { ReactElement } from 'react';
import { Outlet } from 'react-router';

import { SiteHeader } from '../../ui/shell/site-header';

/** La barra publica sobre la landing, la autenticacion y el estado. El panel tendra la suya. */
export default function PublicLayout(): ReactElement {
  return (
    <>
      <SiteHeader />
      <Outlet />
    </>
  );
}
