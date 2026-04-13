import { tokens } from "../../design-system/tokens";

/**
 * Voice Input Indicator Props
 */
export interface VoiceInputIndicatorProps {
  /** Interim transcript text */
  interimTranscript?: string;
  /** Placeholder text when no interim transcript */
  placeholder?: string;
  /** Additional className */
  className?: string;
  /** Custom style */
  style?: React.CSSProperties;
}

/**
 * VoiceInputIndicator - Visual indicator for voice input
 *
 * Shows animated bars and interim transcript while recording.
 *
 * @example
 * ```tsx
 * <VoiceInputIndicator
 *   interimTranscript="Hello..."
 *   placeholder="Listening..."
 * />
 * ```
 */
export function VoiceInputIndicator({
  interimTranscript,
  placeholder = "Listening...",
  className,
  style,
}: VoiceInputIndicatorProps) {
  return (
    <div
      className={className}
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: tokens.spacing[3],
        padding: tokens.spacing[4],
        ...style,
      }}
    >
      {/* Animated waveform */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: tokens.spacing[1],
          height: 24,
        }}
      >
        {[0, 1, 2, 3, 4].map((i) => (
          <div
            key={i}
            style={{
              width: 3,
              height: "100%",
              backgroundColor: "var(--color-primary)",
              borderRadius: tokens.radii.full,
              animation: `voice-bar-${i} 0.8s ease-in-out infinite`,
              animationDelay: `${i * 0.1}s`,
            }}
          />
        ))}
      </div>

      {/* Interim transcript or placeholder */}
      <div
        style={{
          fontSize: tokens.typography.fontSize.sm,
          color: interimTranscript ? "var(--text-primary)" : "var(--text-tertiary)",
          textAlign: "center",
          maxWidth: 200,
          overflow: "hidden",
          textOverflow: "ellipsis",
        }}
      >
        {interimTranscript || placeholder}
      </div>

      {/* CSS animations for waveform bars */}
      <style>{`
        @keyframes voice-bar-0 {
          0%, 100% { transform: scaleY(0.3); }
          50% { transform: scaleY(1); }
        }
        @keyframes voice-bar-1 {
          0%, 100% { transform: scaleY(0.5); }
          50% { transform: scaleY(0.8); }
        }
        @keyframes voice-bar-2 {
          0%, 100% { transform: scaleY(0.4); }
          50% { transform: scaleY(1); }
        }
        @keyframes voice-bar-3 {
          0%, 100% { transform: scaleY(0.6); }
          50% { transform: scaleY(0.9); }
        }
        @keyframes voice-bar-4 {
          0%, 100% { transform: scaleY(0.3); }
          50% { transform: scaleY(0.7); }
        }
      `}</style>
    </div>
  );
}

export default VoiceInputIndicator;
