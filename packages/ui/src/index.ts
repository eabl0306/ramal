// API publica de @ramal/ui. Los juegos de iconos y los temas van en subrutas
// propias (icons/lucide, icons/codicons, themes/*.css, themes/fonts) para que
// cada host solo cargue lo suyo.

export {
  COLOR_TOKENS,
  LANE_COUNT,
  SHAPE_TOKENS,
  SHARED_TOKENS,
  type ThemeToken,
  cssVar,
  laneColor,
} from './tokens/tokens';
export {
  DENSITIES,
  DENSITY_NAMES,
  type Density,
  type DensityMetrics,
  type GraphGeometry,
} from './tokens/densities';

export { I18nProvider, type I18nProviderProps } from './i18n/i18n-provider';
export { useLocale, useT } from './i18n/i18n-context';

export { ICON_NAMES, type IconName } from './icons/icon-names';
export type { IconGlyphProps, IconSet } from './icons/icon-set';
export { Icon, IconProvider, type IconProps, type IconProviderProps } from './icons/icon';

export {
  Button,
  type ButtonProps,
  type ButtonSize,
  type ButtonVariant,
} from './components/button/button';
export {
  IconButton,
  type IconButtonProps,
  type IconButtonSize,
  type IconButtonVariant,
} from './components/icon-button/icon-button';
export {
  Badge,
  type BadgePlatform,
  type BadgeProps,
  type FileStatus,
  FileStatusBadge,
  type FileStatusBadgeProps,
  PlatformBadge,
  type PlatformBadgeProps,
  type PlatformBadgeSize,
  type PrBadgeState,
  PrStateBadge,
  type Tone,
} from './components/badge/badge';
export { Chip, type ChipProps } from './components/chip/chip';
export { Toggle, type ToggleProps } from './components/toggle/toggle';
export {
  type SegmentOption,
  SegmentedControl,
  type SegmentedControlProps,
  type SegmentedVariant,
} from './components/segmented-control/segmented-control';
export { TextField, type TextFieldProps } from './components/text-field/text-field';
export { Modal, type ModalProps } from './components/modal/modal';
export { ContextMenu, type ContextMenuProps } from './components/context-menu/context-menu';
export type {
  MenuActionEntry,
  MenuAnchor,
  MenuEntry,
  MenuSeparatorEntry,
} from './components/context-menu/menu-model';
export {
  type ContextMenuController,
  useContextMenu,
} from './components/context-menu/use-context-menu';
export {
  TOAST_DURATION,
  ToastProvider,
  type ToastProviderProps,
} from './components/toast/toast-provider';
export {
  type ToastApi,
  type ToastKind,
  type ToastMessage,
  useToast,
} from './components/toast/toast-context';
