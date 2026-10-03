import { $, expect } from '@wdio/globals';

import { FIXTURE_COMMITS } from '../support/fixture-repo';

/**
 * El camino del primer corte con todo real: el comando repo_pick_and_open abre la
 * carpeta en el motor, el comando rpc trae graph/window con git de verdad y
 * @git-rudder/repo-view lo pinta en WebKitGTK con la CSP de produccion.
 *
 * El idioma del arranque depende del sistema, asi que lo primero es fijarlo con
 * el conmutador de la barra de titulo.
 */
describe('Git Rudder Desktop con el motor real', () => {
  it('abre el repositorio elegido y pinta su grafo', async () => {
    await $('button=ES').click();
    await $('button=Abrir repositorio local').click();

    await $('[role="grid"]').waitForDisplayed();
    for (const summary of FIXTURE_COMMITS) {
      // La celda del mensaje lleva el asunto entero en su title.
      await expect($(`[role="gridcell"][title="${summary}"]`)).toBeDisplayed();
    }
    // La pestana se llama como la carpeta del repositorio de prueba.
    await expect($('button=repo')).toHaveAttribute('aria-current', 'page');
  });

  it('cambia el tema del documento', async () => {
    await $('button[aria-label="Claro"]').click();
    await expect($('html')).toHaveAttribute('data-theme', 'light');

    await $('button[aria-label="Oscuro"]').click();
    await expect($('html')).toHaveAttribute('data-theme', 'dark');
  });

  it('cambia el idioma de la ventana', async () => {
    await $('button=EN').click();

    await expect($('html')).toHaveAttribute('lang', 'en');
    await expect($('button=Home')).toBeDisplayed();
  });
});
