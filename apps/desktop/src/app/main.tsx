import '@git-rudder/ui/themes/git-rudder.css';
import '@git-rudder/ui/themes/fonts';

import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import { applyAppearance } from '../ui/appearance/appearance-context';

import { App } from './app';
import { browserStorage, createContainer } from './composition-root';

const element = document.getElementById('root');
if (element === null) {
  throw new Error('No existe el elemento #root en index.html');
}

const container = createContainer({
  storage: browserStorage(window),
  systemLanguage: navigator.language,
});
const root = document.documentElement;
// Antes del primer pintado, para que la ventana no aparezca un instante con el
// tema o el idioma equivocados.
applyAppearance(root, container.initialAppearance);

createRoot(element).render(
  <StrictMode>
    <App container={container} root={root} />
  </StrictMode>,
);
