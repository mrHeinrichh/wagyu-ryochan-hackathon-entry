// A tiny observable store: single source of truth for UI state.
//
// Views never mutate `state` directly. They call `store.set(...)` (or a
// dedicated action) and subscribe to changes. This keeps state transitions in
// one place and makes re-renders predictable.

const INITIAL_STATE = Object.freeze({
  health: null,
  currentReceipt: null,
  selectedCardId: null,
  theme: localStorage.getItem("gtp-theme") || "system",
});

/**
 * Create an observable store around a plain state object.
 *
 * @param {object} initial - the starting state
 * @returns store with get/set/update/subscribe
 */
function createStore(initial) {
  let state = { ...initial };
  const subscribers = new Set();

  const notify = (changedKeys) => {
    for (const subscriber of subscribers) {
      subscriber(state, changedKeys);
    }
  };

  return {
    /** Return the current (frozen-in-spirit) state snapshot. */
    get() {
      return state;
    },

    /** Shallow-merge a patch and notify subscribers of the changed keys. */
    set(patch) {
      const changedKeys = Object.keys(patch).filter((key) => state[key] !== patch[key]);
      if (changedKeys.length === 0) return;
      state = { ...state, ...patch };
      notify(changedKeys);
    },

    /** Compute a patch from the current state, then apply it. */
    update(updater) {
      this.set(updater(state));
    },

    /**
     * Subscribe to state changes. Returns an unsubscribe function.
     * @param {(state: object, changedKeys: string[]) => void} subscriber
     */
    subscribe(subscriber) {
      subscribers.add(subscriber);
      return () => subscribers.delete(subscriber);
    },
  };
}

export const store = createStore(INITIAL_STATE);
