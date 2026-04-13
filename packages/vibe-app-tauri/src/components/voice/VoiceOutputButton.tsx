import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { tokens } from "../../design-system/tokens";
import { useSpeechSynthesis } from "../../hooks/useSpeechSynthesis";
import { useLocalSettings } from "../../local-settings";
import { IconButton } from "../ui/IconButton";

/**
 * Voice Output Button Props
 */
export interface VoiceOutputButtonProps {
  /** Text to speak */
  text: string;
  /** Whether the button is disabled */
  disabled?: boolean;
  /** Size of the button */
  size?: "sm" | "md" | "lg";
  /** Additional className */
  className?: string;
  /** Custom style */
  style?: React.CSSProperties;
  /** Callback when speech starts */
  onStart?: () => void;
  /** Callback when speech ends */
  onEnd?: () => void;
  /** Auto-play on mount (if enabled in settings) */
  autoPlay?: boolean;
}

/**
 * VoiceOutputButton - Button for text-to-speech output
 *
 * Plays text content using speech synthesis.
 *
 * @example
 * ```tsx
 * <VoiceOutputButton
 *   text="Hello, world!"
 *   onEnd={() => console.log('Speech finished')}
 * />
 * ```
 */
export function VoiceOutputButton({
  text,
  disabled = false,
  size = "sm",
  className,
  style,
  onStart,
  onEnd,
  autoPlay = false,
}: VoiceOutputButtonProps) {
  const { t } = useTranslation("ui");
  const { settings } = useLocalSettings();

  const {
    isSupported,
    isSpeaking,
    isPaused,
    speak,
    stop,
    pause,
    resume,
    error,
  } = useSpeechSynthesis({
    language: settings.voiceLanguage,
    rate: settings.voiceRate,
    pitch: settings.voicePitch,
    onStart,
    onEnd,
  });

  const handleClick = useCallback(() => {
    if (isSpeaking) {
      if (isPaused) {
        resume();
      } else {
        pause();
      }
    } else {
      speak(text);
    }
  }, [isSpeaking, isPaused, speak, pause, resume, text]);

  const handleStop = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      stop();
    },
    [stop]
  );

  // Determine icon based on state
  const getIcon = () => {
    if (!isSupported) {
      return (
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
          <line x1="23" y1="9" x2="17" y2="15" />
          <line x1="17" y1="9" x2="23" y2="15" />
        </svg>
      );
    }

    if (isPaused) {
      return (
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <polygon points="5 3 19 12 5 21 5 3" />
        </svg>
      );
    }

    if (isSpeaking) {
      return (
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <rect x="6" y="4" width="4" height="16" rx="1" />
          <rect x="14" y="4" width="4" height="16" rx="1" />
        </svg>
      );
    }

    return (
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
        <path d="M15.54 8.46a5 5 0 0 1 0 7.07" />
        <path d="M19.07 4.93a10 10 0 0 1 0 14.14" />
      </svg>
    );
  };

  // Get button title
  const getTitle = () => {
    if (!isSupported) {
      return t("voice.notSupported");
    }
    if (isPaused) {
      return t("voice.output.resume");
    }
    if (isSpeaking) {
      return t("voice.output.pause");
    }
    return t("voice.output.play");
  };

  // Don't render if voice output is disabled in settings
  if (!settings.voiceOutputEnabled) {
    return null;
  }

  return (
    <div className={className} style={{ position: "relative", display: "inline-flex", ...style }}>
      <IconButton
        onClick={handleClick}
        disabled={disabled || !isSupported || !text}
        title={getTitle()}
        variant={isSpeaking ? "primary" : "ghost"}
        size={size}
        aria-label={getTitle()}
        aria-pressed={isSpeaking}
      >
        {getIcon()}
      </IconButton>

      {/* Stop button when speaking */}
      {isSpeaking && (
        <IconButton
          onClick={handleStop}
          title={t("voice.output.stop")}
          variant="danger"
          size={size}
          style={{
            marginLeft: tokens.spacing[1],
          }}
          aria-label={t("voice.output.stop")}
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <rect x="6" y="6" width="12" height="12" rx="1" />
          </svg>
        </IconButton>
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

export default VoiceOutputButton;