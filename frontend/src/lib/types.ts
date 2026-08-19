// Shared types mirroring the Rust backend's decision-receipt shape.

export type ThemeChoice = "light" | "system" | "dark";

export interface HealthResponse {
  status: string;
  service: string;
  generated_at: string;
  ryo_configured: boolean;
  tavily_configured: boolean;
  openai_configured: boolean;
  coingecko_configured: boolean;
  defillama_enabled: boolean;
  dexscreener_enabled: boolean;
  watch_loop_enabled: boolean;
}

export interface TokenInfo {
  symbol: string;
  name: string;
  default_peers: string[];
}

export interface StoryScore {
  relevance: number | null;
  credibility: number | null;
  novelty: number | null;
  urgency: number | null;
  market_confirmation: number | null;
  uncertainty: number | null;
  impact: number;
  formula: string;
}

export interface StoryCard {
  id: string;
  headline: string;
  source: string;
  url: string;
  region: string | null;
  language: string | null;
  timestamp: string | null;
  related_token: string;
  narrative_cluster: string;
  sentiment: string;
  score: StoryScore;
  ryo_alignment: string;
  missing_data: string[];
  recommendation: string;
  confidence: string;
  reasoning: string[];
  category: string;
  data_mode: string;
}

export interface RankedSection {
  key: string;
  label: string;
  cards: StoryCard[];
}

export interface ReceiptSummary {
  headline: string;
  conclusion: string;
  highest_impact: number | null;
  position_changing_count: number;
  watch_count: number;
  noise_count: number;
  unverified_count: number;
}

export interface ReasoningVerdict {
  decision: string;
  confidence: number;
  token_symbol: string;
  top_global_news: string[];
  why_it_matters: string[];
  ryo_market_evidence: string[];
  missing_data: string[];
  warnings: string[];
  recommended_next_action: string;
  generated_by: string;
}

export interface ReasoningLayerOutput {
  signal: string;
  symbol: string;
  confidence: number;
  reasoning: string;
  ryo_tools_used: string[];
  unavailable_data: string[];
  next_action: string;
  affected_tokens: string[];
  sentiment: string;
  market_confirmation: string;
  timestamp: string;
  run_id: string;
}

export interface SourceAvailability {
  source: string;
  status: string;
  data_mode: string;
  detail: string;
  as_of: string;
}

export interface RyoToolEvidence {
  tool: string;
  status: string;
  data_mode: string;
  as_of: string;
  request: unknown;
  result: unknown;
  summary: string | null;
  warnings: string[];
}

export interface NewsStory {
  id: string;
  headline: string;
  source: string;
  url: string;
  content: string;
  published_at: string | null;
  region: string | null;
  language: string | null;
  data_mode: string;
}

export interface DecisionReceipt {
  id: string;
  run_id: string;
  symbol: string;
  timeframe: string;
  regions: string[];
  sources: string[];
  user_thesis: string | null;
  created_at: string;
  status: string;
  data_mode: string;
  signal: string;
  confidence: number;
  reasoning: string;
  ryo_tools_used: string[];
  unavailable_data: string[];
  next_action: string;
  reasoning_layer: ReasoningLayerOutput;
  summary: ReceiptSummary;
  verdict: ReasoningVerdict;
  sections: RankedSection[];
  stories: NewsStory[];
  ryo: RyoToolEvidence[];
  availability: SourceAvailability[];
  warnings: string[];
  scoring_notes: string[];
}

export interface ReceiptListItem {
  id: string;
  run_id: string;
  symbol: string;
  created_at: string;
  status: string;
  data_mode: string;
  signal: string;
  confidence: number;
  headline: string;
  highest_impact: number | null;
}

export interface WatchItem {
  symbol: string;
  interval_minutes: number;
  enabled: boolean;
  created_at: string;
  last_checked_at: string | null;
  next_check_at: string | null;
  last_receipt_id: string | null;
  last_cluster: string | null;
}

export interface PulseRequestBody {
  symbol: string;
  timeframe: string;
  regions: string[];
  sources: string[];
  thesis: string;
}
