import type { ReactElement } from 'react';
import {
  TextField as AriaTextField,
  FieldError,
  Input,
  Label,
  Text,
  TextArea,
} from 'react-aria-components';

import { moduleClasses } from '../module-classes';

import styles from './text-field.module.css';

const css = moduleClasses(styles);

export interface TextFieldProps {
  readonly label: string;
  readonly value: string;
  readonly onChange: (value: string) => void;
  readonly placeholder?: string;
  /** Ayuda bajo el campo. */
  readonly description?: string;
  /**
   * Error en linea (gaps C-03: los modales del prototipo fallaban en silencio).
   * Con texto, el campo se marca como invalido.
   */
  readonly errorMessage?: string;
  /** `password` para secretos: nunca se ensena un token en claro (gaps B-37). */
  readonly type?: 'text' | 'email' | 'password' | 'url' | 'search';
  /** Monoespaciada, para rutas, URLs, ramas y shas. */
  readonly mono?: boolean;
  /** Varias lineas, con este numero de filas visibles. */
  readonly rows?: number;
  readonly name?: string;
  readonly autoComplete?: string;
  readonly isRequired?: boolean;
  readonly isDisabled?: boolean;
  readonly autoFocus?: boolean;
}

export function TextField({
  label,
  description,
  errorMessage,
  type = 'text',
  mono = false,
  rows,
  placeholder,
  ...field
}: TextFieldProps): ReactElement {
  const hint = placeholder === undefined ? {} : { placeholder };
  const invalid = errorMessage !== undefined && errorMessage !== '';
  return (
    <AriaTextField
      {...field}
      className={css('field')}
      type={type}
      isInvalid={invalid}
      data-mono={mono ? 'true' : undefined}
    >
      <Label className={css('label')}>{label}</Label>
      {rows === undefined ? (
        <Input className={css('input')} {...hint} />
      ) : (
        <TextArea className={css('input')} rows={rows} {...hint} />
      )}
      {description === undefined ? null : (
        <Text slot="description" className={css('description')}>
          {description}
        </Text>
      )}
      <FieldError className={css('error')}>{errorMessage}</FieldError>
    </AriaTextField>
  );
}
