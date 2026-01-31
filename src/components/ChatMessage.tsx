import { clsx } from "clsx";
import type { Message } from "../types";

interface ChatMessageProps {
  message: Message;
}

export function ChatMessage({ message }: ChatMessageProps) {
  const isUser = message.role === "user";

  return (
    <div
      className={clsx(
        "max-w-[85%] px-4 py-3 shadow-sm",
        isUser
          ? "self-end bg-surface rounded-xl rounded-br-sm animate-[message-in-right_0.3s_ease]"
          : "self-start bg-warm rounded-xl rounded-bl-sm border-l-2 border-l-accent-glow animate-[message-in-left_0.3s_ease]",
        !isUser && "relative",
      )}
      style={
        !isUser
          ? {
              boxShadow:
                "0 2px 4px rgba(0,0,0,0.3), -4px 0 20px rgba(232,160,69,0.15)",
            }
          : undefined
      }
    >
      {/* Glow dot for AI messages */}
      {!isUser && (
        <div
          className="absolute -left-3 top-3 w-2 h-2 bg-accent-glow rounded-full"
          style={{ boxShadow: "0 0 10px #E8A045" }}
        />
      )}
      <div className="text-sm text-text-primary leading-relaxed whitespace-pre-wrap break-words">
        {message.content}
      </div>
    </div>
  );
}

interface StreamingMessageProps {
  content: string;
}

export function StreamingMessage({ content }: StreamingMessageProps) {
  return (
    <div
      className="self-start max-w-[85%] px-4 py-3 bg-warm rounded-xl rounded-bl-sm border-l-2 border-l-accent-glow relative animate-[message-in-left_0.3s_ease]"
      style={{
        boxShadow:
          "0 2px 4px rgba(0,0,0,0.3), -4px 0 20px rgba(232,160,69,0.15)",
      }}
    >
      <div
        className="absolute -left-3 top-3 w-2 h-2 bg-accent-glow rounded-full"
        style={{ boxShadow: "0 0 10px #E8A045" }}
      />
      <div className="text-sm text-text-primary leading-relaxed whitespace-pre-wrap break-words">
        {content}
        <span className="inline-block w-0.5 h-4 bg-accent-glow ml-0.5 align-middle animate-pulse" />
      </div>
    </div>
  );
}
