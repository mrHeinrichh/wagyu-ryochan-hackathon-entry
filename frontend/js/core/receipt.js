// Small read helpers over the receipt shape. Pure, no DOM.

/** Flatten every card across all ranked sections. */
export function allCards(receipt) {
  return receipt.sections.flatMap((section) => section.cards);
}

/** The first card in ranked order, if any. */
export function firstCard(receipt) {
  return allCards(receipt)[0];
}

/** Find a card by id across all sections. */
export function findCard(receipt, cardId) {
  return allCards(receipt).find((card) => card.id === cardId);
}
