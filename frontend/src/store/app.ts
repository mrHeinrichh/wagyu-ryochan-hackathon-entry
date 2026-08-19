// Central UI store (Zustand). Single source of truth for cross-component
// state: theme, the current receipt, and the selected card.

import { create } from "zustand";
import type { DecisionReceipt, ThemeChoice } from "@/lib/types";
import { firstCard } from "@/lib/receipt";

const THEMES: ThemeChoice[] = ["light", "system", "dark"];

function initialTheme(): ThemeChoice {
  if (typeof window === "undefined") return "system";
  const stored = window.localStorage.getItem("gtp-theme");
  return stored && (THEMES as string[]).includes(stored) ? (stored as ThemeChoice) : "system";
}

interface AppState {
  theme: ThemeChoice;
  sidebarCollapsed: boolean;
  currentReceipt: DecisionReceipt | null;
  selectedCardId: string | null;
  setTheme: (theme: ThemeChoice) => void;
  toggleSidebar: () => void;
  setReceipt: (receipt: DecisionReceipt) => void;
  selectCard: (cardId: string) => void;
}

export const useAppStore = create<AppState>((set) => ({
  theme: initialTheme(),
  sidebarCollapsed: false,
  currentReceipt: null,
  selectedCardId: null,

  setTheme: (theme) => {
    const next = (THEMES as string[]).includes(theme) ? theme : "system";
    if (typeof window !== "undefined") {
      window.localStorage.setItem("gtp-theme", next);
      document.documentElement.dataset.theme = next;
    }
    set({ theme: next });
  },

  toggleSidebar: () => set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),

  setReceipt: (receipt) =>
    set({ currentReceipt: receipt, selectedCardId: firstCard(receipt)?.id ?? null }),

  selectCard: (cardId) => set({ selectedCardId: cardId }),
}));
