import { describe, expect, it } from 'vitest';

import { DELETED_FILE, I18N_FILE, LABELS, SESSION_FILE } from '../test/conflict-fixtures';

import { takeAll, toggleLine } from './choices';
import {
  acceptMarkersAnyway,
  cancelManualEdit,
  isEditingManually,
  resetManualEdit,
  saveManualEdit,
  startManualEdit,
  updateManualDraft,
} from './manual-edit';
import { initResolution } from './model';
import { hasManualEdit, isFileResolved, mergeProgress } from './resolved';
import {
  conflictSessionProgress,
  createConflictSession,
  selectConflictFile,
  updateConflictResolution,
} from './session';
import { conflictSideRoles } from './side-roles';

describe('regla de resuelto', () => {
  it('un bloque elegido no basta si queda otro sin decidir', () => {
    const one = takeAll(SESSION_FILE, initResolution(SESSION_FILE), 'theirs');
    expect(isFileResolved(one)).toBe(true);
    const partial = toggleLine(initResolution(SESSION_FILE), 0, 'ours', 0);
    expect(isFileResolved(partial)).toBe(false);
  });

  it('un fichero sin bloques esta resuelto', () => {
    expect(isFileResolved({ hunks: [], manual: null, draft: null, acceptMarkers: false })).toBe(
      true,
    );
  });

  it('con texto manual mira los marcadores y acepta marcarlo igualmente', () => {
    const manual = { ...initResolution(I18N_FILE), manual: 'a\n<<<<<<< x\nb' };
    expect(isFileResolved(manual)).toBe(false);
    expect(isFileResolved(acceptMarkersAnyway(manual))).toBe(true);
    expect(isFileResolved({ ...manual, manual: 'a = "=======";' })).toBe(true);
  });

  it('cuenta el progreso de varios ficheros', () => {
    const done = takeAll(DELETED_FILE, initResolution(DELETED_FILE), 'ours');
    expect(mergeProgress([done, initResolution(I18N_FILE)])).toEqual({
      resolved: 1,
      total: 2,
      complete: false,
    });
    expect(mergeProgress([done]).complete).toBe(true);
  });
});

describe('edicion manual', () => {
  it('parte del resultado actual con marcadores y guarda el borrador', () => {
    const editing = startManualEdit(I18N_FILE, initResolution(I18N_FILE), LABELS);
    expect(isEditingManually(editing)).toBe(true);
    expect(editing.draft).toContain('<<<<<<< feature/graph-ui');
    expect(hasManualEdit(editing)).toBe(true);
    const saved = saveManualEdit(updateManualDraft(editing, 'final'));
    expect(saved).toMatchObject({ manual: 'final', draft: null });
    expect(isFileResolved(saved)).toBe(true);
  });

  it('retoma un borrador existente y parte del texto guardado si lo hay', () => {
    const drafting = updateManualDraft(initResolution(I18N_FILE), 'a medias');
    expect(startManualEdit(I18N_FILE, drafting, LABELS)).toBe(drafting);
    const saved = { ...initResolution(I18N_FILE), manual: 'guardado' };
    expect(startManualEdit(I18N_FILE, saved, LABELS).draft).toBe('guardado');
  });

  it('cancelar descarta el borrador y restablecer vuelve a las elecciones', () => {
    const saved = { ...initResolution(I18N_FILE), manual: 'guardado', draft: 'otro' };
    expect(cancelManualEdit(saved)).toMatchObject({ manual: 'guardado', draft: null });
    expect(resetManualEdit(saved)).toMatchObject({ manual: null, draft: null });
    expect(saveManualEdit(cancelManualEdit(saved))).toMatchObject({ manual: 'guardado' });
  });

  it('marcar como resuelto igualmente no hace nada sin texto manual', () => {
    const initial = initResolution(I18N_FILE);
    expect(acceptMarkersAnyway(initial)).toBe(initial);
  });
});

describe('sesion del resolvedor', () => {
  it('cambia de fichero sin perder el borrador del anterior', () => {
    let session = createConflictSession('merge', [SESSION_FILE, I18N_FILE]);
    session = updateConflictResolution(session, 0, (resolution) =>
      updateManualDraft(resolution, 'borrador'),
    );
    session = selectConflictFile(session, 1);
    expect(session.activeFile).toBe(1);
    expect(session.resolutions[0]?.draft).toBe('borrador');
    expect(selectConflictFile(session, 7)).toBe(session);
    expect(updateConflictResolution(session, 9, (resolution) => resolution)).toBe(session);
  });

  it('solo se completa con todos los ficheros resueltos', () => {
    let session = createConflictSession('merge', [DELETED_FILE]);
    expect(conflictSessionProgress(session).complete).toBe(false);
    session = updateConflictResolution(session, 0, (resolution) =>
      takeAll(DELETED_FILE, resolution, 'theirs'),
    );
    expect(conflictSessionProgress(session)).toEqual({ resolved: 1, total: 1, complete: true });
  });
});

describe('papeles de cada lado', () => {
  it('en un merge A es la rama actual y B la entrante', () => {
    expect(conflictSideRoles('merge')).toEqual({
      ours: 'current-branch',
      theirs: 'incoming-branch',
    });
  });

  it('en un rebase A es la base y B el commit que se reaplica', () => {
    expect(conflictSideRoles('rebase')).toEqual({
      ours: 'rebase-base',
      theirs: 'replayed-commit',
    });
    expect(conflictSideRoles('stash-apply').theirs).toBe('stash');
  });
});
