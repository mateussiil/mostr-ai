import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const CIRCUMFERENCE = 81.68; // 2 * PI * r(13)
const CRITICAL_THRESHOLD = 90;

interface UsageSnapshot {
  percent: number;
  resets_at: string | null;
  stale: boolean;
  last_error: string | null;
}

interface UsagePayload {
  claude: UsageSnapshot | null;
  cursor: UsageSnapshot | null;
  codex: UsageSnapshot | null;
}

const PROVIDERS = ["claude", "cursor", "codex"] as const;

function formatCountdown(resetsAt: string | null): string {
  if (!resetsAt) return "—";
  const target = new Date(resetsAt).getTime();
  if (Number.isNaN(target)) return "—";

  const diffMs = target - Date.now();
  if (diffMs <= 0) return "reset ~agora";

  const totalMinutes = Math.round(diffMs / 60000);
  const totalHours = Math.floor(totalMinutes / 60);
  const totalDays = Math.floor(totalHours / 24);

  if (totalDays >= 31) {
    const months = Math.floor(totalDays / 30);
    const days = totalDays % 30;
    const monthLabel = months === 1 ? "mês" : "meses";
    if (days === 0) return `reset ${months} ${monthLabel}`;
    const dayLabel = days === 1 ? "dia" : "dias";
    return `reset ${months} ${monthLabel} e ${days} ${dayLabel}`;
  }
  if (totalDays >= 1) return `reset ${totalDays}d`;
  if (totalHours >= 1) return `reset ${totalHours}h`;
  return `reset ${totalMinutes}m`;
}

function renderChip(id: (typeof PROVIDERS)[number], snapshot: UsageSnapshot | null) {
  const chip = document.getElementById(`chip-${id}`);
  const value = document.getElementById(`value-${id}`);
  const reset = document.getElementById(`reset-${id}`);
  const progress = chip?.querySelector<SVGCircleElement>(".ring-progress");
  if (!chip || !value || !reset || !progress) return;

  if (!snapshot) {
    value.textContent = "--";
    reset.textContent = "sem dados";
    delete reset.dataset.resetsAt;
    progress.style.strokeDashoffset = `${CIRCUMFERENCE}`;
    chip.classList.remove("critical", "stale");
    return;
  }

  const pct = Math.round(snapshot.percent);
  value.textContent = `${pct}%`;
  if (snapshot.resets_at) {
    reset.dataset.resetsAt = snapshot.resets_at;
  } else {
    delete reset.dataset.resetsAt;
  }
  reset.textContent = formatCountdown(snapshot.resets_at);
  progress.style.strokeDashoffset = `${CIRCUMFERENCE * (1 - snapshot.percent / 100)}`;
  chip.classList.toggle("critical", snapshot.percent >= CRITICAL_THRESHOLD);
  chip.classList.toggle("stale", snapshot.stale);
}

function setupContextMenu() {
  const menu = document.getElementById("context-menu");
  const autostartCheckbox = document.getElementById("menu-autostart") as HTMLInputElement | null;
  const quitItem = document.getElementById("menu-quit");
  if (!menu || !autostartCheckbox || !quitItem) return;

  const closeMenu = () => {
    if (menu.classList.contains("hidden")) return;
    menu.classList.add("hidden");
    void invoke("pin_interactive", { pinned: false });
  };

  document.addEventListener("contextmenu", async (event) => {
    event.preventDefault();
    void invoke("pin_interactive", { pinned: true });
    autostartCheckbox.checked = await invoke<boolean>("get_autostart_enabled");
    menu.style.left = `${event.clientX}px`;
    menu.style.top = `${event.clientY}px`;
    menu.classList.remove("hidden");
  });

  document.addEventListener(
    "click",
    (event) => {
      if (menu.classList.contains("hidden")) return;
      if (event.target instanceof Node && menu.contains(event.target)) return;
      closeMenu();
    },
    true,
  );

  autostartCheckbox.addEventListener("change", async () => {
    autostartCheckbox.checked = await invoke<boolean>("toggle_autostart");
  });

  quitItem.addEventListener("click", () => {
    void invoke("quit_app");
  });
}

function fitWindowToContent() {
  const panel = document.getElementById("panel");
  if (!panel) return;
  requestAnimationFrame(() => {
    void invoke("resize_panel", { width: panel.getBoundingClientRect().width });
  });
}

function setupCloseAndRemoveButtons() {
  document.getElementById("panel-close")?.addEventListener("click", (event) => {
    event.stopPropagation();
    void invoke("quit_app");
  });

  for (const button of document.querySelectorAll<HTMLButtonElement>(".chip-remove")) {
    button.addEventListener("click", (event) => {
      event.stopPropagation();
      button.closest(".chip")?.classList.add("hidden");
      fitWindowToContent();
    });
  }
}

async function main() {
  const panel = document.getElementById("panel");
  if (!panel) return;

  panel.addEventListener("mousedown", (event) => {
    if (event.button !== 0) return;
    if (event.target instanceof HTMLElement && event.target.closest(".x-button")) return;
    void invoke("start_drag");
  });

  setupContextMenu();
  setupCloseAndRemoveButtons();
  fitWindowToContent();

  await listen<boolean>("unlock-changed", (event) => {
    document.body.classList.toggle("unlocked", event.payload);
  });

  await listen<UsagePayload>("usage-updated", (event) => {
    renderChip("claude", event.payload.claude);
    renderChip("cursor", event.payload.cursor);
    renderChip("codex", event.payload.codex);
  });

  // Keep the reset countdown fresh between refreshes without waiting on new data.
  setInterval(() => {
    for (const id of PROVIDERS) {
      const reset = document.getElementById(`reset-${id}`);
      const resetsAt = reset?.dataset.resetsAt;
      if (reset && resetsAt) {
        reset.textContent = formatCountdown(resetsAt);
      }
    }
  }, 30_000);
}

void main();
