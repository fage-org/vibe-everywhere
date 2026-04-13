import { useState, useEffect, useCallback, useRef } from "react";

/**
 * Speech Recognition State
 */
export interface SpeechRecognitionState {
  /** Whether speech recognition is supported */
  isSupported: boolean;
  /** Whether currently listening */
  isListening: boolean;
  /** Whether permission has been granted */
  hasPermission: boolean | null;
  /** Transcribed text */
  transcript: string;
  /** Interim (partial) transcript */
  interimTranscript: string;
  /** Error message if any */
  error: string | null;
  /** Whether currently processing speech */
  isProcessing: boolean;
}

/**
 * Speech Recognition Options
 */
export interface SpeechRecognitionOptions {
  /** Language for recognition (BCP 47 code, e.g., 'en-US', 'zh-CN') */
  language?: string;
  /** Whether to return interim results */
  interimResults?: boolean;
  /** Whether to keep listening after a result */
  continuous?: boolean;
  /** Maximum alternatives to return */
  maxAlternatives?: number;
  /** Callback when transcript is finalized */
  onResult?: (transcript: string) => void;
  /** Callback on error */
  onError?: (error: string) => void;
  /** Callback when permission status changes */
  onPermissionChange?: (granted: boolean) => void;
}

/**
 * Web Speech API type declarations
 */
interface SpeechRecognitionEvent extends Event {
  readonly resultIndex: number;
  readonly results: SpeechRecognitionResultList;
}

interface SpeechRecognitionResultList {
  readonly length: number;
  item(index: number): SpeechRecognitionResult;
  [index: number]: SpeechRecognitionResult;
}

interface SpeechRecognitionResult {
  readonly isFinal: boolean;
  readonly length: number;
  item(index: number): SpeechRecognitionAlternative;
  [index: number]: SpeechRecognitionAlternative;
}

interface SpeechRecognitionAlternative {
  readonly transcript: string;
  readonly confidence: number;
}

interface SpeechRecognitionErrorEvent extends Event {
  readonly error: string;
  readonly message: string;
}

interface SpeechRecognition extends EventTarget {
  continuous: boolean;
  interimResults: boolean;
  lang: string;
  maxAlternatives: number;
  onaudioend: ((this: SpeechRecognition, ev: Event) => void) | null;
  onaudiostart: ((this: SpeechRecognition, ev: Event) => void) | null;
  onend: ((this: SpeechRecognition, ev: Event) => void) | null;
  onerror: ((this: SpeechRecognition, ev: SpeechRecognitionErrorEvent) => void) | null;
  onnomatch: ((this: SpeechRecognition, ev: Event) => void) | null;
  onresult: ((this: SpeechRecognition, ev: SpeechRecognitionEvent) => void) | null;
  onsoundend: ((this: SpeechRecognition, ev: Event) => void) | null;
  onsoundstart: ((this: SpeechRecognition, ev: Event) => void) | null;
  onspeechend: ((this: SpeechRecognition, ev: Event) => void) | null;
  onspeechstart: ((this: SpeechRecognition, ev: Event) => void) | null;
  onstart: ((this: SpeechRecognition, ev: Event) => void) | null;
  abort(): void;
  start(): void;
  stop(): void;
}

interface SpeechRecognitionConstructor {
  new (): SpeechRecognition;
}

interface WindowWithSpeechRecognition extends Window {
  SpeechRecognition?: SpeechRecognitionConstructor;
  webkitSpeechRecognition?: SpeechRecognitionConstructor;
}

/**
 * Hook for speech recognition (Speech-to-Text)
 *
 * Uses the Web Speech API for browser-native speech recognition.
 * Works on Chrome, Edge, Safari, and Chromium-based Tauri desktop apps.
 *
 * @param options - Configuration options
 * @returns Speech recognition state and controls
 *
 * @example
 * ```tsx
 * const {
 *   isSupported,
 *   isListening,
 *   transcript,
 *   startListening,
 *   stopListening,
 *   resetTranscript,
 * } = useSpeechRecognition({
 *   language: 'en-US',
 *   onResult: (text) => console.log('Final:', text),
 * });
 *
 * if (!isSupported) {
 *   return <p>Voice input not supported</p>;
 * }
 *
 * return (
 *   <button onClick={isListening ? stopListening : startListening}>
 *     {isListening ? 'Stop' : 'Start'} Recording
 *   </button>
 * );
 * ```
 */
export function useSpeechRecognition(
  options: SpeechRecognitionOptions = {}
): SpeechRecognitionState & {
  /** Start listening for speech */
  startListening: () => void;
  /** Stop listening */
  stopListening: () => void;
  /** Toggle listening state */
  toggleListening: () => void;
  /** Reset the transcript */
  resetTranscript: () => void;
  /** Request microphone permission */
  requestPermission: () => Promise<boolean>;
} {
  const {
    language = "en-US",
    interimResults = true,
    continuous = false,
    maxAlternatives = 1,
    onResult,
    onError,
    onPermissionChange,
  } = options;

  const recognitionRef = useRef<SpeechRecognition | null>(null);
  const [isSupported, setIsSupported] = useState(false);
  const [isListening, setIsListening] = useState(false);
  const [hasPermission, setHasPermission] = useState<boolean | null>(null);
  const [transcript, setTranscript] = useState("");
  const [interimTranscript, setInterimTranscript] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isProcessing, setIsProcessing] = useState(false);

  // Check support and initialize recognition
  useEffect(() => {
    const win = window as WindowWithSpeechRecognition;
    const SpeechRecognitionAPI =
      win.SpeechRecognition || win.webkitSpeechRecognition;

    if (!SpeechRecognitionAPI) {
      setIsSupported(false);
      return;
    }

    setIsSupported(true);

    const recognition = new SpeechRecognitionAPI();
    recognition.continuous = continuous;
    recognition.interimResults = interimResults;
    recognition.lang = language;
    recognition.maxAlternatives = maxAlternatives;

    recognition.onstart = () => {
      setIsListening(true);
      setError(null);
      setIsProcessing(true);
    };

    recognition.onend = () => {
      setIsListening(false);
      setIsProcessing(false);
    };

    recognition.onresult = (event: SpeechRecognitionEvent) => {
      let finalTranscript = "";
      let interim = "";

      for (let i = event.resultIndex; i < event.results.length; i++) {
        const result = event.results[i];
        if (result.isFinal) {
          finalTranscript += result[0].transcript;
        } else {
          interim += result[0].transcript;
        }
      }

      if (finalTranscript) {
        setTranscript((prev) => prev + finalTranscript);
        onResult?.(finalTranscript);
      }
      setInterimTranscript(interim);
    };

    recognition.onerror = (event: SpeechRecognitionErrorEvent) => {
      const errorMessage = getErrorMessage(event.error);
      setError(errorMessage);
      setIsListening(false);
      setIsProcessing(false);
      onError?.(errorMessage);

      if (event.error === "not-allowed") {
        setHasPermission(false);
        onPermissionChange?.(false);
      }
    };

    recognitionRef.current = recognition;

    return () => {
      recognition.abort();
    };
  }, [language, continuous, interimResults, maxAlternatives, onResult, onError, onPermissionChange]);

  // Check permission status
  useEffect(() => {
    if (!isSupported) return;

    const checkPermission = async () => {
      try {
        // Try to query permission status
        const result = await navigator.permissions.query({
          name: "microphone" as PermissionName,
        });
        setHasPermission(result.state === "granted");

        result.addEventListener("change", () => {
          const granted = result.state === "granted";
          setHasPermission(granted);
          onPermissionChange?.(granted);
        });
      } catch {
        // Permissions API not supported, will check on first use
      }
    };

    checkPermission();
  }, [isSupported, onPermissionChange]);

  const startListening = useCallback(() => {
    if (!recognitionRef.current || isListening) return;

    setError(null);
    setInterimTranscript("");

    try {
      recognitionRef.current.start();
    } catch (err) {
      // Recognition might already be running
      console.warn("Speech recognition start error:", err);
    }
  }, [isListening]);

  const stopListening = useCallback(() => {
    if (!recognitionRef.current || !isListening) return;

    try {
      recognitionRef.current.stop();
    } catch (err) {
      console.warn("Speech recognition stop error:", err);
    }
  }, [isListening]);

  const toggleListening = useCallback(() => {
    if (isListening) {
      stopListening();
    } else {
      startListening();
    }
  }, [isListening, startListening, stopListening]);

  const resetTranscript = useCallback(() => {
    setTranscript("");
    setInterimTranscript("");
    setError(null);
  }, []);

  const requestPermission = useCallback(async (): Promise<boolean> => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: true,
      });
      stream.getTracks().forEach((track) => track.stop());
      setHasPermission(true);
      onPermissionChange?.(true);
      return true;
    } catch (err) {
      setHasPermission(false);
      onPermissionChange?.(false);
      return false;
    }
  }, [onPermissionChange]);

  return {
    isSupported,
    isListening,
    hasPermission,
    transcript,
    interimTranscript,
    error,
    isProcessing,
    startListening,
    stopListening,
    toggleListening,
    resetTranscript,
    requestPermission,
  };
}

/**
 * Get human-readable error message
 */
function getErrorMessage(error: string): string {
  const messages: Record<string, string> = {
    "not-allowed": "Microphone access denied. Please allow microphone access.",
    "no-speech": "No speech detected. Please try again.",
    aborted: "Speech recognition was aborted.",
    "audio-capture": "No microphone found. Please connect a microphone.",
    network: "Network error occurred. Please check your connection.",
    "not-supported": "Speech recognition not supported in this browser.",
    "service-not-allowed": "Speech recognition service not allowed.",
    "language-not-supported": "Language not supported.",
    "grammar-not-supported": "Grammar not supported.",
  };

  return messages[error] || `Speech recognition error: ${error}`;
}

export default useSpeechRecognition;
