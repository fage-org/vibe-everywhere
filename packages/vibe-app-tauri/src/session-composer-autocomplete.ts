export type ComposerSuggestion = {
  key: string;
  label: string;
  description: string;
  insertText: string;
  prefix: "@" | "/";
};

type ActiveComposerToken = {
  activeWord: string;
  offset: number;
  endOffset: number;
};

const STOP_CHARACTERS = ["\n", ",", "(", ")", "[", "]", "{", "}", "<", ">", ";", "!", "?"];

const COMMAND_SUGGESTIONS = [
  {
    key: "command-plan",
    label: "/plan",
    description: "Ask the agent to outline the next steps before editing.",
    insertText: "/plan",
    prefix: "/" as const,
  },
  {
    key: "command-review",
    label: "/review",
    description: "Request a focused review of the current session work.",
    insertText: "/review",
    prefix: "/" as const,
  },
  {
    key: "command-test",
    label: "/test",
    description: "Ask the agent to run or explain the relevant test path.",
    insertText: "/test",
    prefix: "/" as const,
  },
  {
    key: "command-explain",
    label: "/explain",
    description: "Have the agent explain a selected change or result.",
    insertText: "/explain",
    prefix: "/" as const,
  },
] satisfies ReadonlyArray<ComposerSuggestion>;

function findActiveWordStart(content: string, cursorPosition: number, prefixes: string[]): number {
  let startIndex = cursorPosition - 1;
  let spaceIndex = -1;

  while (startIndex >= 0) {
    const character = content.charAt(startIndex);

    if (character === " ") {
      if (spaceIndex >= 0) {
        return spaceIndex + 1;
      }
      spaceIndex = startIndex;
      startIndex -= 1;
      continue;
    }

    if (
      prefixes.includes(character) &&
      (startIndex === 0 ||
        content.charAt(startIndex - 1) === " " ||
        content.charAt(startIndex - 1) === "\n")
    ) {
      return startIndex;
    }

    if (STOP_CHARACTERS.includes(character)) {
      return startIndex + 1;
    }

    startIndex -= 1;
  }

  return (spaceIndex >= 0 ? spaceIndex : startIndex) + 1;
}

function findActiveWordEnd(content: string, cursorPosition: number, wordStart: number): number {
  let endIndex = cursorPosition;
  const isFilePath = content.charAt(wordStart) === "@";

  while (endIndex < content.length) {
    const character = content.charAt(endIndex);
    if (isFilePath && (character === "/" || character === ".")) {
      endIndex += 1;
      continue;
    }
    if (character === " " || STOP_CHARACTERS.includes(character)) {
      break;
    }
    endIndex += 1;
  }

  return endIndex;
}

export function findActiveComposerToken(
  content: string,
  selectionStart: number,
  selectionEnd = selectionStart,
  prefixes: Array<"@" | "/"> = ["@", "/"],
): ActiveComposerToken | null {
  if (selectionStart !== selectionEnd || selectionStart === 0) {
    return null;
  }

  const startIndex = findActiveWordStart(content, selectionStart, prefixes);
  const activeWord = content.substring(startIndex, selectionEnd);
  if (!activeWord) {
    return null;
  }

  if (!prefixes.includes(activeWord.charAt(0) as "@" | "/")) {
    return null;
  }

  const endOffset = findActiveWordEnd(content, selectionEnd, startIndex);
  return {
    activeWord,
    offset: startIndex,
    endOffset,
  };
}

export function buildComposerSuggestions(
  activeWord: string | null,
  files: string[],
): ComposerSuggestion[] {
  if (!activeWord) {
    return [];
  }

  if (activeWord.startsWith("/")) {
    const query = activeWord.slice(1).toLowerCase();
    return COMMAND_SUGGESTIONS.filter((suggestion) =>
      suggestion.label.slice(1).toLowerCase().includes(query),
    );
  }

  if (activeWord.startsWith("@")) {
    const query = activeWord.slice(1).toLowerCase();
    return files
      .filter((relativePath) => relativePath.toLowerCase().includes(query))
      .slice(0, 6)
      .map((relativePath) => ({
        key: `file-${relativePath}`,
        label: `@${relativePath}`,
        description: "Reference this workspace file in the next prompt.",
        insertText: `@${relativePath}`,
        prefix: "@" as const,
      }));
  }

  return [];
}

export function applyComposerSuggestion(
  content: string,
  selectionStart: number,
  selectionEnd: number,
  suggestion: ComposerSuggestion,
): {
  text: string;
  cursorPosition: number;
} {
  const activeWord = findActiveComposerToken(content, selectionStart, selectionEnd, [
    suggestion.prefix,
  ]);

  const insertText = `${suggestion.insertText} `;
  if (!activeWord) {
    const text =
      content.slice(0, selectionStart) + insertText + content.slice(selectionEnd);
    return {
      text,
      cursorPosition: selectionStart + insertText.length,
    };
  }

  const text =
    content.slice(0, activeWord.offset) + insertText + content.slice(activeWord.endOffset);
  return {
    text,
    cursorPosition: activeWord.offset + insertText.length,
  };
}
