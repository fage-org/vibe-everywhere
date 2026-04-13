/**
 * Tool Result Renderers
 *
 * This module provides specialized renderers for different tool types.
 * Each renderer is designed to present tool-specific output in a user-friendly way.
 *
 * Usage:
 * 1. Import the specific renderer directly:
 *    import { BashToolResult } from './tool-results';
 *
 * 2. Or use the registry to get a renderer by tool name:
 *    const Renderer = getToolResultRenderer('Bash');
 */

import type { ReactNode } from "react";
import { BashToolResult, type BashToolResultProps } from "./BashToolResult";
import {
  FileReadToolResult,
  type FileReadToolResultProps,
} from "./FileReadToolResult";
import { EditToolResult, type EditToolResultProps } from "./EditToolResult";
import {
  SearchToolResult,
  type SearchToolResultProps,
  type SearchMatch,
} from "./SearchToolResult";
import {
  ToolResultSection,
  type ToolResultSectionProps,
} from "./ToolResultSection";

// Re-export individual components
export {
  BashToolResult,
  FileReadToolResult,
  EditToolResult,
  SearchToolResult,
  ToolResultSection,
};

// Re-export types
export type {
  BashToolResultProps,
  FileReadToolResultProps,
  EditToolResultProps,
  SearchToolResultProps,
  SearchMatch,
  ToolResultSectionProps,
};

/**
 * Common props for all tool result renderers
 */
export interface ToolResultBaseProps {
  /** Tool call ID */
  toolCallId?: string;
  /** Tool name */
  toolName: string;
  /** Tool arguments */
  input: Record<string, unknown>;
  /** Tool execution result */
  result?: unknown;
  /** Execution status */
  status?: "pending" | "running" | "completed" | "error";
  /** Error message if failed */
  error?: string;
}

/**
 * Type for tool result renderer components
 */
export type ToolResultRenderer = (props: ToolResultBaseProps) => ReactNode;

/**
 * Registry mapping tool names to their result renderers
 *
 * Tool names can vary by agent:
 * - Claude Code: Bash, Read, Edit, Glob, Grep
 * - Codex: CodexBash, CodexPatch, CodexDiff
 * - Gemini: execute, edit (lowercase)
 */
export const toolResultRegistry: Record<string, ToolResultRenderer> = {
  // Claude Code tools
  Bash: BashToolResultRenderer,
  Read: FileReadToolResultRenderer,
  Edit: EditToolResultRenderer,
  Glob: SearchToolResultRenderer,
  Grep: SearchToolResultRenderer,

  // Codex tools
  CodexBash: BashToolResultRenderer,
  CodexPatch: EditToolResultRenderer,
  CodexDiff: EditToolResultRenderer,

  // Gemini tools (lowercase)
  execute: BashToolResultRenderer,
  execute_bash: BashToolResultRenderer,
};

/**
 * Get a tool result renderer by tool name
 *
 * @param toolName - Name of the tool
 * @returns The renderer function or null if not found
 */
export function getToolResultRenderer(
  toolName: string,
): ToolResultRenderer | null {
  return toolResultRegistry[toolName] || null;
}

/**
 * Check if a tool has a specialized result renderer
 *
 * @param toolName - Name of the tool
 * @returns True if a renderer exists
 */
export function hasToolResultRenderer(toolName: string): boolean {
  return toolName in toolResultRegistry;
}

// ============================================================================
// Renderer Wrappers
// ============================================================================

/**
 * Bash tool result renderer wrapper
 */
function BashToolResultRenderer(props: ToolResultBaseProps): ReactNode {
  const { input, result, error, status } = props;

  // Parse result structure
  let stdout: string | undefined;
  let stderr: string | undefined;
  let exitCode: number | undefined;

  if (result && typeof result === "object") {
    const r = result as Record<string, unknown>;
    stdout = typeof r.stdout === "string" ? r.stdout : undefined;
    stderr = typeof r.stderr === "string" ? r.stderr : undefined;
    exitCode = typeof r.exit_code === "number" ? r.exit_code : undefined;
  } else if (typeof result === "string") {
    stdout = result;
  }

  return (
    <BashToolResult
      command={
        typeof input.command === "string" ? input.command : String(input)
      }
      stdout={stdout}
      stderr={stderr}
      error={status === "error" ? error : undefined}
      exitCode={exitCode}
      cwd={typeof input.cwd === "string" ? input.cwd : undefined}
    />
  );
}

/**
 * File read tool result renderer wrapper
 */
function FileReadToolResultRenderer(props: ToolResultBaseProps): ReactNode {
  const { input, result } = props;

  const filePath =
    typeof input.file_path === "string"
      ? input.file_path
      : typeof input.path === "string"
        ? input.path
        : "unknown";

  const content =
    typeof result === "string" ? result : JSON.stringify(result, null, 2);

  return (
    <FileReadToolResult
      filePath={filePath}
      content={content}
      showLineNumbers={true}
    />
  );
}

/**
 * Edit tool result renderer wrapper
 */
function EditToolResultRenderer(props: ToolResultBaseProps): ReactNode {
  const { input, result } = props;

  const filePath =
    typeof input.file_path === "string"
      ? input.file_path
      : typeof input.path === "string"
        ? input.path
        : "unknown";

  const oldString =
    typeof input.old_string === "string" ? input.old_string : "";
  const newString =
    typeof input.new_string === "string" ? input.new_string : "";

  // Check if result is a diff
  const diffText =
    typeof result === "string" && result.includes("@@") ? result : undefined;

  return (
    <EditToolResult
      filePath={filePath}
      oldString={oldString}
      newString={newString}
      diffText={diffText}
    />
  );
}

/**
 * Search tool result renderer wrapper
 */
function SearchToolResultRenderer(props: ToolResultBaseProps): ReactNode {
  const { input, result, toolName } = props;

  const query =
    typeof input.pattern === "string"
      ? input.pattern
      : typeof input.query === "string"
        ? input.query
        : typeof input.glob === "string"
          ? input.glob
          : "";

  // Parse search results
  const matches: Array<{
    filePath: string;
    lineNumber: number;
    line: string;
    match?: string;
  }> = [];

  if (result && typeof result === "object" && Array.isArray(result)) {
    result.forEach((item) => {
      if (typeof item === "object" && item !== null) {
        const r = item as Record<string, unknown>;
        if (typeof r.file_path === "string" || typeof r.path === "string") {
          matches.push({
            filePath:
              typeof r.file_path === "string"
                ? r.file_path
                : (r.path as string),
            lineNumber: typeof r.line_number === "number" ? r.line_number : 1,
            line: typeof r.line === "string" ? r.line : "",
            match: typeof r.match === "string" ? r.match : undefined,
          });
        }
      }
    });
  }

  return (
    <SearchToolResult
      query={query}
      matches={matches}
      searchType={toolName.toLowerCase() as "glob" | "grep" | "search"}
    />
  );
}
