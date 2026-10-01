// API publica de @ramal/domain. Solo se exporta lo que usan las apps; cada
// modulo vive en su carpeta y se prueba por separado.

export {
  type AuthFieldError,
  type AuthValidation,
  type LoginForm,
  MIN_PASSWORD_LENGTH,
  type PasswordHint,
  type SignupForm,
  isValidEmail,
  passwordHint,
  validateLogin,
  validateSignup,
} from './auth/auth-form';

export {
  type CpuArch,
  type DesktopOs,
  type DetectedOs,
  type DetectedPlatform,
  type PlatformHints,
  detectOs,
  detectPlatform,
  isDesktopOs,
} from './downloads/detect-platform';
export {
  type DownloadArtifact,
  type DownloadRecommendation,
  highlightedDownloadOs,
  recommendDownload,
} from './downloads/recommend-download';

export {
  type ConflictFile,
  type ConflictLabels,
  type ConflictSegment,
  type ConflictSide,
  type ContextSegment,
  type FileResolution,
  type HunkChoice,
  type HunkDecision,
  type HunkSegment,
  hunksOf,
  initResolution,
} from './conflict/model';
export { clearHunk, resolveEmpty, takeAll, toggleBlock, toggleLine } from './conflict/choices';
export {
  type ConflictOutput,
  type OutputLine,
  buildOutput,
  hunkMarkerText,
} from './conflict/build-output';
export {
  type MarkerKind,
  type MarkerLine,
  type ParseConflictResult,
  findConflictMarkers,
  hasConflictMarkers,
  parseConflictMarkers,
} from './conflict/markers';
export {
  type MergeProgress,
  hasManualEdit,
  isFileResolved,
  isHunkResolved,
  mergeProgress,
} from './conflict/resolved';
export {
  acceptMarkersAnyway,
  cancelManualEdit,
  isEditingManually,
  resetManualEdit,
  saveManualEdit,
  startManualEdit,
  updateManualDraft,
} from './conflict/manual-edit';
export {
  type ConflictOperation,
  type ConflictSideRole,
  type ConflictSideRoles,
  conflictSideRoles,
} from './conflict/side-roles';
export {
  type ConflictSession,
  conflictSessionProgress,
  createConflictSession,
  selectConflictFile,
  updateConflictResolution,
} from './conflict/session';

export {
  REBASE_ACTIONS,
  type RebaseAction,
  type RebaseCommit,
  type RebaseItem,
  type RebasePlan,
  type RebaseSummary,
  createRebasePlan,
  moveRebaseItem,
  setRebaseAction,
  setRebaseMessage,
  summarizeRebasePlan,
} from './rebase/plan';
export {
  type RebaseContext,
  type RebaseIssue,
  canStartRebase,
  validateRebasePlan,
} from './rebase/validate';
export { rebaseTodo } from './rebase/todo';

export {
  PLATFORMS,
  PLATFORM_IDS,
  type PlatformId,
  type PlatformInfo,
  prNumberLabel,
} from './platforms/platforms';

export {
  type AccountSummary,
  EMPTY_REPO_FILTERS,
  type RepoContext,
  type RepoFilters,
  type RepoKindFilter,
  type RepoSummary,
  type WorkspaceFilter,
  type WorkspaceSummary,
  hasActiveRepoFilters,
  repoAccount,
  repoWorkspaces,
} from './repos/model';
export {
  countReposInWorkspace,
  countReposUsingAccount,
  filterRepos,
  repoMatchesFilters,
} from './repos/filter';
export { type RepoGroup, type RepoGroupKey, type RepoGrouping, groupRepos } from './repos/group';

export {
  OPEN_PR_STATES,
  PR_STATES,
  type PlatformPrStatus,
  type PrState,
  type PrStateFilter,
  type PullRequestSummary,
  type ReviewDecision,
  type WorkspacePrSummary,
  countOpenPrsInRepo,
  countPrsByState,
  derivePrState,
  filterPrsByState,
  isOpenPr,
  sortPrsByUpdate,
  summarizeWorkspacePrs,
} from './prs/aggregate';

export {
  type RelativeTimeBucket,
  type RelativeTimeUnit,
  relativeTimeBucket,
} from './time/relative-time';

export { MIN_MASKABLE_LENGTH, SECRET_MASK, maskSecret } from './secrets/mask-secret';

export {
  type CommitMessageParts,
  type CommitReadiness,
  SUMMARY_LIMIT,
  type SummaryCounter,
  canCommit,
  composeCommitMessage,
  countGraphemes,
  splitCommitMessage,
  summaryCounter,
} from './commits/message';
export {
  CONVENTIONAL_TYPES,
  type ConventionalCheck,
  type ConventionalSummary,
  checkConventional,
  formatConventional,
  parseConventional,
} from './commits/conventional';
export { cherryPickMessage, mergeMessage, revertMessage } from './commits/templates';

export {
  type PromptIssue,
  type PromptValidation,
  type RefNameIssue,
  isValidRefName,
  refNameIssues,
  suggestBranchName,
  validatePromptValue,
} from './refs/ref-name';
