"use client";

import Image from "next/image";
import {
  ArrowUp,
  BookOpen,
  ChevronDown,
  MessageCircleMore,
  Sparkles,
  X,
} from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { api } from "@/lib/api";
import type { ChatResponse, ChatRole, ChatTurn, DecisionReceipt } from "@/lib/types";

export type AssistantTab = "chat" | "faq";

interface ResearchAssistantProps {
  receipt: DecisionReceipt | null;
  open: boolean;
  tab: AssistantTab;
  onOpenChange: (open: boolean) => void;
  onTabChange: (tab: AssistantTab) => void;
}

interface UiMessage {
  id: string;
  role: ChatRole;
  content: string;
  suggestions?: string[];
  source?: string;
  warnings?: string[];
}

const FAQS = [
  {
    question: "What is a decision receipt?",
    answer: "A replayable record of the news, RYO evidence, scores, uncertainty, conclusion, and paper-only next action used for one analysis.",
  },
  {
    question: "What does Needs attention mean?",
    answer: "The story is position-changing, highly urgent, or materially bearish under the scoring rules. The red label always includes the reason it was triggered.",
  },
  {
    question: "Is this live trading advice?",
    answer: "No. The product explains research evidence and can record a bounded practice plan. It does not execute trades or promise outcomes.",
  },
  {
    question: "What happens when a source is down?",
    answer: "Unavailable evidence stays marked as missing and is excluded from weighted scoring. The receipt continues with an explicit warning and deterministic fallback where possible.",
  },
  {
    question: "How is the assistant grounded?",
    answer: "For receipt questions, it receives a compact copy of that receipt and is instructed not to add prices, events, or market facts that are absent from it.",
  },
  {
    question: "What does demo mode contain?",
    answer: "A fully labelled simulated receipt for testing the product flow without external keys. Simulated evidence is never presented as live confirmation.",
  },
];

function welcomeMessage(receipt: DecisionReceipt | null): UiMessage {
  return {
    id: `welcome-${receipt?.id || "empty"}`,
    role: "assistant",
    content: receipt
      ? `I am reading the ${receipt.symbol} receipt now. I can explain the conclusion, strongest news, risk controls, or missing evidence.`
      : "Ask me about the workspace, or run an analysis and I will summarize its decision receipt here.",
    source: "RYO-CHAN guide",
  };
}

function localReceiptSummary(receipt: DecisionReceipt): string {
  return `${receipt.symbol} is ${receipt.verdict.decision} at ${receipt.verdict.confidence}% confidence. ${receipt.summary.conclusion} Next: ${receipt.next_action}`;
}

export default function ResearchAssistant({
  receipt,
  open,
  tab,
  onOpenChange,
  onTabChange,
}: ResearchAssistantProps) {
  const [messages, setMessages] = useState<UiMessage[]>([welcomeMessage(receipt)]);
  const [question, setQuestion] = useState("");
  const [loading, setLoading] = useState(false);
  const [unread, setUnread] = useState(0);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let cancelled = false;
    const welcome = welcomeMessage(receipt);
    setMessages([welcome]);
    setUnread(0);
    if (!receipt) return;

    setLoading(true);
    api
      .chat({
        receipt_id: receipt.id,
        question: "Summarize this result, explain why it matters, and suggest what I should inspect next.",
        history: [],
      })
      .then((response) => {
        if (cancelled) return;
        setMessages([welcome, responseMessage(response)]);
        if (window.matchMedia("(min-width: 901px)").matches) {
          onOpenChange(true);
        } else {
          setUnread(1);
        }
      })
      .catch(() => {
        if (cancelled) return;
        setMessages([
          welcome,
          {
            id: `summary-${receipt.id}`,
            role: "assistant",
            content: localReceiptSummary(receipt),
            source: "Receipt fallback",
            suggestions: [
              "What could invalidate this view?",
              "Which headline matters most?",
              "What should I monitor next?",
            ],
          },
        ]);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [receipt?.id, onOpenChange]);

  useEffect(() => {
    if (open) setUnread(0);
  }, [open]);

  useEffect(() => {
    scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight, behavior: "smooth" });
  }, [messages, loading, open]);

  const history = useMemo<ChatTurn[]>(
    () => messages.map(({ role, content }) => ({ role, content })).slice(-8),
    [messages],
  );

  const ask = async (prompt: string) => {
    const clean = prompt.trim();
    if (!clean || loading) return;
    setQuestion("");
    setMessages((current) => [
      ...current,
      { id: `user-${Date.now()}`, role: "user", content: clean },
    ]);
    setLoading(true);
    try {
      const response = await api.chat({
        receipt_id: receipt?.id,
        question: clean,
        history,
      });
      setMessages((current) => [...current, responseMessage(response)]);
    } catch (error) {
      setMessages((current) => [
        ...current,
        {
          id: `error-${Date.now()}`,
          role: "assistant",
          content: error instanceof Error ? error.message : "The assistant is temporarily unavailable.",
          source: "Connection notice",
        },
      ]);
    } finally {
      setLoading(false);
    }
  };

  if (!open) {
    return (
      <button
        type="button"
        className="assistant-launcher"
        aria-label="Open RYO-CHAN assistant"
        title="Open RYO-CHAN assistant"
        data-tour="assistant"
        onClick={() => onOpenChange(true)}
      >
        <Image src="/assets/ryo-avatar.webp" alt="" width={44} height={44} />
        <span className="assistant-launcher-icon" aria-hidden="true">
          <MessageCircleMore />
        </span>
        {unread > 0 ? <span className="assistant-unread">{unread}</span> : null}
      </button>
    );
  }

  return (
    <aside className="assistant-drawer" aria-label="RYO-CHAN research assistant">
      <header className="assistant-header">
        <div className="assistant-identity">
          <span className="assistant-avatar">
            <Image src="/assets/ryo-avatar.webp" alt="" width={38} height={38} />
          </span>
          <div>
            <strong>RYO-CHAN</strong>
            <span>{receipt ? `${receipt.symbol} receipt aware` : "Research guide"}</span>
          </div>
        </div>
        <button
          type="button"
          className="icon-button assistant-close"
          aria-label="Hide assistant"
          title="Hide assistant"
          onClick={() => onOpenChange(false)}
        >
          <X aria-hidden="true" />
        </button>
      </header>

      <div className="assistant-tabs" role="tablist" aria-label="Assistant views">
        <button
          type="button"
          role="tab"
          aria-selected={tab === "chat"}
          className={tab === "chat" ? "active" : undefined}
          onClick={() => onTabChange("chat")}
        >
          <Sparkles aria-hidden="true" />
          Chat
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={tab === "faq"}
          className={tab === "faq" ? "active" : undefined}
          onClick={() => onTabChange("faq")}
        >
          <BookOpen aria-hidden="true" />
          FAQ
        </button>
      </div>

      {tab === "chat" ? (
        <>
          <div className="assistant-messages" ref={scrollRef} role="log" aria-live="polite">
            {messages.map((message) => (
              <div key={message.id} className={`assistant-message ${message.role}`}>
                <span className="message-role">{message.role === "assistant" ? "RYO-CHAN" : "You"}</span>
                <p>
                  {message.role === "assistant" ? <TypingText text={message.content} /> : message.content}
                </p>
                {message.source ? <small>{sourceLabel(message.source)}</small> : null}
                {message.warnings?.map((warning) => (
                  <small key={warning} className="message-warning">{warning}</small>
                ))}
                {message.suggestions?.length ? (
                  <div className="assistant-suggestions" aria-label="Suggested questions">
                    {message.suggestions.map((suggestion) => (
                      <button key={suggestion} type="button" onClick={() => void ask(suggestion)}>
                        {suggestion}
                      </button>
                    ))}
                  </div>
                ) : null}
              </div>
            ))}
            {loading ? (
              <div className="assistant-message assistant typing-status" aria-label="RYO-CHAN is thinking">
                <span className="message-role">RYO-CHAN</span>
                <span className="typing-dots" aria-hidden="true"><i /><i /><i /></span>
              </div>
            ) : null}
          </div>

          <form
            className="assistant-composer"
            onSubmit={(event) => {
              event.preventDefault();
              void ask(question);
            }}
          >
            <label htmlFor="assistantQuestion">Ask about this result</label>
            <div>
              <textarea
                id="assistantQuestion"
                value={question}
                rows={1}
                maxLength={800}
                placeholder={receipt ? `Ask about ${receipt.symbol}` : "Ask about the workspace"}
                onChange={(event) => setQuestion(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === "Enter" && !event.shiftKey) {
                    event.preventDefault();
                    void ask(question);
                  }
                }}
              />
              <button type="submit" aria-label="Send question" title="Send question" disabled={!question.trim() || loading}>
                <ArrowUp aria-hidden="true" />
              </button>
            </div>
          </form>
        </>
      ) : (
        <div className="assistant-faq" role="tabpanel">
          {FAQS.map((item) => (
            <details key={item.question}>
              <summary>
                <span>{item.question}</span>
                <ChevronDown className="chevron" aria-hidden="true" />
              </summary>
              <p>{item.answer}</p>
            </details>
          ))}
        </div>
      )}
    </aside>
  );
}

function responseMessage(response: ChatResponse): UiMessage {
  return {
    id: `assistant-${Date.now()}-${Math.random().toString(16).slice(2)}`,
    role: "assistant",
    content: response.answer,
    suggestions: response.suggestions,
    source: response.generated_by,
    warnings: response.warnings,
  };
}

function sourceLabel(source: string): string {
  if (source.startsWith("openai:")) return "AI summary grounded in this receipt";
  if (source.startsWith("deterministic:")) return "Receipt-based fallback";
  return source;
}

function TypingText({ text }: { text: string }) {
  const [visible, setVisible] = useState("");
  const [done, setDone] = useState(false);

  useEffect(() => {
    const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (reducedMotion) {
      setVisible(text);
      setDone(true);
      return;
    }

    setVisible("");
    setDone(false);
    let index = 0;
    const chunk = Math.max(1, Math.ceil(text.length / 150));
    const interval = window.setInterval(() => {
      index = Math.min(text.length, index + chunk);
      setVisible(text.slice(0, index));
      if (index >= text.length) {
        window.clearInterval(interval);
        setDone(true);
      }
    }, 18);
    return () => window.clearInterval(interval);
  }, [text]);

  return (
    <span className="typing-copy" aria-label={text}>
      <span aria-hidden="true">{visible}</span>
      {!done ? <span className="typing-caret" aria-hidden="true" /> : null}
    </span>
  );
}
