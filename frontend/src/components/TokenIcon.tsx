"use client";

import { useEffect, useState } from "react";
import type { TokenInfo } from "@/lib/types";

export default function TokenIcon({ token, size = "md" }: { token: TokenInfo | undefined; size?: "sm" | "md" }) {
  const [failed, setFailed] = useState(false);

  useEffect(() => setFailed(false), [token?.image_url, token?.symbol]);

  return (
    <span className={`token-icon ${size}`} aria-hidden="true">
      {token?.image_url && !failed ? (
        <img
          src={token.image_url}
          alt=""
          width={size === "sm" ? 20 : 28}
          height={size === "sm" ? 20 : 28}
          referrerPolicy="no-referrer"
          onError={() => setFailed(true)}
        />
      ) : (
        <span>{token?.symbol.slice(0, 2) || "--"}</span>
      )}
    </span>
  );
}
