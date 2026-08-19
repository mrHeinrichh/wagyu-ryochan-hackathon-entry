// Read helpers for the pulse form controls.

import { el, all } from "./dom.js";

/** Value of the checked radio in a group, or empty string. */
export function valueOfRadio(name) {
  return el(`input[name="${name}"]:checked`)?.value || "";
}

/** Values of all checked boxes in a group. */
export function checkedValues(name) {
  return all(`input[name="${name}"]:checked`).map((node) => node.value);
}

/** Collect the full pulse request body from the form. */
export function readPulseForm() {
  return {
    symbol: el("#symbolInput").value,
    timeframe: valueOfRadio("timeframe"),
    regions: checkedValues("regions"),
    sources: checkedValues("sources"),
    thesis: el("#thesisInput").value,
  };
}
