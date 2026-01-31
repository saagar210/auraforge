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
