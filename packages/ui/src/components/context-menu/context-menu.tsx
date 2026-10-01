import { type ReactElement, type ReactNode, type RefObject, useMemo, useRef } from 'react';
import {
  type Key,
  Menu,
  MenuItem,
  MenuSection,
  MenuTrigger,
  Popover,
  Pressable,
  Separator,
  SubmenuTrigger,
} from 'react-aria-components';

import { Icon } from '../../icons/icon';
import { moduleClasses } from '../module-classes';

import styles from './context-menu.module.css';
import {
  type MenuActionEntry,
  type MenuAnchor,
  type MenuEntry,
  groupEntries,
  toggledEntry,
} from './menu-model';

const css = moduleClasses(styles);

interface EntryItemProps {
  readonly entry: MenuActionEntry;
  readonly checkable?: boolean;
}

function EntryItem({ entry, checkable = false }: EntryItemProps): ReactElement {
  return (
    <MenuItem
      id={entry.id}
      textValue={entry.label}
      className={css('item')}
      data-danger={entry.danger === true ? 'true' : undefined}
      isDisabled={entry.disabled === true}
      shouldCloseOnSelect={entry.keepOpen !== true}
      // En una seccion de casillas la accion la lanza onSelectionChange, no el elemento.
      {...(checkable || entry.onAction === undefined ? {} : { onAction: entry.onAction })}
    >
      {({ isSelected, hasSubmenu }) => (
        <>
          <span className={css('check')}>
            {checkable && isSelected ? <Icon name="check" size={14} /> : null}
          </span>
          {entry.icon === undefined ? null : <Icon name={entry.icon} size={15} />}
          <span className={css('label')}>{entry.label}</span>
          {entry.hint === undefined ? null : <kbd className={css('hint')}>{entry.hint}</kbd>}
          {hasSubmenu ? <Icon name="chevron-right" size={14} className={css('more')} /> : null}
        </>
      )}
    </MenuItem>
  );
}

function CheckSection({ entries }: { readonly entries: readonly MenuActionEntry[] }): ReactElement {
  const selected = entries.filter((entry) => entry.checked === true).map((entry) => entry.id);
  const onSelectionChange = (keys: 'all' | Set<Key>): void => {
    if (keys === 'all') return;
    toggledEntry(entries, keys)?.onAction?.();
  };
  return (
    <MenuSection
      selectionMode="multiple"
      selectedKeys={selected}
      onSelectionChange={onSelectionChange}
    >
      {entries.map((entry) => (
        <EntryItem key={entry.id} entry={entry} checkable />
      ))}
    </MenuSection>
  );
}

function Submenu({ entry }: { readonly entry: MenuActionEntry }): ReactElement {
  return (
    <SubmenuTrigger>
      <EntryItem entry={entry} />
      <Popover className={css('popover')} offset={-4}>
        <Menu className={css('menu')} aria-label={entry.label}>
          {renderEntries(entry.children ?? [])}
        </Menu>
      </Popover>
    </SubmenuTrigger>
  );
}

function renderEntries(entries: readonly MenuEntry[]): ReactNode[] {
  return groupEntries(entries).map((group) => {
    switch (group.kind) {
      case 'separator':
        return <Separator key={group.id} className={css('separator')} />;
      case 'checks':
        return <CheckSection key={group.id} entries={group.entries} />;
      case 'item':
        return group.entry.children === undefined ? (
          <EntryItem key={group.entry.id} entry={group.entry} />
        ) : (
          <Submenu key={group.entry.id} entry={group.entry} />
        );
    }
  });
}

export interface ContextMenuProps {
  readonly entries: readonly MenuEntry[];
  /** null: cerrado. Lo gestiona useContextMenu. */
  readonly anchor: MenuAnchor | null;
  readonly onClose: () => void;
  /** Nombre accesible del menu. */
  readonly label: string;
}

/**
 * Menu contextual de toda la app (inventario 2.4): submenus, casillas que dejan
 * el menu abierto, Esc para cerrar, y se abre con clic derecho o con el boton ⋯.
 * React Aria lo coloca dentro de la ventana y gestiona el teclado.
 *
 * El ancla del puntero es un span fijo e invisible: React Aria necesita un
 * elemento real que medir para colocar el menu. Lleva dentro el nombre del menu
 * porque React Aria nombra el menu con su disparador (aria-labelledby).
 */
interface AnchorTarget {
  readonly pointRef: RefObject<HTMLSpanElement | null>;
  readonly triggerRef: RefObject<Element | null>;
  readonly point: { readonly x: number; readonly y: number };
  /** Separacion con el ancla: 4 px bajo un boton, ninguna bajo el puntero. */
  readonly offset: number;
}

function useAnchorTarget(anchor: MenuAnchor | null): AnchorTarget {
  const pointRef = useRef<HTMLSpanElement>(null);
  const element = anchor?.kind === 'element' ? anchor.element : null;
  const elementRef = useMemo(() => ({ current: element }), [element]);
  if (anchor?.kind === 'point') {
    return { pointRef, triggerRef: pointRef, point: anchor, offset: 0 };
  }
  return {
    pointRef,
    triggerRef: element === null ? pointRef : elementRef,
    point: ORIGIN,
    offset: 4,
  };
}

const ORIGIN = { x: 0, y: 0 };

export function ContextMenu({ entries, anchor, onClose, label }: ContextMenuProps): ReactElement {
  const { pointRef, triggerRef, point, offset } = useAnchorTarget(anchor);
  return (
    <MenuTrigger
      isOpen={anchor !== null}
      onOpenChange={(isOpen) => {
        if (!isOpen) onClose();
      }}
    >
      <Pressable isDisabled>
        <span
          ref={pointRef}
          aria-hidden="true"
          className={css('anchor')}
          style={{ left: point.x, top: point.y }}
        >
          {label}
        </span>
      </Pressable>
      <Popover
        className={css('popover')}
        triggerRef={triggerRef}
        placement="bottom start"
        offset={offset}
      >
        <Menu className={css('menu')} aria-label={label}>
          {renderEntries(entries)}
        </Menu>
      </Popover>
    </MenuTrigger>
  );
}
