import '@testing-library/jest-dom/vitest';

import { cleanup } from '@testing-library/react';
import { afterEach } from 'vitest';

// Vitest corre sin globals, asi que el limpiado automatico de Testing Library se
// declara aqui.
afterEach(() => {
  cleanup();
});
