import { useState, useEffect, useCallback, useRef } from "react";

/**
 * Speech Synthesis Voice
 */
export interface SpeechVoice {
  name: string;
  lang: string;
  default: boolean;
  localService: boolean;
}

/**
 * Speech Synthesis State
 */
export interface SpeechSynthesisState {
  /** Whether speech synthesis is supported */
  isSupported: boolean;
  /** Whether currently speaking */
  isSpeaking: boolean;
  /** Whether speech is paused */
  isPaused: boolean;
  /** Available voices */
  voices: SpeechVoice[];
  /** Current voice name */
  currentVoice: string | null;
  /** Whether speech is pending */
  isPending: boolean;
  /** Error message if any */
  error: string | null;
}

/**
 * Speech Synthesis Options
 */
export interface SpeechSynthesisOptions {
  /** Language for synthesis (BCP 47 code) */
  language?: string;
  /** Speech rate (0.5 - 2.0) */
  rate?: number;
  /** Speech pitch (0.5 - 2.0) */
  pitch?: number;
  /** Speech volume (0 - 1) */
  volume?: number;
  /** Voice name to use */
  voiceName?: string;
  /** Callback when speech starts */
  onStart?: () => void;
  /** Callback when speech ends */
  onEnd?: () => void;
  /** Callback on error */
  onError?: (error: string) => void;
  /** Callback when speech is paused */
  onPause?: () => void;
  /** Callback when speech is resumed */
  onResume?: () => void;
}

/**
 * Extended SpeechSynthesisUtterance for type safety
 */
interface ExtendedSpeechSynthesisUtterance extends SpeechSynthesisUtterance {
  voice: SpeechSynthesisVoice | null;
}

/**
 * Hook for speech synthesis (Text-to-Speech)
 *
 * Uses the Web Speech API for browser-native text-to-speech.
 * Works on all modern browsers and Tauri desktop apps.
 *
 * @param options - Configuration options
 * @returns Speech synthesis state and controls
 *
 * @example
 * ```tsx
 * const {
 *   isSupported,
 *   isSpeaking,
 *   speak,
 *   stop,
 *   pause,
 *   resume,
 *   voices,
 * } = useSpeechSynthesis({
 *   language: 'en-US',
 *   rate: 1.0,
 *   onEnd: () => console.log('Speech finished'),
 * });
 *
 * if (!isSupported) {
 *   return <p>TTS not supported</p>;
 * }
 *
 * return (
 *   <button onClick={() => speak('Hello, world!')}>
 *     {isSpeaking ? 'Speaking...' : 'Play'}
 *   </button>
 * );
 * ```
 */
export function useSpeechSynthesis(
  options: SpeechSynthesisOptions = {}
): SpeechSynthesisState & {
  /** Speak text */
  speak: (text: string) => void;
  /** Stop speaking */
  stop: () => void;
  /** Pause speaking */
  pause: () => void;
  /** Resume speaking */
  resume: () => void;
  /** Set voice by name */
  setVoice: (voiceName: string) => void;
  /** Cancel current speech */
  cancel: () => void;
} {
  const {
    language = "en-US",
    rate = 1.0,
    pitch = 1.0,
    volume = 1.0,
    voiceName,
    onStart,
    onEnd,
    onError,
    onPause,
    onResume,
  } = options;

  const utteranceRef = useRef<ExtendedSpeechSynthesisUtterance | null>(null);
  const [isSupported, setIsSupported] = useState(false);
  const [isSpeaking, setIsSpeaking] = useState(false);
  const [isPaused, setIsPaused] = useState(false);
  const [isPending, setIsPending] = useState(false);
  const [voices, setVoices] = useState<SpeechVoice[]>([]);
  const [currentVoice, setCurrentVoice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Check support
  useEffect(() => {
    if (typeof speechSynthesis !== "undefined") {
      setIsSupported(true);
    } else {
      setIsSupported(false);
    }
  }, []);

  // Load voices
  useEffect(() => {
    if (!isSupported) return;

    const loadVoices = () => {
      const availableVoices = speechSynthesis.getVoices();
      const voiceList: SpeechVoice[] = availableVoices.map((voice) => ({
        name: voice.name,
        lang: voice.lang,
        default: voice.default,
        localService: voice.localService,
      }));
      setVoices(voiceList);

      // Set default voice for language
      if (!currentVoice && language) {
        const langVoice = availableVoices.find((v) =>
          v.lang.startsWith(language.split("-")[0])
        );
        if (langVoice) {
          setCurrentVoice(langVoice.name);
        }
      }
    };

    loadVoices();

    // Chrome loads voices asynchronously
    speechSynthesis.addEventListener("voiceschanged", loadVoices);

    return () => {
      speechSynthesis.removeEventListener("voiceschanged", loadVoices);
    };
  }, [isSupported, language, currentVoice]);

  // Set voice from options
  useEffect(() => {
    if (voiceName && voices.some((v) => v.name === voiceName)) {
      setCurrentVoice(voiceName);
    }
  }, [voiceName, voices]);

  const speak = useCallback(
    (text: string) => {
      if (!isSupported || !text.trim()) return;

      // Stop any current speech
      speechSynthesis.cancel();

      setError(null);
      setIsPending(true);

      const utterance = new SpeechSynthesisUtterance(text) as ExtendedSpeechSynthesisUtterance;
      utterance.lang = language;
      utterance.rate = rate;
      utterance.pitch = pitch;
      utterance.volume = volume;

      // Set voice if specified
      if (currentVoice) {
        const voice = speechSynthesis.getVoices().find((v) => v.name === currentVoice);
        if (voice) {
          utterance.voice = voice;
        }
      }

      utterance.onstart = () => {
        setIsSpeaking(true);
        setIsPending(false);
        setIsPaused(false);
        onStart?.();
      };

      utterance.onend = () => {
        setIsSpeaking(false);
        setIsPaused(false);
        setIsPending(false);
        onEnd?.();
      };

      utterance.onerror = (event) => {
        setIsSpeaking(false);
        setIsPaused(false);
        setIsPending(false);
        setError(event.error);
        onError?.(event.error);
      };

      utterance.onpause = () => {
        setIsPaused(true);
        onPause?.();
      };

      utterance.onresume = () => {
        setIsPaused(false);
        onResume?.();
      };

      utteranceRef.current = utterance;
      speechSynthesis.speak(utterance);
    },
    [
      isSupported,
      language,
      rate,
      pitch,
      volume,
      currentVoice,
      onStart,
      onEnd,
      onError,
      onPause,
      onResume,
    ]
  );

  const stop = useCallback(() => {
    if (!isSupported) return;
    speechSynthesis.cancel();
    setIsSpeaking(false);
    setIsPaused(false);
    setIsPending(false);
  }, [isSupported]);

  const pause = useCallback(() => {
    if (!isSupported || !isSpeaking) return;
    speechSynthesis.pause();
    setIsPaused(true);
  }, [isSupported, isSpeaking]);

  const resume = useCallback(() => {
    if (!isSupported || !isSpeaking || !isPaused) return;
    speechSynthesis.resume();
    setIsPaused(false);
  }, [isSupported, isSpeaking, isPaused]);

  const setVoice = useCallback((voiceName: string) => {
    setCurrentVoice(voiceName);
  }, []);

  const cancel = useCallback(() => {
    if (!isSupported) return;
    speechSynthesis.cancel();
    setIsSpeaking(false);
    setIsPaused(false);
    setIsPending(false);
    utteranceRef.current = null;
  }, [isSupported]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      speechSynthesis.cancel();
    };
  }, []);

  return {
    isSupported,
    isSpeaking,
    isPaused,
    isPending,
    voices,
    currentVoice,
    error,
    speak,
    stop,
    pause,
    resume,
    setVoice,
    cancel,
  };
}

export default useSpeechSynthesis;