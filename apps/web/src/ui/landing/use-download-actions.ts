import { useT, useToast } from '@ramal/ui';

import type { DownloadFile } from '../../domain/downloads/release';
import { useUiPorts } from '../ports/ui-ports';

export interface DownloadActions {
  readonly download: (file: DownloadFile) => void;
  readonly copy: (command: string) => void;
}

/**
 * Lo que hacen los botones de descarga y las cajas de comandos. Con los datos
 * fijos del primer corte no hay URL publicada que abrir: el boton avisa de que
 * fichero tocaria, y la descarga real llega con el manifiesto de la API.
 */
export function useDownloadActions(): DownloadActions {
  const t = useT();
  const toast = useToast();
  const { clipboard } = useUiPorts();
  return {
    download: (file) => {
      toast.show(t('web.download.started', { file: file.file }));
    },
    copy: (command) => {
      void clipboard.copy(command).then((copied) => {
        if (copied) toast.show(t('web.common.copied', { text: command }));
      });
    },
  };
}
