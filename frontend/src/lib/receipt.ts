// Small read helpers over the receipt shape.

import type { DecisionReceipt, StoryCard } from "./types";

export function allCards(receipt: DecisionReceipt): StoryCard[] {
  return receipt.sections.flatMap((section) => section.cards);
}

export function firstCard(receipt: DecisionReceipt): StoryCard | undefined {
  return allCards(receipt)[0];
}

export function findCard(receipt: DecisionReceipt, cardId: string | null): StoryCard | undefined {
  if (!cardId) return undefined;
  return allCards(receipt).find((card) => card.id === cardId);
}
