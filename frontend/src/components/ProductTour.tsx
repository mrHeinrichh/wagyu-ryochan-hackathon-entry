"use client";

import { ArrowLeft, ArrowRight, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";

interface ProductTourProps {
  open: boolean;
  onClose: () => void;
}

interface SpotlightRect {
  top: number;
  left: number;
  width: number;
  height: number;
  viewportWidth: number;
  viewportHeight: number;
}

const STEPS = [
  {
    selector: "[data-tour='token']",
    eyebrow: "Choose",
    title: "Pick the market asset",
    description: "The catalog follows CoinGecko's latest ranked market list. The selected token becomes the subject of every evidence check.",
  },
  {
    selector: "[data-tour='context']",
    eyebrow: "Focus",
    title: "Add the question you care about",
    description: "Optional context narrows the research toward a headline, event, or thesis without replacing the live evidence.",
  },
  {
    selector: "[data-tour='timeframe']",
    eyebrow: "Scope",
    title: "Set the decision window",
    description: "Use a short window for immediate narrative shifts or a longer one when you need broader confirmation.",
  },
  {
    selector: "[data-tour='filters']",
    eyebrow: "Refine",
    title: "Control sources and paper risk",
    description: "Filters change regional and source coverage. The risk limit applies only to the simulated practice plan.",
  },
  {
    selector: "[data-tour='actions']",
    eyebrow: "Run",
    title: "Analyze, watch, or try the sample",
    description: "Analyze builds a receipt, Watch schedules repeat checks, and Try sample demonstrates the complete flow with labelled simulated data.",
  },
  {
    selector: "[data-tour='decision'], [data-tour='empty-results']",
    eyebrow: "Decide",
    title: "Read the conclusion first",
    description: "The decision panel puts the verdict, confidence, evidence path, invalidation, and paper-only plan before technical detail.",
  },
  {
    selector: "[data-tour='news'], [data-tour='empty-results']",
    eyebrow: "Explain",
    title: "Open the news that matters",
    description: "Stories are ranked by impact. Red is reserved for evidence that needs attention, and every alert explains why.",
  },
  {
    selector: "[data-tour='assistant']",
    eyebrow: "Ask",
    title: "Continue with RYO-CHAN",
    description: "Open the assistant for a receipt-grounded summary, follow-up questions, suggestions, and the FAQ.",
  },
] as const;

export default function ProductTour({ open, onClose }: ProductTourProps) {
  const [step, setStep] = useState(0);
  const [rect, setRect] = useState<SpotlightRect | null>(null);
  const cardRef = useRef<HTMLElement>(null);
  const current = STEPS[step];

  useEffect(() => {
    if (!open) return;
    setStep(0);
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const target = document.querySelector<HTMLElement>(current.selector);
    const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    target?.scrollIntoView({ behavior: reducedMotion ? "auto" : "smooth", block: "center" });

    const update = () => {
      const targetRect = target?.getBoundingClientRect();
      const padding = 8;
      setRect({
        top: targetRect ? Math.max(8, targetRect.top - padding) : window.innerHeight / 2 - 60,
        left: targetRect ? Math.max(8, targetRect.left - padding) : 16,
        width: targetRect ? Math.min(window.innerWidth - 16, targetRect.width + padding * 2) : window.innerWidth - 32,
        height: targetRect ? targetRect.height + padding * 2 : 120,
        viewportWidth: window.innerWidth,
        viewportHeight: window.innerHeight,
      });
    };

    const timeout = window.setTimeout(update, reducedMotion ? 0 : 260);
    window.addEventListener("resize", update);
    window.addEventListener("scroll", update, true);
    return () => {
      window.clearTimeout(timeout);
      window.removeEventListener("resize", update);
      window.removeEventListener("scroll", update, true);
    };
  }, [current.selector, open]);

  useEffect(() => {
    if (!open) return;
    cardRef.current?.focus();
  }, [open, step, rect]);

  if (!open || !rect) return null;

  const mobile = rect.viewportWidth <= 700;
  const cardWidth = Math.min(340, rect.viewportWidth - 24);
  const roomBelow = rect.viewportHeight - (rect.top + rect.height);
  const cardTop = roomBelow >= 250 ? rect.top + rect.height + 14 : Math.max(12, rect.top - 230);
  const cardLeft = Math.min(Math.max(12, rect.left), rect.viewportWidth - cardWidth - 12);

  const next = () => {
    if (step === STEPS.length - 1) onClose();
    else setStep((value) => value + 1);
  };

  return (
    <div
      className="tour-layer"
      role="dialog"
      aria-modal="true"
      aria-labelledby="tour-title"
      onKeyDown={(event) => {
        if (event.key === "Escape") onClose();
        if (event.key === "ArrowRight") next();
        if (event.key === "ArrowLeft" && step > 0) setStep((value) => value - 1);
        if (event.key === "Tab") {
          const buttons = cardRef.current?.querySelectorAll<HTMLButtonElement>("button:not(:disabled)");
          if (!buttons?.length) return;
          const first = buttons[0];
          const last = buttons[buttons.length - 1];
          if (event.shiftKey && document.activeElement === first) {
            event.preventDefault();
            last.focus();
          } else if (!event.shiftKey && document.activeElement === last) {
            event.preventDefault();
            first.focus();
          }
        }
      }}
    >
      <div className="tour-shield" />
      <div
        className="tour-spotlight"
        style={{ top: rect.top, left: rect.left, width: rect.width, height: rect.height }}
      />
      <section
        ref={cardRef}
        className="tour-card"
        tabIndex={-1}
        style={mobile ? undefined : { top: cardTop, left: cardLeft, width: cardWidth }}
      >
        <div className="tour-card-topline">
          <span>{current.eyebrow}</span>
          <span>{step + 1} / {STEPS.length}</span>
          <button type="button" aria-label="Close tutorial" title="Close tutorial" onClick={onClose}>
            <X aria-hidden="true" />
          </button>
        </div>
        <div className="tour-progress" aria-hidden="true">
          <span style={{ width: `${((step + 1) / STEPS.length) * 100}%` }} />
        </div>
        <h2 id="tour-title">{current.title}</h2>
        <p>{current.description}</p>
        <div className="tour-actions">
          <button type="button" className="tour-back" disabled={step === 0} onClick={() => setStep((value) => value - 1)}>
            <ArrowLeft aria-hidden="true" />
            Back
          </button>
          <button type="button" className="tour-next" onClick={next}>
            {step === STEPS.length - 1 ? "Finish" : "Next"}
            <ArrowRight aria-hidden="true" />
          </button>
        </div>
      </section>
    </div>
  );
}
