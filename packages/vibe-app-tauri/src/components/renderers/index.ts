// Content Renderer Components
// For rendering various content types (diffs, markdown, tool output)

export {
  DiffRenderer,
  DiffRendererWithSettings,
  parseUnifiedDiff,
  type DiffRendererProps,
  type DiffFile,
  type DiffHunk,
  type DiffLine,
} from "./DiffRenderer";

export {
  MarkdownRenderer,
  type MarkdownRendererProps,
} from "./MarkdownRenderer";

export {
  ToolRenderer,
  type ToolRendererProps,
  type ToolCall,
  type Tool,
} from "./ToolRenderer";

// Tool Result Renderers
export {
  BashToolResult,
  FileReadToolResult,
  EditToolResult,
  SearchToolResult,
  ToolResultSection,
  toolResultRegistry,
  getToolResultRenderer,
  hasToolResultRenderer,
  type BashToolResultProps,
  type FileReadToolResultProps,
  type EditToolResultProps,
  type SearchToolResultProps,
  type SearchMatch,
  type ToolResultSectionProps,
  type ToolResultBaseProps,
  type ToolResultRenderer,
} from "./tool-results";
