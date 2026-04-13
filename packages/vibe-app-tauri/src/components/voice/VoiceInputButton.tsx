import { useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { tokens } from "../../design-system/tokens";
import { useSpeechRecognition } from "../../hooks/useSpeechRecognition";
import { useLocalSettings } from "../../local-settings";
import { IconButton } from "../ui/IconButton";
import { VoiceInputIndicator } from "./VoiceInputIndicator";

/**
 * Voice Input Button Props
 */
export interface VoiceInputButtonProps {
  /** Callback when transcript is ready */
  onTranscript?: (transcript: string) => void;
  /** Whether the button is disabled */
  disabled?: boolean;
  /** Size of the button */
  size?: "sm" | "md" | "lg";
  /** Additional className */
  className?: string;
  /** Custom style */
  style?: React.CSSProperties;
  /** Show visual indicator when recording */
  showIndicator?: boolean;
  /** Placeholder text when not recording */
  placeholder?: string;
}

/**
 * VoiceInputButton - Button for voice input
 *
 * Allows users to input text via speech recognition.
 * Press and hold or click to toggle recording.
 *
 * @example
 * ```tsx
 * <VoiceInputButton
 *   onTranscript={(text) => setInputText(text)}
 *   showIndicator
 * />
 * ```
 */
export function VoiceInputButton({
  onTranscript,
  disabled = false,
  size = "md",
  className,
  style,
  showIndicator = true,
  placeholder,
}: VoiceInputButtonProps) {
  const { t } = useTranslation("ui");
  const { settings } = useLocalSettings();

  const {
    isSupported,
    isListening,
    hasPermission,
    transcript,
    interimTranscript,
    error,
    toggleListening,
    stopListening,
    resetTranscript,
    requestPermission,
  } = useSpeechRecognition({
    language: settings.voiceLanguage,
    continuous: false,
    interimResults: true,
    onResult: (text) => {
      onTranscript?.(text);
    },
  });

  // Handle permission request
  useEffect(() => {
    if (isSupported && hasPermission === false && isListening) {
      requestPermission();
    }
  }, [isSupported, hasPermission, isListening, requestPermission]);

  // Handle click
  const handleClick = useCallback(() => {
    if (hasPermission === false) {
      requestPermission();
      return;
    }

    if (isListening) {
      stopListening();
      if (transcript) {
        onTranscript?.(transcript);
        resetTranscript();
      }
    } else {
      resetTranscript();
      toggleListening();
    }
  }, [
    hasPermission,
    isListening,
    transcript,
    stopListening,
    toggleListening,
    resetTranscript,
    requestPermission,
    onTranscript,
  ]);

  // Determine icon based on state
  const getIcon = () => {
    if (!isSupported) {
      return (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <line x1="1" y1="1" x2="23" y2="23" />
          <path d="M9 9v3a3 3 0 0 0 5.12 2.12M15 9.34V4a3 3 0 0 0-5.94-.6" />
          <path d="M17 16.95A7 7 0 0 1 5 12v-2m14 0v2a7 7 0 0 1-.11 1.23" />
          <line x1="12" y1="19" x2="12" y2="23" />
          <line x1="8" y1="23" x2="16" y2="23" />
        </svg>
      );
    }

    if (isListening) {
      return (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <rect x="6" y="4" width="4" height="16" rx="1" />
          <rect x="14" y="4" width="4" height="16" rx="1" />
        </svg>
      );
    }

    return (
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" />
        <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
        <line x1="12" y1="19" x2="12" y2="23" />
        <line x1="8" y1="23" x2="16" y2="23" />
      </svg>
    );
  };

  // Get button title
  const getTitle = () => {
    if (!isSupported) {
      return t("voice.notSupported");
    }
    if (hasPermission === false) {
      return t("voice.permissionRequired");
    }
    if (isListening) {
      return t("voice.input.stop");
    }
    return t("voice.input.start");
  };

  // Don't render if voice input is disabled in settings
  if (!settings.voiceInputEnabled) {
    return null;
  }

  return (
    <div className={className} style={{ position: "relative", ...style }}>
      <IconButton
        onClick={handleClick}
        disabled={disabled || !isSupported}
        title={getTitle()}
        variant={isListening ? "primary" : "ghost"}
        size={size}
        aria-label={getTitle()}
        aria-pressed={isListening}
      >
        {getIcon()}
      </IconButton>

      {/* Visual indicator when recording */}
      {showIndicator && isListening && (
        <VoiceInputIndicator
          interimTranscript={interimTranscript}
          placeholder={placeholder || t("voice.input.listening")}
        />
      )}

      {/* Error tooltip */}
      {error && (
        <div
          style={{
            position: "absolute",
            bottom: "100%",
            left: "50%",
            transform: "translateX(-50%)",
            marginBottom: tokens.spacing[2],
            padding: `${tokens.spacing[2]} ${tokens.spacing[3]}`,
            backgroundColor: "var(--color-error)",
            color: "white",
            borderRadius: tokens.radii.md,
            fontSize: tokens.typography.fontSize.xs,
            whiteSpace: "nowrap",
            zIndex: 1000,
          }}
        >
          {error}
        </div>
      )}
    </div>
  );
}

export default VoiceInputButton;
