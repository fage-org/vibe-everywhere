import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { oneDark } from "react-syntax-highlighter/dist/esm/styles/prism";

const codeThemeStyle = {
  margin: 0,
  borderRadius: "14px",
  background: "rgba(4, 11, 17, 0.92)",
  border: "1px solid rgba(123, 226, 196, 0.12)",
} as const;

export function SyntaxCodeBlock({
  code,
  language,
  showLineNumbers = false,
  wrapLines = false,
}: {
  code: string;
  language: string;
  showLineNumbers?: boolean;
  wrapLines?: boolean;
}) {
  return (
    <SyntaxHighlighter
      language={language || "text"}
      style={oneDark}
      customStyle={codeThemeStyle}
      showLineNumbers={showLineNumbers}
      wrapLongLines={wrapLines}
    >
      {code}
    </SyntaxHighlighter>
  );
}
