import type { ReactElement, ReactNode } from 'react';
import { Modal as AriaModal, Dialog, Heading, ModalOverlay } from 'react-aria-components';

import { moduleClasses } from '../module-classes';

import styles from './modal.module.css';

const css = moduleClasses(styles);

export interface ModalProps {
  readonly isOpen: boolean;
  readonly onOpenChange: (isOpen: boolean) => void;
  readonly title: string;
  readonly subtitle?: string;
  readonly children: ReactNode;
  /** Botones de la derecha del pie (Cancelar y la accion). */
  readonly footer?: ReactNode;
  /** 460 px para confirmaciones y formularios cortos, 580 para los largos (crear PR). */
  readonly size?: 'sm' | 'md';
  /**
   * Si un clic fuera lo cierra. Esc lo cierra siempre. Un formulario a medio
   * rellenar puede pedir que no, para no perder lo escrito por un clic perdido.
   */
  readonly isDismissable?: boolean;
  /** Para dialogos de peligro: los lectores de pantalla los anuncian como alerta. */
  readonly role?: 'dialog' | 'alertdialog';
}

/**
 * Dialogo modal sobre React Aria: atrapa el foco, lo devuelve al cerrar, cierra
 * con Esc y bloquea el scroll con estilos en linea (CSSOM), sin inyectar
 * etiquetas <style> que la CSP bloquearia (ADR 0011).
 */
export function Modal({
  isOpen,
  onOpenChange,
  title,
  subtitle,
  children,
  footer,
  size = 'sm',
  isDismissable = true,
  role = 'dialog',
}: ModalProps): ReactElement {
  return (
    <ModalOverlay
      className={css('overlay')}
      isOpen={isOpen}
      onOpenChange={onOpenChange}
      isDismissable={isDismissable}
    >
      <AriaModal className={css('modal')} data-size={size}>
        <Dialog className={css('dialog')} role={role}>
          <header className={css('header')}>
            <Heading slot="title" className={css('title')}>
              {title}
            </Heading>
            {subtitle === undefined ? null : <p className={css('subtitle')}>{subtitle}</p>}
          </header>
          <div className={css('body')}>{children}</div>
          {footer === undefined ? null : <footer className={css('footer')}>{footer}</footer>}
        </Dialog>
      </AriaModal>
    </ModalOverlay>
  );
}
