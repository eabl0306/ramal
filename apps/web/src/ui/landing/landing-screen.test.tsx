import type { DetectedPlatform } from '@ramal/domain';
import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import type { ReleaseLoad } from '../../application/use-cases/load-release';
import { createStaticReleaseCatalog } from '../../infrastructure/releases/static-release-catalog';
import { fixedVisitor, recordingClipboard, renderScreen, testPorts } from '../../test/render';

import { LandingScreen } from './landing-screen';

const RELEASE: ReleaseLoad = {
  outcome: 'loaded',
  release: await createStaticReleaseCatalog().latest(),
};

function renderLanding(platform: DetectedPlatform, language: 'es' | 'en' = 'es') {
  const clipboard = recordingClipboard();
  const ports = testPorts({ visitor: fixedVisitor(platform), clipboard });
  const view = renderScreen(<LandingScreen language={language} release={RELEASE} />, {
    language,
    ports,
  });
  return { ...view, ports, clipboard };
}

describe('LandingScreen', () => {
  it('ofrece el instalador del sistema detectado y destaca su tarjeta', () => {
    renderLanding({ os: 'windows', arch: 'x64' });

    expect(screen.getByRole('button', { name: 'Descargar para Windows' })).toBeInTheDocument();
    const card = screen.getByTestId('download-card-windows');
    expect(card).toHaveAttribute('data-highlighted', 'true');
    expect(within(card).getByText('Tu sistema')).toBeInTheDocument();
    expect(screen.getByTestId('download-card-mac')).toHaveAttribute('data-highlighted', 'false');
  });

  it('en un movil no destaca ninguna tarjeta y avisa de que Ramal es de escritorio', () => {
    renderLanding({ os: 'ios', arch: 'unknown' });

    expect(screen.queryByText('Tu sistema')).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /Descargar para/u })).not.toBeInTheDocument();
    expect(screen.getByRole('note')).toHaveTextContent('Ramal es una app de escritorio');
  });

  it('avisa del fichero que se descarga y copia los comandos de instalacion', async () => {
    const user = userEvent.setup();
    const { clipboard } = renderLanding({ os: 'linux', arch: 'x64' });

    await user.click(screen.getByRole('button', { name: 'Descargar para Linux' }));
    expect(await screen.findByText('Descarga iniciada: ramal_2.4.0_amd64.deb')).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: /flatpak install flathub/u }));
    expect(clipboard.copied).toEqual(['flatpak install flathub io.ramal.Ramal']);
    expect(await screen.findByText(/Copiado: flatpak/u)).toBeInTheDocument();
  });

  it('ensena tamanos de las alternativas y enlaza las tiendas', async () => {
    const user = userEvent.setup();
    renderLanding({ os: 'mac', arch: 'unknown' });

    const card = screen.getByTestId('download-card-mac');
    await user.click(within(card).getByRole('button', { name: /Intel \(\.dmg\)/u }));
    expect(within(card).getByText('118 MB')).toBeInTheDocument();
    expect(await screen.findByText('Descarga iniciada: Ramal-2.4.0-x64.dmg')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /^Flathub/u })).toHaveAttribute(
      'href',
      'https://flathub.org/apps/io.ramal.Ramal',
    );
  });

  it('ensena los requisitos reales de Linux y avisa de los formatos que no se actualizan', () => {
    renderLanding({ os: 'linux', arch: 'x64' });

    const card = screen.getByTestId('download-card-linux');
    // Tauri 2 necesita webkit2gtk-4.1, que no existe en Ubuntu 20.04 ni Debian 11 (gaps X-09).
    expect(card).toHaveTextContent('Ubuntu 22.04+, Debian 12+, Fedora 38+');
    expect(card).not.toHaveTextContent('20.04');
    // El aviso va en el requisito: en el boton principal el texto no cabria.
    expect(card).toHaveTextContent('Los .deb y .rpm no se actualizan solos');
    expect(within(card).getByRole('link', { name: /se actualiza con Flatpak/u })).toBeVisible();
  });

  it('la tarjeta de VS Code pide la version del motor y ofrece un .vsix por plataforma', () => {
    renderLanding({ os: 'mac', arch: 'arm64' });

    const card = screen.getByTestId('download-card-vscode');
    expect(card).toHaveTextContent('VS Code 1.138 o posterior. No disponible en vscode.dev');
    expect(within(card).getAllByRole('button', { name: /\(\.vsix\)$/u })).toHaveLength(8);
    expect(within(card).getByRole('button', { name: 'Alpine ARM64 (.vsix)' })).toBeVisible();
    expect(within(card).queryByText(/MB$/u)).not.toBeInTheDocument();
  });

  it('se pinta en ingles y recuerda el idioma de la pagina', () => {
    const { ports } = renderLanding({ os: 'unknown', arch: 'unknown' }, 'en');

    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('A Git client');
    expect(ports.preferences.read().language).toBe('en');
  });

  it('sin datos de descarga la portada se pinta igual', () => {
    renderScreen(<LandingScreen language="es" release={{ outcome: 'unavailable' }} />);

    expect(screen.getByRole('heading', { level: 1 })).toBeInTheDocument();
    expect(screen.queryByTestId('download-card-mac')).not.toBeInTheDocument();
  });
});
