// Theme control: reads persisted preference, applies it, and keeps the
// segmented sun/monitor/moon toggle in sync.

import { all } from "./dom.js";
import { store } from "./store.js";

const THEMES = ["light", "system", "dark"];

/** Apply the current theme and wire up the toggle buttons. */
export function initTheme() {
  setTheme(store.get().theme);
  for (const button of all("[data-theme-choice]")) {
    button.addEventListener("click", () => setTheme(button.dataset.themeChoice));
  }
}

/** Persist and apply a theme, then refresh the toggle's active state. */
export function setTheme(theme) {
  const next = THEMES.includes(theme) ? theme : "system";
  store.set({ theme: next });
  localStorage.setItem("gtp-theme", next);
  document.documentElement.dataset.theme = next;
  renderThemeControls(next);
}

/** Reflect the active theme on the toggle buttons. */
function renderThemeControls(active) {
  for (const button of all("[data-theme-choice]")) {
    const isActive = button.dataset.themeChoice === active;
    button.classList.toggle("active", isActive);
    button.setAttribute("aria-pressed", isActive ? "true" : "false");
  }
}
