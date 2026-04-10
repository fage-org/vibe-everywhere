import { type ReactNode } from "react";
import { tokens } from "../../design-system/tokens";
import {
  Title1,
  Title2,
  Title3,
  Body,
  Subheadline,
  Footnote,
} from "../ui/Typography";

export interface MarkdownRendererProps {
  /** Markdown content to render */
  content: string;
  /** Whether to enable soft line breaks */
  softBreaks?: boolean;
  /** Custom link click handler */
  onLinkClick?: (url: string) => void;
}

/**
 * MarkdownRenderer - Markdown content rendering
 *
 * Matches Happy's markdown styling:
 * - Proper heading hierarchy
 * - Styled lists and code blocks
 * - Link handling
 * - Blockquotes and horizontal rules
 */
export function MarkdownRenderer({
  content,
  softBreaks = true,
  onLinkClick,
}: MarkdownRendererProps) {
  const elements = parseMarkdown(content, softBreaks);

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: tokens.spacing[4],
        color: "var(--text-primary)",
        lineHeight: tokens.typography.lineHeight.relaxed,
      }}
    >
      {elements.map((element, index) => (
        <MarkdownElement key={index} element={element} onLinkClick={onLinkClick} />
      ))}
    </div>
  );
}

// Markdown AST types
type MarkdownNode =
  | { type: "heading"; level: number; content: InlineNode[] }
  | { type: "paragraph"; content: InlineNode[] }
  | { type: "codeBlock"; language?: string; content: string }
  | { type: "blockquote"; content: MarkdownNode[] }
  | { type: "list"; ordered: boolean; items: MarkdownNode[][] }
  | { type: "horizontalRule" }
  | { type: "lineBreak" };

type InlineNode =
  | { type: "text"; content: string }
  | { type: "bold"; content: InlineNode[] }
  | { type: "italic"; content: InlineNode[] }
  | { type: "code"; content: string }
  | { type: "link"; url: string; content: InlineNode[] }
  | { type: "strikethrough"; content: InlineNode[] };

interface MarkdownElementProps {
  element: MarkdownNode;
  onLinkClick?: (url: string) => void;
}

function MarkdownElement({ element, onLinkClick }: MarkdownElementProps) {
  switch (element.type) {
    case "heading":
      return <MarkdownHeading level={element.level} content={element.content} onLinkClick={onLinkClick} />;
    case "paragraph":
      return (
        <Body>
          <InlineContent nodes={element.content} onLinkClick={onLinkClick} />
        </Body>
      );
    case "codeBlock":
      return <MarkdownCodeBlock language={element.language} content={element.content} />;
    case "blockquote":
      return (
        <blockquote
          style={{
            margin: 0,
            paddingLeft: tokens.spacing[4],
            borderLeft: `3px solid var(--border-primary)`,
            color: "var(--text-secondary)",
          }}
        >
          {element.content.map((child, index) => (
            <MarkdownElement key={index} element={child} onLinkClick={onLinkClick} />
          ))}
        </blockquote>
      );
    case "list":
      return (
        <MarkdownList ordered={element.ordered} items={element.items} onLinkClick={onLinkClick} />
      );
    case "horizontalRule":
      return (
        <hr
          style={{
            border: "none",
            borderTop: "1px solid var(--border-primary)",
            margin: `${tokens.spacing[4]} 0`,
          }}
        />
      );
    case "lineBreak":
      return <br />;
    default:
      return null;
  }
}

interface MarkdownHeadingProps {
  level: number;
  content: InlineNode[];
  onLinkClick?: (url: string) => void;
}

function MarkdownHeading({ level, content, onLinkClick }: MarkdownHeadingProps) {
  const text = <InlineContent nodes={content} onLinkClick={onLinkClick} />;

  switch (level) {
    case 1:
      return <Title1>{text}</Title1>;
    case 2:
      return <Title2>{text}</Title2>;
    case 3:
      return <Title3>{text}</Title3>;
    default:
      return (
        <h4
          style={{
            margin: 0,
            fontSize: tokens.typography.fontSize.lg,
            fontWeight: tokens.typography.fontWeight.semibold,
            color: "var(--text-primary)",
          }}
        >
          {text}
        </h4>
      );
  }
}

interface MarkdownCodeBlockProps {
  language?: string;
  content: string;
}

function MarkdownCodeBlock({ language, content }: MarkdownCodeBlockProps) {
  return (
    <div
      style={{
        backgroundColor: "var(--surface-secondary)",
        borderRadius: tokens.radii.lg,
        overflow: "hidden",
      }}
    >
      {language && (
        <div
          style={{
            padding: `${tokens.spacing[2]} ${tokens.spacing[3]}`,
            backgroundColor: "var(--surface-tertiary)",
            borderBottom: "1px solid var(--border-primary)",
          }}
        >
          <Footnote color="tertiary">{language}</Footnote>
        </div>
      )}
      <pre
        style={{
          margin: 0,
          padding: tokens.spacing[3],
          overflow: "auto",
          fontFamily: tokens.typography.fontFamily.mono,
          fontSize: tokens.typography.fontSize.sm,
          lineHeight: tokens.typography.lineHeight.normal,
        }}
      >
        <code style={{ color: "var(--text-primary)" }}>{content}</code>
      </pre>
    </div>
  );
}

interface MarkdownListProps {
  ordered: boolean;
  items: MarkdownNode[][];
  onLinkClick?: (url: string) => void;
}

function MarkdownList({ ordered, items, onLinkClick }: MarkdownListProps) {
  const ListTag = ordered ? "ol" : "ul";

  return (
    <ListTag
      style={
        {
          margin: 0,
          paddingLeft: tokens.spacing[6],
          display: "flex",
          flexDirection: "column",
          gap: tokens.spacing[2],
        } as React.CSSProperties
      }
    >
      {items.map((itemContent, index) => (
        <li key={index}>
          {itemContent.map((node, nodeIndex) => (
            <MarkdownElement key={nodeIndex} element={node} onLinkClick={onLinkClick} />
          ))}
        </li>
      ))}
    </ListTag>
  );
}

interface InlineContentProps {
  nodes: InlineNode[];
  onLinkClick?: (url: string) => void;
}

function InlineContent({ nodes, onLinkClick }: InlineContentProps): ReactNode {
  return (
    <>
      {nodes.map((node, index) => {
        switch (node.type) {
          case "text":
            return <span key={index}>{node.content}</span>;
          case "bold":
            return (
              <strong key={index} style={{ fontWeight: tokens.typography.fontWeight.bold }}>
                <InlineContent nodes={node.content} onLinkClick={onLinkClick} />
              </strong>
            );
          case "italic":
            return (
              <em key={index} style={{ fontStyle: "italic" }}>
                <InlineContent nodes={node.content} onLinkClick={onLinkClick} />
              </em>
            );
          case "code":
            return (
              <code
                key={index}
                style={{
                  fontFamily: tokens.typography.fontFamily.mono,
                  fontSize: "0.9em",
                  padding: `${tokens.spacing[0.5]} ${tokens.spacing[1]}`,
                  backgroundColor: "var(--surface-secondary)",
                  borderRadius: tokens.radii.sm,
                }}
              >
                {node.content}
              </code>
            );
          case "link":
            return (
              <a
                key={index}
                href={node.url}
                onClick={(e) => {
                  if (onLinkClick) {
                    e.preventDefault();
                    onLinkClick(node.url);
                  }
                }}
                style={{
                  color: "var(--color-primary)",
                  textDecoration: "underline",
                }}
              >
                <InlineContent nodes={node.content} onLinkClick={onLinkClick} />
              </a>
            );
          case "strikethrough":
            return (
              <del key={index} style={{ textDecoration: "line-through" }}>
                <InlineContent nodes={node.content} onLinkClick={onLinkClick} />
              </del>
            );
          default:
            return null;
        }
      })}
    </>
  );
}

/**
 * Simple markdown parser
 * Note: For production, consider using a library like marked or remark
 */
function parseMarkdown(content: string, softBreaks: boolean): MarkdownNode[] {
  const lines = content.split("\n");
  const nodes: MarkdownNode[] = [];
  let i = 0;

  while (i < lines.length) {
    const line = lines[i];

    // Empty line
    if (!line.trim()) {
      if (softBreaks && nodes.length > 0) {
        nodes.push({ type: "lineBreak" });
      }
      i++;
      continue;
    }

    // Heading
    const headingMatch = line.match(/^(#{1,6})\s+(.+)$/);
    if (headingMatch) {
      nodes.push({
        type: "heading",
        level: headingMatch[1].length,
        content: parseInline(headingMatch[2]),
      });
      i++;
      continue;
    }

    // Code block
    if (line.startsWith("```")) {
      const language = line.slice(3).trim();
      const codeLines: string[] = [];
      i++;
      while (i < lines.length && !lines[i].startsWith("```")) {
        codeLines.push(lines[i]);
        i++;
      }
      nodes.push({
        type: "codeBlock",
        language: language || undefined,
        content: codeLines.join("\n"),
      });
      i++;
      continue;
    }

    // Horizontal rule
    if (/^(---|___|\*\*\*)$/.test(line.trim())) {
      nodes.push({ type: "horizontalRule" });
      i++;
      continue;
    }

    // Blockquote
    if (line.startsWith("> ")) {
      const quoteLines: string[] = [];
      while (i < lines.length && lines[i].startsWith("> ")) {
        quoteLines.push(lines[i].slice(2));
        i++;
      }
      nodes.push({
        type: "blockquote",
        content: parseMarkdown(quoteLines.join("\n"), softBreaks),
      });
      continue;
    }

    // List
    const listMatch = line.match(/^(\s*)([-*+]|\d+\.)\s+(.+)$/);
    if (listMatch) {
      const isOrdered = /^\d+\./.test(listMatch[2]);
      const items: MarkdownNode[][] = [];

      while (i < lines.length) {
        const itemMatch = lines[i].match(/^(\s*)([-*+]|\d+\.)\s+(.+)$/);
        if (!itemMatch) break;

        const itemContent: MarkdownNode[] = [];
        itemContent.push({
          type: "paragraph",
          content: parseInline(itemMatch[3]),
        });
        i++;

        // Collect continuation lines
        while (i < lines.length && lines[i].startsWith(" ") && !lines[i].match(/^(\s*)([-*+]|\d+\.)\s/)) {
          itemContent.push({
            type: "paragraph",
            content: parseInline(lines[i].trim()),
          });
          i++;
        }

        items.push(itemContent);
      }

      nodes.push({
        type: "list",
        ordered: isOrdered,
        items,
      });
      continue;
    }

    // Paragraph
    const paraLines: string[] = [line];
    i++;
    while (i < lines.length && lines[i].trim() && !lines[i].match(/^(#{1,6}\s|>|```|\s*[-*+]|\s*\d+\.|---|___|\*\*\*)/)) {
      paraLines.push(lines[i]);
      i++;
    }

    nodes.push({
      type: "paragraph",
      content: parseInline(paraLines.join(" ")),
    });
  }

  return nodes;
}

function parseInline(text: string): InlineNode[] {
  const nodes: InlineNode[] = [];
  let remaining = text;

  const patterns = [
    { regex: /\*\*\*(.+?)\*\*\*/g, type: "boldItalic" as const },
    { regex: /\*\*(.+?)\*\*/g, type: "bold" as const },
    { regex: /\*(.+?)\*/g, type: "italic" as const },
    { regex: /`(.+?)`/g, type: "code" as const },
    { regex: /~~(.+?)~~/g, type: "strikethrough" as const },
    { regex: /\[(.+?)\]\((.+?)\)/g, type: "link" as const },
  ];

  while (remaining) {
    let earliestMatch: { index: number; match: RegExpMatchArray; type: string } | null = null;

    for (const pattern of patterns) {
      const match = pattern.regex.exec(remaining);
      if (match && (!earliestMatch || match.index < earliestMatch.index)) {
        earliestMatch = { index: match.index, match, type: pattern.type };
      }
    }

    if (!earliestMatch) {
      nodes.push({ type: "text", content: remaining });
      break;
    }

    // Add text before match
    if (earliestMatch.index > 0) {
      nodes.push({ type: "text", content: remaining.slice(0, earliestMatch.index) });
    }

    // Add matched content
    switch (earliestMatch.type) {
      case "bold":
        nodes.push({
          type: "bold",
          content: parseInline(earliestMatch.match[1]),
        });
        break;
      case "italic":
        nodes.push({
          type: "italic",
          content: parseInline(earliestMatch.match[1]),
        });
        break;
      case "code":
        nodes.push({ type: "code", content: earliestMatch.match[1] });
        break;
      case "strikethrough":
        nodes.push({
          type: "strikethrough",
          content: parseInline(earliestMatch.match[1]),
        });
        break;
      case "link":
        nodes.push({
          type: "link",
          url: earliestMatch.match[2],
          content: parseInline(earliestMatch.match[1]),
        });
        break;
    }

    remaining = remaining.slice(earliestMatch.index + earliestMatch.match[0].length);
  }

  return nodes;
}
