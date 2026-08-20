"use client";

import { FileJson, ImageDown, Link2 } from "lucide-react";
import type { DecisionReceipt } from "@/lib/types";
import { copyReceiptLink, downloadDecisionCard, downloadReceiptJson } from "@/lib/share";

export default function ReceiptActions({
  receipt,
  onNotice,
}: {
  receipt: DecisionReceipt;
  onNotice: (message: string) => void;
}) {
  const copyLink = async () => {
    try {
      await copyReceiptLink(receipt);
      onNotice("Receipt link copied.");
    } catch {
      onNotice("Could not copy the receipt link.");
    }
  };

  return (
    <div className="receipt-actions" aria-label="Receipt actions">
      <button type="button" className="icon-button" title="Copy receipt link" aria-label="Copy receipt link" onClick={copyLink}>
        <Link2 aria-hidden="true" />
      </button>
      <button type="button" className="icon-button" title="Download social image" aria-label="Download decision image" onClick={() => downloadDecisionCard(receipt)}>
        <ImageDown aria-hidden="true" />
      </button>
      <button type="button" className="icon-button" title="Download receipt JSON" aria-label="Download receipt JSON" onClick={() => downloadReceiptJson(receipt)}>
        <FileJson aria-hidden="true" />
      </button>
    </div>
  );
}
