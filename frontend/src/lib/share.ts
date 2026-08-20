import type { DecisionReceipt } from "./types";

export function receiptLink(receipt: DecisionReceipt): string {
  const url = new URL(window.location.href);
  url.search = "";
  url.searchParams.set("receipt", receipt.id);
  return url.toString();
}

export async function copyReceiptLink(receipt: DecisionReceipt): Promise<void> {
  await navigator.clipboard.writeText(receiptLink(receipt));
}

export function downloadReceiptJson(receipt: DecisionReceipt): void {
  downloadBlob(
    new Blob([JSON.stringify(receipt, null, 2)], { type: "application/json" }),
    `${receipt.symbol.toLowerCase()}-${receipt.run_id.slice(0, 8)}-receipt.json`,
  );
}

export function downloadDecisionCard(receipt: DecisionReceipt): void {
  const canvas = document.createElement("canvas");
  canvas.width = 1200;
  canvas.height = 630;
  const context = canvas.getContext("2d");
  if (!context) throw new Error("Image export is unavailable in this browser.");

  context.fillStyle = "#090c14";
  context.fillRect(0, 0, canvas.width, canvas.height);
  context.fillStyle = "#efb94e";
  context.fillRect(0, 0, 14, canvas.height);

  context.fillStyle = "#efb94e";
  context.font = "700 26px Inter, sans-serif";
  context.fillText("RYO-CHAN", 72, 78);
  context.fillStyle = "#a8acb6";
  context.font = "500 20px Inter, sans-serif";
  context.fillText("GLOBAL TOKEN NEWS PULSE", 72, 112);

  context.fillStyle = "#f2f3f6";
  context.font = "700 72px Inter, sans-serif";
  context.fillText(`${receipt.symbol}  ${receipt.signal}`, 72, 222);
  context.fillStyle = "#efb94e";
  context.font = "700 30px Inter, sans-serif";
  context.fillText(`${Math.round(receipt.confidence * 100)}% confidence`, 74, 270);

  context.fillStyle = "#f2f3f6";
  context.font = "600 27px Inter, sans-serif";
  drawWrappedText(context, receipt.summary.conclusion, 72, 348, 1030, 39, 3);

  context.fillStyle = "#a8acb6";
  context.font = "500 21px Inter, sans-serif";
  drawWrappedText(context, `Next: ${receipt.next_action}`, 72, 490, 1030, 31, 2);

  context.fillStyle = "#7c8392";
  context.font = "500 17px Inter, sans-serif";
  context.fillText(
    `${receipt.data_mode.toUpperCase()} DATA  •  ${receipt.practice_plan.label}  •  ${receipt.run_id.slice(0, 8)}`,
    72,
    586,
  );

  canvas.toBlob((blob) => {
    if (!blob) return;
    downloadBlob(blob, `${receipt.symbol.toLowerCase()}-${receipt.run_id.slice(0, 8)}-decision.png`);
  }, "image/png");
}

function drawWrappedText(
  context: CanvasRenderingContext2D,
  text: string,
  x: number,
  y: number,
  maxWidth: number,
  lineHeight: number,
  maxLines: number,
): void {
  const words = text.split(/\s+/);
  let line = "";
  let lineIndex = 0;
  for (const word of words) {
    const candidate = line ? `${line} ${word}` : word;
    if (context.measureText(candidate).width <= maxWidth) {
      line = candidate;
      continue;
    }
    context.fillText(line, x, y + lineIndex * lineHeight);
    lineIndex += 1;
    if (lineIndex >= maxLines) return;
    line = word;
  }
  if (line && lineIndex < maxLines) context.fillText(line, x, y + lineIndex * lineHeight);
}

function downloadBlob(blob: Blob, filename: string): void {
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  anchor.click();
  URL.revokeObjectURL(url);
}
