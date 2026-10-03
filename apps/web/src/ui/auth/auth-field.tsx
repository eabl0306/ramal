import { TextField, type TextFieldProps } from '@git-rudder/ui';
import type { ReactElement } from 'react';

/**
 * Campo de la tarjeta de entrar y crear cuenta. El prototipo web los dibuja de
 * 42 px con esquinas de 10, mas grandes que los de modales y ajustes; el tamano
 * vive aqui para que ningun campo de la tarjeta se quede con el de 36.
 */
export function AuthField(props: Omit<TextFieldProps, 'size'>): ReactElement {
  return <TextField {...props} size={42} />;
}
