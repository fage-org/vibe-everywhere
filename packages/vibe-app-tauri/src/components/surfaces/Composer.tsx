import { useState, useRef, useCallback, type ReactNode, type KeyboardEvent } from "react";
import { tokens } from "../../design-system/tokens";
import { Caption1 } from "../ui/Typography";

export interface ComposerSuggestion {
  id: string;
  label: string;
  icon?: ReactNode;
  description?: string;
  insertText: string;
}

interface ComposerProps {
  /** Current input value */
  value: string;
  /** Callback when value changes */
  onChange: (value: string) => void;
  /** Callback when message is submitted */
  onSubmit: () => void;
  /** Placeholder text */
  placeholder?: string;
  /** Whether input is disabled */
  disabled?: boolean;
  /** Whether message is being sent */
  isSending?: boolean;
  /** Suggestions for autocomplete */
  suggestions?: ComposerSuggestion[];
  /** Callback when suggestion is selected */
  onSuggestionSelect?: (suggestion: ComposerSuggestion) => void;
  /** Additional toolbar elements */
  toolbar?: ReactNode;
  /** Footer elements (e.g., model selector) */
  footer?: ReactNode;
}

/**
 * Composer - Message input composer matching Happy's AgentInput
 *
 * Features:
 * - Auto-resizing textarea
 * - Keyboard shortcuts (Cmd+Enter to send)
 * - Autocomplete suggestions
 * - Toolbar for attachments/actions
 * - Character count
 * - Disabled/sending states
 */
export function Composer({
  value,
  onChange,
  onSubmit,
  placeholder = "Message...",
  disabled = false,
  isSending = false,
  suggestions = [],
  onSuggestionSelect,
  toolbar,
  footer,
}: ComposerProps) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [selectedSuggestionIndex, setSelectedSuggestionIndex] = useState(0);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent<HTMLTextAreaElement>) => {
      // Handle suggestion navigation
      if (showSuggestions && suggestions.length > 0) {
        if (e.key === "ArrowDown") {
          e.preventDefault();
          setSelectedSuggestionIndex((prev) => (prev + 1) % suggestions.length);
          return;
        }
        if (e.key === "ArrowUp") {
          e.preventDefault();
          setSelectedSuggestionIndex((prev) => (prev - 1 + suggestions.length) % suggestions.length);
          return;
        }
        if (e.key === "Enter" && !e.shiftKey) {
          e.preventDefault();
          const suggestion = suggestions[selectedSuggestionIndex];
          if (suggestion) {
            onSuggestionSelect?.(suggestion);
            setShowSuggestions(false);
          }
          return;
        }
        if (e.key === "Escape") {
          setShowSuggestions(false);
          return;
        }
      }

      // Submit on Cmd/Ctrl + Enter
      if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        if (!disabled && !isSending && value.trim()) {
          onSubmit();
        }
        return;
      }

      // Check for suggestion trigger (@ or /)
      if (e.key === "@" || e.key === "/") {
        setShowSuggestions(true);
        setSelectedSuggestionIndex(0);
      }
    },
    [showSuggestions, suggestions, selectedSuggestionIndex, disabled, isSending, value, onSubmit, onSuggestionSelect],
  );

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLTextAreaElement>) => {
      onChange(e.target.value);

      // Auto-resize
      const textarea = e.target;
      textarea.style.height = "auto";
      textarea.style.height = `${Math.min(textarea.scrollHeight, 200)}px`;

      // Hide suggestions if no trigger character
      if (!textarea.value.includes("@") && !textarea.value.includes("/")) {
        setShowSuggestions(false);
      }
    },
    [onChange],
  );

  const handleSuggestionClick = useCallback(
    (suggestion: ComposerSuggestion) => {
      onSuggestionSelect?.(suggestion);
      setShowSuggestions(false);
      textareaRef.current?.focus();
    },
    [onSuggestionSelect],
  );

  const canSubmit = !disabled && !isSending && value.trim().length > 0;

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: tokens.spacing[2],
        padding: tokens.spacing[3],
        backgroundColor: "var(--surface-primary)",
        borderTop: "1px solid var(--border-primary)",
      }}
    >
      {/* Suggestions Dropdown */}
      {showSuggestions && suggestions.length > 0 && (
        <div
          style={{
            position: "absolute",
            bottom: "100%",
            left: 0,
            right: 0,
            marginBottom: tokens.spacing[2],
            backgroundColor: "var(--surface-elevated)",
            borderRadius: tokens.radii.lg,
            boxShadow: tokens.shadows.ios.large,
            border: "1px solid var(--border-primary)",
            maxHeight: "200px",
            overflow: "auto",
            zIndex: tokens.zIndex.popover,
          }}
        >
          {suggestions.map((suggestion, index) => (
            <button
              key={suggestion.id}
              onClick={() => handleSuggestionClick(suggestion)}
              style={{
                display: "flex",
                alignItems: "center",
                gap: tokens.spacing[3],
                width: "100%",
                padding: `${tokens.spacing[2]} ${tokens.spacing[3]}`,
                backgroundColor: index === selectedSuggestionIndex ? "var(--surface-hover)" : "transparent",
                border: "none",
                borderBottom: "1px solid var(--border-primary)",
                cursor: "pointer",
                textAlign: "left",
              }}
              onMouseEnter={() => setSelectedSuggestionIndex(index)}
            >
              {suggestion.icon && (
                <span style={{ color: "var(--text-tertiary)" }}>{suggestion.icon}</span>
              )}
              <div style={{ flex: 1 }}>
                <Caption1 color="primary">{suggestion.label}</Caption1>
                {suggestion.description && (
                  <Caption1 color="tertiary">{suggestion.description}</Caption1>
                )}
              </div>
            </button>
          ))}
        </div>
      )}

      {/* Toolbar */}
      {toolbar && (
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: tokens.spacing[2],
          }}
        >
          {toolbar}
        </div>
      )}

      {/* Input Area */}
      <div
        style={{
          display: "flex",
          alignItems: "flex-end",
          gap: tokens.spacing[2],
          backgroundColor: "var(--surface-secondary)",
          borderRadius: tokens.radii.xl,
          padding: `${tokens.spacing[2]} ${tokens.spacing[3]}`,
          border: "1px solid var(--border-primary)",
          transition: `border-color ${tokens.animation.duration.fast} ${tokens.animation.easing.default}`,
        }}
        onFocus={(e) => {
          e.currentTarget.style.borderColor = "var(--color-primary)";
        }}
        onBlur={(e) => {
          e.currentTarget.style.borderColor = "var(--border-primary)";
        }}
      >
        <textarea
          ref={textareaRef}
          value={value}
          onChange={handleChange}
          onKeyDown={handleKeyDown}
          placeholder={placeholder}
          disabled={disabled || isSending}
          rows={1}
          style={{
            flex: 1,
            minHeight: "24px",
            maxHeight: "200px",
            padding: 0,
            backgroundColor: "transparent",
            border: "none",
            outline: "none",
            resize: "none",
            fontFamily: "inherit",
            fontSize: tokens.typography.fontSize.base,
            lineHeight: tokens.typography.lineHeight.relaxed,
            color: "var(--text-primary)",
            cursor: disabled ? "not-allowed" : "text",
          }}
        />

        <SendButton onClick={onSubmit} disabled={!canSubmit} isLoading={isSending} />
      </div>

      {/* Footer */}
      {footer && (
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            gap: tokens.spacing[2],
          }}
        >
          {footer}
          <Caption1 color="quaternary">
            {value.length > 0 && `${value.length} chars`}
          </Caption1>
        </div>
      )}
    </div>
  );
}

interface SendButtonProps {
  onClick: () => void;
  disabled: boolean;
  isLoading: boolean;
}

function SendButton({ onClick, disabled, isLoading }: SendButtonProps) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        width: "32px",
        height: "32px",
        borderRadius: tokens.radii.full,
        backgroundColor: disabled ? "var(--surface-tertiary)" : "var(--color-primary)",
        border: "none",
        cursor: disabled ? "not-allowed" : "pointer",
        transition: `all ${tokens.animation.duration.fast} ${tokens.animation.easing.ios}`,
        flexShrink: 0,
      }}
    >
      {isLoading ? (
        <span
          style={{
            width: "16px",
            height: "16px",
            border: "2px solid rgba(255, 255, 255, 0.3)",
            borderTopColor: "#ffffff",
            borderRadius: "50%",
            animation: "spin 1s linear infinite",
          }}
        />
      ) : (
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="#ffffff"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <line x1="22" y1="2" x2="11" y2="13" />
          <polygon points="22 2 15 22 11 13 2 9 22 2" />
        </svg>
      )}
    </button>
  );
}
