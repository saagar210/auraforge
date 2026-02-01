export function normalizeError(error: unknown): string {
  if (typeof error === "string") return error;

  if (error && typeof error === "object") {
    const maybeError = error as Record<string, unknown>;

    const direct = extractMessage(maybeError);
    if (direct) return direct;

    if (maybeError.error && typeof maybeError.error === "object") {
      const nested = extractMessage(maybeError.error as Record<string, unknown>);
      if (nested) return nested;
    }

    if (typeof maybeError.message === "string") {
      const parsed = tryParseErrorResponse(maybeError.message);
      if (parsed) return parsed;
      return maybeError.message;
    }
  }

  try {
    return JSON.stringify(error);
  } catch {
    return String(error);
  }
}

function extractMessage(error: Record<string, unknown>): string | null {
  if (typeof error.message === "string") {
    const parsed = tryParseErrorResponse(error.message);
    if (parsed) return parsed;
    return error.message;
  }
  if (typeof error.code === "string" && typeof error.message === "string") {
    return error.message;
  }
  if (typeof error.code === "string" && typeof error.message !== "string") {
    return String(error.code);
  }
  return null;
}

function tryParseErrorResponse(raw: string): string | null {
  if (!raw.trim().startsWith("{")) return null;
  try {
    const parsed = JSON.parse(raw) as { message?: unknown; action?: unknown };
    if (!parsed || typeof parsed !== "object") return null;
    if (typeof parsed.message === "string") {
      return parsed.action
        ? `${parsed.message} (${String(parsed.action)})`
        : parsed.message;
    }
  } catch {
    return null;
  }
  return null;
}

export function friendlyError(error: string): {
  message: string;
  suggestion: string;
} {
  const lower = error.toLowerCase();

  if (lower.includes("connection refused") || lower.includes("failed to connect to ollama")) {
    return {
      message: "Can't reach Ollama",
      suggestion:
        "Is it running? Look for the llama icon in your menu bar, or open Ollama from Applications.",
    };
  }

  if (/model.*not found/i.test(error)) {
    return {
      message: "AI model isn't installed",
      suggestion:
        "Go to Settings and click 'Re-run Setup' to download the model.",
    };
  }

  if (lower.includes("429") || lower.includes("too many requests")) {
    return {
      message: "Search is taking a break",
      suggestion:
        "Your conversation will continue without web search. Try again later.",
    };
  }

  if (
    lower.includes("enospc") ||
    lower.includes("error 28") ||
    (lower.includes("disk") && lower.includes("full"))
  ) {
    return {
      message: "Your disk is full",
      suggestion: "Free up some space and try again.",
    };
  }

  if (lower.includes("stream") && lower.includes("interrupt")) {
    return {
      message: "The AI response was interrupted",
      suggestion: "Click Retry to try again.",
    };
  }

  if (lower.includes("stream") && lower.includes("cancel")) {
    return {
      message: "Response cancelled",
      suggestion: "You can send a new message anytime.",
    };
  }

  if (lower.includes("sqlite") || lower.includes("database")) {
    return {
      message: "Something went wrong saving your data",
      suggestion: "Try restarting AuraForge.",
    };
  }

  return {
    message: "Something went wrong",
    suggestion: error,
  };
}
