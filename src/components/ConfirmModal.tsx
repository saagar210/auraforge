interface ConfirmModalProps {
  open: boolean;
  title: string;
  message: string;
  onConfirm: () => void;
  onCancel: () => void;
  confirmLabel?: string;
  cancelLabel?: string;
}

export function ConfirmModal({
  open,
  title,
  message,
  onConfirm,
  onCancel,
  confirmLabel = "Confirm",
  cancelLabel = "Cancel",
}: ConfirmModalProps) {
  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center animate-[fade-in_0.2s_ease]"
      style={{
        background: "rgba(0, 0, 0, 0.7)",
        backdropFilter: "blur(4px)",
      }}
      onClick={(e) => e.target === e.currentTarget && onCancel()}
      role="dialog"
      aria-modal="true"
      aria-label={title}
    >
      <div className="bg-elevated border border-border-default rounded-xl w-full max-w-md mx-4 shadow-lg overflow-hidden animate-[modal-in_0.3s_ease]">
        <div className="px-6 py-5">
          <h3 className="text-lg font-heading font-semibold text-text-primary mb-3">
            {title}
          </h3>
          <p className="text-sm text-text-secondary whitespace-pre-line">
            {message}
          </p>
        </div>
        <div className="px-6 py-4 border-t border-border-subtle flex justify-end gap-3">
          <button
            onClick={onCancel}
            className="px-4 py-2 text-sm text-text-secondary bg-transparent border border-border-default rounded-lg hover:text-text-primary hover:border-text-muted transition-colors cursor-pointer"
          >
            {cancelLabel}
          </button>
          <button
            onClick={onConfirm}
            className="px-4 py-2 bg-accent-gold text-void text-sm font-medium rounded-lg hover:bg-accent-gold/90 transition-colors cursor-pointer border-none"
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
