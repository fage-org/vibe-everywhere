import {
  forwardRef,
  useRef,
  useEffect,
  useCallback,
  type KeyboardEvent,
  type CSSProperties,
} from "react";
import { tokens } from "../../design-system/tokens";

export type SupportedKey =
  | "Enter"
  | "Escape"
  | "ArrowUp"
  | "ArrowDown"
  | "ArrowLeft"
  | "ArrowRight"
  | "Tab";

export interface KeyPressEvent {
  key: SupportedKey;
  shiftKey: boolean;
  metaKey: boolean;
  ctrlKey: boolean;
}

export type OnKeyPressCallback = (event: KeyPressEvent) => boolean;

export interface TextInputState {
  text: string;
  selection: {
    start: number;
    end: number;
  };
}

export interface MultiTextInputHandle {
  setTextAndSelection: (
    text: string,
    selection: { start: number; end: number }
  ) => void;
  focus: () => void;
  blur: () => void;
  getState: () => TextInputState;
}

export interface MultiTextInputProps {
  value: string;
  onChange: (text: string) => void;
  placeholder?: string;
  disabled?: boolean;
  maxHeight?: number;
  onKeyPress?: OnKeyPressCallback;
  onSelectionChange?: (selection: { start: number; end: number }) => void;
  onStateChange?: (state: TextInputState) => void;
  style?: CSSProperties;
}

const FONT_SIZE = 16;
const LINE_HEIGHT = 22;

/**
 * MultiTextInput - Multi-line text input with keyboard event handling
 *
 * Features:
 * - Keyboard event handling (Enter, Escape, arrows, Tab)
 * - Selection tracking and control
 * - Auto-resize with max height limit
 * - Imperative handle for direct control
 */
export const MultiTextInput = forwardRef<MultiTextInputHandle, MultiTextInputProps>(
  (props, ref) => {
    const {
      value,
      onChange,
      placeholder,
      disabled = false,
      maxHeight = 120,
      onKeyPress,
      onSelectionChange,
      onStateChange,
      style,
    } = props;

    const inputRef = useRef<HTMLTextAreaElement>(null);
    const selectionRef = useRef({ start: 0, end: 0 });

    // Track disabled state
    useEffect(() => {
      if (disabled && inputRef.current) {
        inputRef.current.blur();
      }
    }, [disabled]);

    // Handle keyboard events
    const handleKeyDown = useCallback(
      (e: KeyboardEvent<HTMLTextAreaElement>) => {
        if (disabled || !onKeyPress) return;

        const key = e.key;
        let normalizedKey: SupportedKey | null = null;

        switch (key) {
          case "Enter":
            normalizedKey = "Enter";
            break;
          case "Escape":
            normalizedKey = "Escape";
            break;
          case "ArrowUp":
          case "Up":
            normalizedKey = "ArrowUp";
            break;
          case "ArrowDown":
          case "Down":
            normalizedKey = "ArrowDown";
            break;
          case "ArrowLeft":
          case "Left":
            normalizedKey = "ArrowLeft";
            break;
          case "ArrowRight":
          case "Right":
            normalizedKey = "ArrowRight";
            break;
          case "Tab":
            normalizedKey = "Tab";
            break;
        }

        if (normalizedKey) {
          const keyEvent: KeyPressEvent = {
            key: normalizedKey,
            shiftKey: e.shiftKey,
            metaKey: e.metaKey,
            ctrlKey: e.ctrlKey,
          };

          const handled = onKeyPress(keyEvent);
          if (handled) {
            e.preventDefault();
          }
        }
      },
      [disabled, onKeyPress]
    );

    // Handle text change
    const handleChange = useCallback(
      (e: React.ChangeEvent<HTMLTextAreaElement>) => {
        const text = e.target.value;
        const selection = { start: text.length, end: text.length };
        selectionRef.current = selection;

        onChange(text);

        if (onStateChange) {
          onStateChange({ text, selection });
        }
        if (onSelectionChange) {
          onSelectionChange(selection);
        }
      },
      [onChange, onStateChange, onSelectionChange]
    );

    // Handle selection change
    const handleSelect = useCallback(
      (e: React.SyntheticEvent<HTMLTextAreaElement>) => {
        const target = e.currentTarget;
        const { selectionStart, selectionEnd } = target;
        const selection = { start: selectionStart, end: selectionEnd };

        if (
          selection.start !== selectionRef.current.start ||
          selection.end !== selectionRef.current.end
        ) {
          selectionRef.current = selection;

          if (onSelectionChange) {
            onSelectionChange(selection);
          }
          if (onStateChange) {
            onStateChange({ text: value, selection });
          }
        }
      },
      [value, onSelectionChange, onStateChange]
    );

    // Imperative handle
    useEffect(() => {
      if (ref) {
        (ref as React.MutableRefObject<MultiTextInputHandle | null>).current = {
          setTextAndSelection: (
            text: string,
            selection: { start: number; end: number }
          ) => {
            if (inputRef.current) {
              inputRef.current.value = text;
              inputRef.current.setSelectionRange(selection.start, selection.end);
              selectionRef.current = selection;

              onChange(text);
              if (onStateChange) {
                onStateChange({ text, selection });
              }
              if (onSelectionChange) {
                onSelectionChange(selection);
              }
            }
          },
          focus: () => {
            inputRef.current?.focus();
          },
          blur: () => {
            inputRef.current?.blur();
          },
          getState: () => ({
            text: value,
            selection: selectionRef.current,
          }),
        };
      }
    }, [ref, value, onChange, onStateChange, onSelectionChange]);

    const textareaStyles: CSSProperties = {
      width: "100%",
      fontSize: FONT_SIZE,
      lineHeight: LINE_HEIGHT,
      maxHeight,
      color: "var(--text-primary)",
      verticalAlign: "top",
      padding: 0,
      paddingTop: tokens.spacing[2],
      paddingBottom: tokens.spacing[2],
      paddingLeft: tokens.spacing[3],
      paddingRight: tokens.spacing[3],
      opacity: disabled ? 0.58 : 1,
      backgroundColor: "var(--surface-secondary)",
      border: "1px solid var(--border-primary)",
      borderRadius: tokens.radii.md,
      resize: "none",
      outline: "none",
      fontFamily: tokens.typography.fontFamily.sans,
      overflow: "auto",
      transition: `border-color ${tokens.animation.duration.fast} ${tokens.animation.easing.default}`,
      ...style,
    };

    const placeholderStyles: CSSProperties = {
      ...textareaStyles,
      color: "var(--text-tertiary)",
      cursor: "not-allowed",
    };

    return (
      <div style={{ width: "100%" }}>
        {disabled ? (
          <div
            style={value ? textareaStyles : placeholderStyles}
            onMouseDown={(e) => e.preventDefault()}
          >
            {value || placeholder || " "}
          </div>
        ) : (
          <textarea
            ref={inputRef}
            style={textareaStyles}
            placeholder={placeholder}
            value={value}
            disabled={disabled}
            onChange={handleChange}
            onKeyDown={handleKeyDown}
            onSelect={handleSelect}
            rows={1}
            autoComplete="off"
            autoCapitalize="sentences"
            spellCheck
            onFocus={(e) => {
              e.currentTarget.style.borderColor = "var(--color-primary)";
            }}
            onBlur={(e) => {
              e.currentTarget.style.borderColor = "var(--border-primary)";
            }}
          />
        )}
      </div>
    );
  }
);

MultiTextInput.displayName = "MultiTextInput";