// Pure formatting helpers. No DOM, no side effects.

export function percentNumber(value: number | string | null | undefined): number | string {
  if (value === null || value === undefined || value === "") return "-";
  const numeric = Number(value);
  if (Number.isNaN(numeric)) return String(value);
  return numeric <= 1 ? Math.round(numeric * 100) : Math.round(numeric);
}

export function percentText(value: number | string | null | undefined): string {
  if (value === null || value === undefined || value === "") return "-";
  const numeric = Number(value);
  if (Number.isNaN(numeric)) return String(value);
  return `${percentNumber(numeric)}%`;
}

export function nullableScore(value: number | null | undefined): string {
  return value === null || value === undefined ? "n/a" : String(value);
}

export function shortDate(value: string | null | undefined): string {
  if (!value) return "-";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString([], {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}
