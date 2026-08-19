// Populate the token datalist that powers the symbol input's suggestions.

import { el } from "../core/dom.js";
import { endpoints } from "../core/api.js";

/** Fill the token datalist; the input stays usable if this fails. */
export async function loadTokens() {
  try {
    const payload = await endpoints.tokens();
    const datalist = el("#tokenList");
    datalist.innerHTML = "";
    for (const token of payload.tokens || []) {
      const option = document.createElement("option");
      option.value = token.symbol;
      option.label = token.name;
      datalist.appendChild(option);
    }
  } catch {
    // Token input remains usable even if the helper list is unavailable.
  }
}
