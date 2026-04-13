import { useEffect, useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Body, Caption1 } from "./Typography";
import { tokens } from "../../design-system/tokens";
import type { Command } from "../../hooks/useCommandPalette";

export interface CommandPaletteProps {
  /** Whether the palette is open */
  isOpen: boolean;
  /** Close the palette */
  close: () => void;
  /** Current search query */
  query: string;
  /** Update search query */
  setQuery: (query: string) => void;
  /** Filtered commands */
  commands: Command[];
  /** Selected command index */
  selectedIndex: number;
  /** Move selection up */
  selectPrevious: () => void;
  /** Move selection down */
  selectNext: () => void;
  /** Execute the selected command */
  executeSelected: () => void;
}

/**
 * CommandPalette - Keyboard-first command menu
 *
 * A modal overlay that provides quick access to commands via keyboard.
 * Triggered by Cmd+K (Mac) or Ctrl+K (Windows/Linux).
 *
 * Features:
 * - Fuzzy search across commands
 * - Keyboard navigation (arrows, enter, escape)
 * - Grouped by category
 * - Keyboard shortcut hints
 */
export function CommandPalette({
  isOpen,
  close,
  query,
  setQuery,
  commands,
  selectedIndex,
  selectPrevious,
  selectNext,
  executeSelected,
}: CommandPaletteProps) {
  const { t } = useTranslation("ui");
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  // Focus input when opened
  useEffect(() => {
    if (isOpen) {
      // Small delay to ensure the modal is rendered
      const timer = setTimeout(() => {
        inputRef.current?.focus();
      }, 10);
      return () => clearTimeout(timer);
    }
  }, [isOpen]);

  // Scroll selected item into view
  useEffect(() => {
    if (listRef.current) {
      const selectedElement = listRef.current.querySelector(`[data-index="${selectedIndex}"]`);
      if (selectedElement) {
        selectedElement.scrollIntoView({ block: "nearest" });
      }
    }
  }, [selectedIndex]);

  // Handle keyboard events
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      switch (e.key) {
        case "ArrowUp":
          e.preventDefault();
          selectPrevious();
          break;
        case "ArrowDown":
          e.preventDefault();
          selectNext();
          break;
        case "Enter":
          e.preventDefault();
          executeSelected();
          break;
        case "Escape":
          e.preventDefault();
          close();
          break;
      }
    },
    [selectPrevious, selectNext, executeSelected, close],
  );

  // Handle click outside
  const handleBackdropClick = useCallback(
    (e: React.MouseEvent) => {
      if (e.target === e.currentTarget) {
        close();
      }
    },
    [close],
  );

  if (!isOpen) {
    return null;
  }

  // Group commands by category
  const groupedCommands = commands.reduce(
    (acc, cmd) => {
      const category = cmd.category;
      if (!acc[category]) {
        acc[category] = [];
      }
      acc[category].push(cmd);
      return acc;
    },
    {} as Record<string, Command[]>,
  );

  const categoryOrder: Command["category"][] = ["navigation", "actions", "settings", "help"];
  const categoryLabels: Record<string, string> = {
    navigation: t("commandPalette.categories.navigation"),
    actions: t("commandPalette.categories.actions"),
    settings: t("commandPalette.categories.settings"),
    help: t("commandPalette.categories.help"),
  };

  let globalIndex = 0;

  return (
    <div
      style={{
        position: "fixed",
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: "rgba(0, 0, 0, 0.5)",
        display: "flex",
        alignItems: "flex-start",
        justifyContent: "center",
        paddingTop: "15vh",
        zIndex: 9999,
      }}
      onClick={handleBackdropClick}
    >
      <div
        style={{
          width: "100%",
          maxWidth: "560px",
          maxHeight: "70vh",
          backgroundColor: "var(--bg-primary)",
          borderRadius: tokens.radii.lg,
          boxShadow: "0 25px 50px -12px rgba(0, 0, 0, 0.25)",
          overflow: "hidden",
          display: "flex",
          flexDirection: "column",
        }}
        onKeyDown={handleKeyDown}
      >
        {/* Search Input */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: tokens.spacing[3],
            padding: tokens.spacing[4],
            borderBottom: "1px solid var(--border-primary)",
          }}
        >
          <span style={{ fontSize: "20px", color: "var(--text-tertiary)" }}>🔍</span>
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder={t("commandPalette.searchPlaceholder")}
            style={{
              flex: 1,
              border: "none",
              outline: "none",
              backgroundColor: "transparent",
              fontSize: tokens.typography.fontSize.lg,
              color: "var(--text-primary)",
            }}
          />
          <div
            style={{
              display: "flex",
              gap: tokens.spacing[1],
            }}
          >
            <kbd
              style={{
                padding: `${tokens.spacing[1]} ${tokens.spacing[2]}`,
                backgroundColor: "var(--surface-secondary)",
                borderRadius: tokens.radii.sm,
                fontSize: tokens.typography.fontSize.xs,
                color: "var(--text-tertiary)",
                fontFamily: tokens.typography.fontFamily.mono,
              }}
            >
              ESC
            </kbd>
          </div>
        </div>

        {/* Commands List */}
        <div
          ref={listRef}
          style={{
            flex: 1,
            overflow: "auto",
            padding: `${tokens.spacing[2]} 0`,
          }}
        >
          {commands.length === 0 ? (
            <div
              style={{
                padding: tokens.spacing[8],
                textAlign: "center",
                color: "var(--text-tertiary)",
              }}
            >
              <Body color="tertiary">{t("commandPalette.noResults")}</Body>
            </div>
          ) : (
            categoryOrder.map((category) => {
              const categoryCommands = groupedCommands[category];
              if (!categoryCommands || categoryCommands.length === 0) return null;

              return (
                <div key={category}>
                  {/* Category Header */}
                  <div
                    style={{
                      padding: `${tokens.spacing[2]} ${tokens.spacing[4]}`,
                      borderBottom: "1px solid var(--border-secondary)",
                    }}
                  >
                    <Caption1 color="tertiary" style={{ textTransform: "uppercase", letterSpacing: "0.05em" }}>
                      {categoryLabels[category] || category}
                    </Caption1>
                  </div>

                  {/* Category Commands */}
                  {categoryCommands.map((command) => {
                    const currentIndex = globalIndex++;
                    const isSelected = currentIndex === selectedIndex;

                    return (
                      <div
                        key={command.id}
                        data-index={currentIndex}
                        onClick={() => {
                          command.action();
                          close();
                        }}
                        style={{
                          display: "flex",
                          alignItems: "center",
                          gap: tokens.spacing[3],
                          padding: `${tokens.spacing[3]} ${tokens.spacing[4]}`,
                          cursor: "pointer",
                          backgroundColor: isSelected ? "var(--surface-secondary)" : "transparent",
                          transition: `background-color ${tokens.animation.duration.fast} ${tokens.animation.easing.default}`,
                        }}
                        onMouseEnter={(e) => {
                          e.currentTarget.style.backgroundColor = "var(--surface-secondary)";
                        }}
                        onMouseLeave={(e) => {
                          e.currentTarget.style.backgroundColor = isSelected
                            ? "var(--surface-secondary)"
                            : "transparent";
                        }}
                      >
                        {/* Icon */}
                        {command.icon && (
                          <span style={{ fontSize: "20px", width: "24px", textAlign: "center" }}>
                            {command.icon}
                          </span>
                        )}

                        {/* Label and Description */}
                        <div style={{ flex: 1, minWidth: 0 }}>
                          <Body style={{ fontWeight: isSelected ? 600 : 400 }}>{command.label}</Body>
                          {command.description && (
                            <Caption1 color="tertiary" style={{ marginTop: tokens.spacing[1] }}>
                              {command.description}
                            </Caption1>
                          )}
                        </div>

                        {/* Shortcut */}
                        {command.shortcut && (
                          <kbd
                            style={{
                              padding: `${tokens.spacing[1]} ${tokens.spacing[2]}`,
                              backgroundColor: "var(--surface-tertiary)",
                              borderRadius: tokens.radii.sm,
                              fontSize: tokens.typography.fontSize.xs,
                              color: "var(--text-tertiary)",
                              fontFamily: tokens.typography.fontFamily.mono,
                            }}
                          >
                            {command.shortcut}
                          </kbd>
                        )}
                      </div>
                    );
                  })}
                </div>
              );
            })
          )}
        </div>

        {/* Footer */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            padding: tokens.spacing[3],
            borderTop: "1px solid var(--border-primary)",
            backgroundColor: "var(--surface-secondary)",
          }}
        >
          <div style={{ display: "flex", gap: tokens.spacing[4] }}>
            <div style={{ display: "flex", alignItems: "center", gap: tokens.spacing[1] }}>
              <kbd style={kbdStyle}>↑</kbd>
              <kbd style={kbdStyle}>↓</kbd>
              <Caption1 color="tertiary">{t("commandPalette.toNavigate")}</Caption1>
            </div>
            <div style={{ display: "flex", alignItems: "center", gap: tokens.spacing[1] }}>
              <kbd style={kbdStyle}>↵</kbd>
              <Caption1 color="tertiary">{t("commandPalette.toSelect")}</Caption1>
            </div>
          </div>
          <Caption1 color="tertiary">{t("commandPalette.toClose")}</Caption1>
        </div>
      </div>
    </div>
  );
}

const kbdStyle: React.CSSProperties = {
  padding: "2px 6px",
  backgroundColor: "var(--surface-tertiary)",
  borderRadius: tokens.radii.sm,
  fontSize: tokens.typography.fontSize.xs,
  color: "var(--text-tertiary)",
  fontFamily: tokens.typography.fontFamily.mono,
};

export default CommandPalette;