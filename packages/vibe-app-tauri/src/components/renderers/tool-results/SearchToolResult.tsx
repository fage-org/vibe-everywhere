import { useMemo } from "react";
import { tokens } from "../../../design-system/tokens";
import { Caption1, Body } from "../../ui/Typography";
import { ToolResultSection } from "./ToolResultSection";

export interface SearchMatch {
  /** File path containing the match */
  filePath: string;
  /** Line number of the match */
  lineNumber: number;
  /** Column number of the match (optional) */
  columnNumber?: number;
  /** The matched line content */
  line: string;
  /** Lines before the match (context) */
  beforeContext?: string[];
  /** Lines after the match (context) */
  afterContext?: string[];
  /** The matched portion (for highlighting) */
  match?: string;
}

export interface SearchToolResultProps {
  /** Search query/pattern */
  query: string;
  /** Search matches */
  matches: SearchMatch[];
  /** Number of context lines to show */
  contextLines?: number;
  /** Whether to show line numbers */
  showLineNumbers?: boolean;
  /** Optional search type label */
  searchType?: "glob" | "grep" | "search";
}

/**
 * SearchToolResult - Renders search/grep results
 *
 * Features:
 * - Grouped by file
 * - Line number display
 * - Match highlighting
 * - Context lines support
 */
export function SearchToolResult({
  query,
  matches,
  contextLines = 0,
  showLineNumbers = true,
  searchType = "search",
}: SearchToolResultProps) {
  // Group matches by file
  const groupedMatches = useMemo(() => {
    const groups = new Map<string, SearchMatch[]>();

    matches.forEach((match) => {
      const existing = groups.get(match.filePath) || [];
      groups.set(match.filePath, [...existing, match]);
    });

    return Array.from(groups.entries()).map(([filePath, fileMatches]) => ({
      filePath,
      matches: fileMatches,
    }));
  }, [matches]);

  const totalMatches = matches.length;
  const totalFiles = groupedMatches.length;

  return (
    <ToolResultSection>
      {/* Search Summary */}
      <div
        style={{
          padding: tokens.spacing[3],
          backgroundColor: "var(--surface-secondary)",
          borderRadius: tokens.radii.md,
          marginBottom: tokens.spacing[3],
        }}
      >
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: tokens.spacing[2],
          }}
        >
          <Caption1 color="tertiary">
            {searchType === "glob"
              ? "Glob"
              : searchType === "grep"
                ? "Grep"
                : "Search"}
            :
          </Caption1>
          <Body
            style={{
              fontFamily: tokens.typography.fontFamily.mono,
              color: "var(--text-primary)",
            }}
          >
            {query}
          </Body>
        </div>
        <Caption1 color="tertiary" style={{ marginTop: tokens.spacing[1] }}>
          {totalMatches} match{totalMatches !== 1 ? "es" : ""} in {totalFiles}{" "}
          file
          {totalFiles !== 1 ? "s" : ""}
        </Caption1>
      </div>

      {/* No Results */}
      {totalMatches === 0 && (
        <div
          style={{
            padding: tokens.spacing[4],
            textAlign: "center",
            color: "var(--text-tertiary)",
          }}
        >
          <Body>No matches found</Body>
        </div>
      )}

      {/* Results by File */}
      {groupedMatches.map(({ filePath, matches: fileMatches }) => (
        <FileMatchGroup
          key={filePath}
          filePath={filePath}
          matches={fileMatches}
          contextLines={contextLines}
          showLineNumbers={showLineNumbers}
        />
      ))}
    </ToolResultSection>
  );
}

interface FileMatchGroupProps {
  filePath: string;
  matches: SearchMatch[];
  contextLines: number;
  showLineNumbers: boolean;
}

function FileMatchGroup({
  filePath,
  matches,
  contextLines,
  showLineNumbers,
}: FileMatchGroupProps) {
  return (
    <div
      style={{
        marginBottom: tokens.spacing[3],
        border: "1px solid var(--border-primary)",
        borderRadius: tokens.radii.md,
        overflow: "hidden",
      }}
    >
      {/* File Header */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: `${tokens.spacing[2]} ${tokens.spacing[3]}`,
          backgroundColor: "var(--surface-tertiary)",
          borderBottom: "1px solid var(--border-primary)",
        }}
      >
        <Caption1
          color="secondary"
          style={{
            fontFamily: tokens.typography.fontFamily.mono,
          }}
        >
          {filePath}
        </Caption1>
        <Caption1 color="tertiary">{matches.length} matches</Caption1>
      </div>

      {/* Match Lines */}
      <div
        style={{
          backgroundColor: "var(--surface-secondary)",
        }}
      >
        {matches.map((match, index) => (
          <MatchLine
            key={`${filePath}-${match.lineNumber}-${index}`}
            match={match}
            contextLines={contextLines}
            showLineNumbers={showLineNumbers}
          />
        ))}
      </div>
    </div>
  );
}

interface MatchLineProps {
  match: SearchMatch;
  contextLines: number;
  showLineNumbers: boolean;
}

function MatchLine({ match, contextLines, showLineNumbers }: MatchLineProps) {
  const {
    lineNumber,
    line,
    beforeContext,
    afterContext,
    match: matchText,
  } = match;

  // Highlight the matched portion
  const highlightedLine = useMemo(() => {
    if (!matchText) return line;

    const index = line.indexOf(matchText);
    if (index === -1) return line;

    return (
      <>
        {line.slice(0, index)}
        <span
          style={{
            backgroundColor: "rgba(255, 214, 0, 0.3)",
            color: "var(--text-primary)",
            fontWeight: tokens.typography.fontWeight.medium,
          }}
        >
          {matchText}
        </span>
        {line.slice(index + matchText.length)}
      </>
    );
  }, [line, matchText]);

  return (
    <div>
      {/* Before Context */}
      {contextLines > 0 &&
        beforeContext?.map((contextLine, idx) => (
          <div
            key={`before-${idx}`}
            style={{
              display: "flex",
              padding: `0 ${tokens.spacing[3]}`,
              backgroundColor: "var(--surface-secondary)",
            }}
          >
            {showLineNumbers && (
              <span
                style={{
                  width: "50px",
                  color: "var(--text-quaternary)",
                  textAlign: "right",
                  paddingRight: tokens.spacing[2],
                  userSelect: "none",
                  fontFamily: tokens.typography.fontFamily.mono,
                  fontSize: tokens.typography.fontSize.xs,
                }}
              >
                {lineNumber - beforeContext.length + idx}
              </span>
            )}
            <pre
              style={{
                margin: 0,
                color: "var(--text-tertiary)",
                fontFamily: tokens.typography.fontFamily.mono,
                fontSize: tokens.typography.fontSize.sm,
                lineHeight: tokens.typography.lineHeight.snug,
              }}
            >
              {contextLine}
            </pre>
          </div>
        ))}

      {/* Match Line */}
      <div
        style={{
          display: "flex",
          padding: `${tokens.spacing[1]} ${tokens.spacing[3]}`,
          backgroundColor: "rgba(255, 214, 0, 0.1)",
        }}
      >
        {showLineNumbers && (
          <span
            style={{
              width: "50px",
              color: "var(--color-warning)",
              textAlign: "right",
              paddingRight: tokens.spacing[2],
              userSelect: "none",
              fontFamily: tokens.typography.fontFamily.mono,
              fontSize: tokens.typography.fontSize.xs,
            }}
          >
            {lineNumber}
          </span>
        )}
        <pre
          style={{
            margin: 0,
            color: "var(--text-primary)",
            fontFamily: tokens.typography.fontFamily.mono,
            fontSize: tokens.typography.fontSize.sm,
            lineHeight: tokens.typography.lineHeight.snug,
          }}
        >
          {highlightedLine}
        </pre>
      </div>

      {/* After Context */}
      {contextLines > 0 &&
        afterContext?.map((contextLine, idx) => (
          <div
            key={`after-${idx}`}
            style={{
              display: "flex",
              padding: `0 ${tokens.spacing[3]}`,
              backgroundColor: "var(--surface-secondary)",
            }}
          >
            {showLineNumbers && (
              <span
                style={{
                  width: "50px",
                  color: "var(--text-quaternary)",
                  textAlign: "right",
                  paddingRight: tokens.spacing[2],
                  userSelect: "none",
                  fontFamily: tokens.typography.fontFamily.mono,
                  fontSize: tokens.typography.fontSize.xs,
                }}
              >
                {lineNumber + idx + 1}
              </span>
            )}
            <pre
              style={{
                margin: 0,
                color: "var(--text-tertiary)",
                fontFamily: tokens.typography.fontFamily.mono,
                fontSize: tokens.typography.fontSize.sm,
                lineHeight: tokens.typography.lineHeight.snug,
              }}
            >
              {contextLine}
            </pre>
          </div>
        ))}
    </div>
  );
}
