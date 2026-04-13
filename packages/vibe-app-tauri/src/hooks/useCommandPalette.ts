import { useState, useCallback, useEffect, useMemo } from "react";

/**
 * Command definition for the command palette
 */
export interface Command {
  /** Unique identifier */
  id: string;
  /** Display label */
  label: string;
  /** Optional description */
  description?: string;
  /** Category for grouping */
  category: "navigation" | "actions" | "settings" | "help";
  /** Keyboard shortcut (display only) */
  shortcut?: string;
  /** Icon (emoji or component) */
  icon?: string;
  /** Action to execute */
  action: () => void;
  /** Whether the command is available */
  enabled?: boolean;
}

/**
 * Command palette state and actions
 */
export interface UseCommandPaletteResult {
  /** Whether the palette is open */
  isOpen: boolean;
  /** Open the palette */
  open: () => void;
  /** Close the palette */
  close: () => void;
  /** Toggle the palette */
  toggle: () => void;
  /** Current search query */
  query: string;
  /** Update search query */
  setQuery: (query: string) => void;
  /** All available commands */
  commands: Command[];
  /** Filtered commands based on query */
  filteredCommands: Command[];
  /** Selected command index */
  selectedIndex: number;
  /** Move selection up */
  selectPrevious: () => void;
  /** Move selection down */
  selectNext: () => void;
  /** Execute the selected command */
  executeSelected: () => void;
  /** Register a global keyboard shortcut */
  registerShortcut: () => void;
  /** Unregister the keyboard shortcut */
  unregisterShortcut: () => void;
}

/**
 * Default commands factory
 */
export function createDefaultCommands(navigate: (path: string) => void): Command[] {
  return [
    // Navigation commands
    {
      id: "nav-home",
      label: "Go to Home",
      category: "navigation",
      icon: "🏠",
      action: () => navigate("/(app)/index"),
    },
    {
      id: "nav-inbox",
      label: "Go to Inbox",
      category: "navigation",
      icon: "📬",
      action: () => navigate("/(app)/inbox/index"),
    },
    {
      id: "nav-new-session",
      label: "New Session",
      category: "navigation",
      icon: "➕",
      shortcut: "⌘N",
      action: () => navigate("/(app)/new/index"),
    },
    {
      id: "nav-sessions",
      label: "Recent Sessions",
      category: "navigation",
      icon: "💬",
      action: () => navigate("/(app)/session/recent"),
    },
    {
      id: "nav-artifacts",
      label: "Artifacts",
      category: "navigation",
      icon: "📁",
      action: () => navigate("/(app)/artifacts/index"),
    },
    {
      id: "nav-friends",
      label: "Friends",
      category: "navigation",
      icon: "👥",
      action: () => navigate("/(app)/friends/index"),
    },
    {
      id: "nav-terminal",
      label: "Terminal",
      category: "navigation",
      icon: "💻",
      action: () => navigate("/(app)/terminal/index"),
    },
    // Settings commands
    {
      id: "settings-account",
      label: "Account Settings",
      category: "settings",
      icon: "👤",
      action: () => navigate("/(app)/settings/account"),
    },
    {
      id: "settings-appearance",
      label: "Appearance Settings",
      category: "settings",
      icon: "🎨",
      action: () => navigate("/(app)/settings/appearance"),
    },
    {
      id: "settings-ai-providers",
      label: "AI Provider Settings",
      category: "settings",
      icon: "🤖",
      action: () => navigate("/(app)/settings/ai-providers"),
    },
    {
      id: "settings-voice",
      label: "Voice Settings",
      category: "settings",
      icon: "🎤",
      action: () => navigate("/(app)/settings/voice"),
    },
    {
      id: "settings-usage",
      label: "Usage Statistics",
      category: "settings",
      icon: "📊",
      action: () => navigate("/(app)/settings/usage"),
    },
    {
      id: "settings-language",
      label: "Language Settings",
      category: "settings",
      icon: "🌐",
      action: () => navigate("/(app)/settings/language"),
    },
    // Note: Actions (Search Sessions) and Help (Keyboard Shortcuts)
    // commands are removed until their functionality is implemented.
    // TODO: Add search mode command when search UI is ready
    // TODO: Add keyboard shortcuts modal command when modal is implemented
  ];
}

/**
 * Hook for managing the command palette
 */
export function useCommandPalette(
  commands: Command[],
  options: {
    /** Custom shortcut to open the palette (default: Cmd+K / Ctrl+K) */
    shortcut?: string;
  } = {},
): UseCommandPaletteResult {
  const [isOpen, setIsOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);

  // Filter commands based on query
  const filteredCommands = useMemo(() => {
    if (!query.trim()) {
      return commands.filter((cmd) => cmd.enabled !== false);
    }
    const lowerQuery = query.toLowerCase();
    return commands.filter((cmd) => {
      if (cmd.enabled === false) return false;
      return (
        cmd.label.toLowerCase().includes(lowerQuery) ||
        cmd.description?.toLowerCase().includes(lowerQuery) ||
        cmd.category.toLowerCase().includes(lowerQuery)
      );
    });
  }, [commands, query]);

  // Reset selection when filtered results change
  useEffect(() => {
    setSelectedIndex(0);
  }, [filteredCommands]);

  const open = useCallback(() => {
    setIsOpen(true);
    setQuery("");
    setSelectedIndex(0);
  }, []);

  const close = useCallback(() => {
    setIsOpen(false);
    setQuery("");
    setSelectedIndex(0);
  }, []);

  const toggle = useCallback(() => {
    if (isOpen) {
      close();
    } else {
      open();
    }
  }, [isOpen, open, close]);

  const selectPrevious = useCallback(() => {
    setSelectedIndex((prev) => (prev > 0 ? prev - 1 : filteredCommands.length - 1));
  }, [filteredCommands.length]);

  const selectNext = useCallback(() => {
    setSelectedIndex((prev) => (prev < filteredCommands.length - 1 ? prev + 1 : 0));
  }, [filteredCommands.length]);

  const executeSelected = useCallback(() => {
    const command = filteredCommands[selectedIndex];
    if (command) {
      command.action();
      close();
    }
  }, [filteredCommands, selectedIndex, close]);

  // Global keyboard shortcut handler
  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      // Check for Cmd+K (Mac) or Ctrl+K (Windows/Linux)
      if ((event.metaKey || event.ctrlKey) && event.key === "k") {
        event.preventDefault();
        toggle();
        return;
      }

      // Handle navigation when open
      if (isOpen) {
        switch (event.key) {
          case "Escape":
            event.preventDefault();
            close();
            break;
          case "ArrowUp":
            event.preventDefault();
            selectPrevious();
            break;
          case "ArrowDown":
            event.preventDefault();
            selectNext();
            break;
          case "Enter":
            event.preventDefault();
            executeSelected();
            break;
        }
      }
    },
    [isOpen, toggle, close, selectPrevious, selectNext, executeSelected],
  );

  const registerShortcut = useCallback(() => {
    if (typeof window !== "undefined" && window.addEventListener) {
      window.addEventListener("keydown", handleKeyDown);
    }
  }, [handleKeyDown]);

  const unregisterShortcut = useCallback(() => {
    if (typeof window !== "undefined" && window.removeEventListener) {
      window.removeEventListener("keydown", handleKeyDown);
    }
  }, [handleKeyDown]);

  // Auto-register on mount
  useEffect(() => {
    registerShortcut();
    return () => unregisterShortcut();
  }, [registerShortcut, unregisterShortcut]);

  return {
    isOpen,
    open,
    close,
    toggle,
    query,
    setQuery,
    commands,
    filteredCommands,
    selectedIndex,
    selectPrevious,
    selectNext,
    executeSelected,
    registerShortcut,
    unregisterShortcut,
  };
}

export default useCommandPalette;