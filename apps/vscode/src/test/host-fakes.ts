import type { Dialogs, PickItem } from '../extension/application/ports/dialogs';
import type { Logger } from '../extension/application/ports/logger';
import type { Notifications } from '../extension/application/ports/notifications';
import type { CancelTimer, Timer } from '../extension/application/ports/timer';
import type {
  PanelChannel,
  ShowSurfaceRequest,
  WebviewSurface,
} from '../extension/application/ports/webview-surface';
import type { HostToWebviewMessage, Surface } from '../shared/messages';

/** Log que guarda cada mensaje por nivel. */
export class RecordingLogger implements Logger {
  readonly lines: { readonly level: string; readonly message: string }[] = [];

  debug(message: string): void {
    this.lines.push({ level: 'debug', message });
  }

  info(message: string): void {
    this.lines.push({ level: 'info', message });
  }

  warn(message: string): void {
    this.lines.push({ level: 'warn', message });
  }

  error(message: string): void {
    this.lines.push({ level: 'error', message });
  }

  messages(level: string): string[] {
    return this.lines.filter((line) => line.level === level).map((line) => line.message);
  }
}

interface Task {
  readonly at: number;
  readonly callback: () => void;
  cancelled: boolean;
}

/** Reloj falso: el tiempo solo avanza cuando el test lo dice. */
export class FakeTimer implements Timer {
  current = 0;
  #tasks: Task[] = [];

  now(): number {
    return this.current;
  }

  setTimeout(callback: () => void, ms: number): CancelTimer {
    const task: Task = { at: this.current + ms, callback, cancelled: false };
    this.#tasks.push(task);
    return () => {
      task.cancelled = true;
    };
  }

  /** Avanza el reloj y ejecuta lo que venza. */
  advance(ms: number): void {
    this.current += ms;
    const due = this.#tasks.filter((task) => !task.cancelled && task.at <= this.current);
    this.#tasks = this.#tasks.filter((task) => !due.includes(task));
    for (const task of due) task.callback();
  }

  /** Lo que falta para cada temporizador pendiente, en ms. */
  get pending(): number[] {
    return this.#tasks.filter((task) => !task.cancelled).map((task) => task.at - this.current);
  }
}

/** Avisos falsos: guarda lo que se ensena y contesta con `reply`. */
export class FakeNotifications implements Notifications {
  readonly infos: string[] = [];
  readonly errors: { readonly message: string; readonly actions: string[] }[] = [];
  reply: string | undefined;

  showInfo(message: string): void {
    this.infos.push(message);
  }

  showError(message: string, ...actions: string[]): Promise<string | undefined> {
    this.errors.push({ message, actions });
    return Promise.resolve(this.reply);
  }
}

/** Dialogos falsos: elige el elemento `choose` (o cancela con `null`). */
export class FakeDialogs implements Dialogs {
  readonly shown: { readonly labels: string[]; readonly placeholder: string }[] = [];
  choose: number | null = 0;

  pick<T>(items: readonly PickItem<T>[], placeholder: string): Promise<T | undefined> {
    this.shown.push({ labels: items.map((item) => item.label), placeholder });
    return Promise.resolve(this.choose === null ? undefined : items[this.choose]?.value);
  }
}

/** Paneles falsos: guarda que se pidio ensenar. */
export class FakePanels implements WebviewSurface {
  readonly shown: ShowSurfaceRequest[] = [];

  show(request: ShowSurfaceRequest): void {
    this.shown.push(request);
  }
}

/** Canal de un panel que guarda lo que el host le manda. */
export function fakeChannel(
  id: string,
  surface: Surface = 'graph',
  repoId = 'repo-1',
): PanelChannel & { readonly posted: HostToWebviewMessage[] } {
  const posted: HostToWebviewMessage[] = [];
  return {
    id,
    surface,
    repoId,
    posted,
    post: (message) => {
      posted.push(message);
    },
  };
}

/** Deja correr las promesas pendientes. */
export async function flush(): Promise<void> {
  for (let round = 0; round < 5; round += 1) await Promise.resolve();
}
