// ───────────────────────────────────────────────────────────────────────────
//  Author: https://github.com/chrisgrieser
//  Source: https://github.com/chrisgrieser/.config/blob/main/obsidian/vimrc/vimrc-jsfile.js
//  Vimrc Support Docs: https://github.com/esm7/obsidian-vimrc-support/blob/master/JsSnippets.md
// ────────────────────────────────────────────────────────────────────────────

//──────────────────────────────────────────────────────────────────────────────
// CONFIGURATION & UTILITIES
//──────────────────────────────────────────────────────────────────────────────

// Toggle line numbers in the editor.
function toggleLineNumbers() {
  const vault = view.app.vault;
  vault.setConfig("showLineNumber", !vault.getConfig("showLineNumber"));
}

// Clear all notices (pop-up messages) in Obsidian.
function clearNotices() {
  const allNotices = activeDocument.body.getElementsByClassName("notice");
  for (const el of allNotices) el.hide();
}

// Delete the last non-whitespace character in the current line.
function deleteLastChar() {
  const cursor = editor.getCursor();
  const updatedText = editor.getLine(cursor.line).replace(/\S\s*$/, "");
  editor.setLine(cursor.line, updatedText);
  editor.setCursor(cursor);
}

// Check for and update all community plugins.
async function updatePlugins() {
  const app = view.app;
  new Notice("Checking for updates…");
  await app.plugins.checkForUpdates();

  setTimeout(() => {
    const updateCount = Object.keys(app.plugins.updates).length;
    if (updateCount > 0) {
      app.setting.open();
      app.setting.openTabById("community-plugins");
      app.setting.activeTab.containerEl.findAll(".mod-cta").last().click();
    }
  }, 1200); // Prevents race conditions
}

// Freeze Obsidian UI for debugging after a short delay.
function freezeInterface() {
  const delaySecs = 4;
  new Notice(`Will freeze Obsidian in ${delaySecs}s`, delaySecs * 1000);
  electronWindow.openDevTools(); // Required for debugger to work

  setTimeout(
    () => {
      debugger;
    },
    delaySecs * 1000 + 200,
  );
}

// Cycle through installed themes, including the default.
function cycleColorscheme() {
  const app = view.app;
  const currentTheme = app.customCss.theme;
  const installedThemes = Object.keys(app.customCss.themes);
  if (installedThemes.length === 0) return;

  installedThemes.push(""); // Include default theme
  const nextTheme =
    installedThemes[
      (installedThemes.indexOf(currentTheme) + 1) % installedThemes.length
    ] || "";
  app.customCss.setTheme(nextTheme);
}

// Open Obsidian's Appearance settings and auto-scroll to snippets.
function openAppearanceSettings() {
  const setting = view.app.setting;
  setting.open();
  setting.openTabById("appearance");
  setting.activeTab.containerEl.scrollTop =
    setting.activeTab.containerEl.scrollHeight;
}

// Open the Community Plugins settings.
function openCommunityPluginsSettings() {
  view.app.setting.open();
  view.app.setting.openTabById("community-plugins");
}

// Open the Dynamic Highlights plugin settings and auto-focus query input.
// function openDynamicHighlightsSettings() {
//   const setting = view.app.setting;
//   setting.open();
//   setting.openTabById("obsidian-dynamic-highlights");

//   setTimeout(() => {
//     setting.activeTab.containerEl
//       .find(".highlighter-container")
//       .find(".mod-cta")
//       .click();
//     const input = setting.activeTab.containerEl
//       .find(".query-wrapper")
//       .find("input");
//     input.focus();
//     input.scrollLeft = 0;
//     input.setSelectionRange(0, 0);
//   }, 100); // Ensures elements are loaded before interaction
// }

//──────────────────────────────────────────────────────────────────────────────
// CURSOR MOVEMENT & JUMPLIST
//──────────────────────────────────────────────────────────────────────────────

/**
 * Move cursor and add the position to Vim's jump list.
 * @param {Editor} editor - The editor instance.
 * @param {EditorPosition} oldCursor - Previous cursor position.
 * @param {EditorPosition} newCursor - New cursor position.
 */
function _setCursorAndAddToJumplist(editor, oldCursor, newCursor) {
  editor.setCursor(newCursor);

  // Add jump point for Vim mode
  activeWindow.CodeMirrorAdapter.Vim.getVimGlobalState_().jumpList.add(
    editor.cm.cm,
    oldCursor,
    newCursor,
  );
}

/**
 * Jump to the next or previous line containing a specific pattern.
 * @param {"next"|"prev"} direction - Search forward or backward.
 * @param {RegExp} pattern - Regular expression to match.
 */
function gotoLineWithPattern(direction, pattern) {
  const reverseLnum = (line) => editor.lineCount() - line - 1;

  const prevCursor = editor.getCursor();
  let currentLnum = prevCursor.line;
  if (direction === "prev") currentLnum = reverseLnum(currentLnum);

  const allLines = editor.getValue().split("\n");
  if (direction === "prev") allLines.reverse();

  const linesBelow = allLines.slice(currentLnum + 1);
  const linesAbove = allLines.slice(0, currentLnum);

  let lnumWithPattern = linesBelow.findIndex((line) => pattern.test(line));
  if (lnumWithPattern > -1) lnumWithPattern += currentLnum + 1;

  if (lnumWithPattern === -1) {
    lnumWithPattern = linesAbove.findIndex((line) => pattern.test(line));
  }

  if (lnumWithPattern === -1) {
    new Notice(`No line found with pattern ${pattern}`);
    return;
  }

  if (direction === "prev") lnumWithPattern = reverseLnum(lnumWithPattern);
  _setCursorAndAddToJumplist(editor, prevCursor, {
    line: lnumWithPattern,
    ch: 0,
  });
}

//──────────────────────────────────────────────────────────────────────────────
// NAVIGATION & EDITING UTILITIES
//──────────────────────────────────────────────────────────────────────────────

// Jump to the last link (`[[wikilink]]`) in the file.
function gotoLastLinkInFile() {
  const pattern = "[[";
  const lastOccurrence = editor.getValue().lastIndexOf(pattern);
  if (lastOccurrence === -1) {
    new Notice("No links found in this file.");
    return;
  }

  const prevCursor = editor.getCursor();
  const newCursor = editor.offsetToPos(lastOccurrence);
  _setCursorAndAddToJumplist(editor, prevCursor, newCursor);
}

/**
 * Increase or decrease heading level (h1 → h2, h2 → h3, etc.).
 * @param {1|-1} dir - Direction: 1 to increase, -1 to decrease.
 */
function headingIncrementor(dir) {
  const { line: lnum, ch: col } = editor.getCursor();
  const curLine = editor.getLine(lnum);
  const cleanLine = curLine.replace(/^- |\*\*|__/g, ""); // Remove Markdown formatting.

  let updatedLine = cleanLine.replace(/^#* /, (match) => {
    if (dir === -1 && match !== "# ") return match.slice(1);
    if (dir === 1 && match !== "###### ") return "#" + match;
    return ""; // Remove heading if decreasing from h1.
  });

  if (updatedLine === cleanLine) {
    updatedLine = (dir === 1 ? "## " : "###### ") + cleanLine;
  }

  editor.setLine(lnum, updatedLine);
  const diff = updatedLine.length - curLine.length;
  editor.setCursor(lnum, col + diff); // Preserve cursor position.
}

/**
 * Open a new line above or below while respecting list, quote, and task formatting.
 * @param {"above"|"below"} where - Where to insert the new line.
 */
function smartOpenLine(where) {
  const lnum = editor.getCursor().line;
  const curLine = editor.getLine(lnum);

  // Preserve formatting (lists, quotes, tasks).
  let [indentAndText] = curLine.match(/^\s*>+ /) || // Blockquote
    curLine.match(/^\s*- \[[x ]\] /) || // Task list
    curLine.match(/^\s*[-*+] /) || // Unordered list
    curLine.match(/^\s*\d+[.)] /) || // Ordered list
    curLine.match(/^\s*/) || [""]; // Generic indentation.

  // Increment ordered list numbers.
  const orderedList = indentAndText.match(/\d+/)?.[0];
  if (orderedList) {
    const incremented = (Number.parseInt(orderedList) + 1).toString();
    indentAndText = indentAndText.replace(/\d+/, incremented);
  }

  const targetLine = where === "above" ? lnum : lnum + 1;
  const atEndOfFile = editor.lastLine() === lnum && where === "below";
  const extraNewline = atEndOfFile ? "\n" : "";

  editor.replaceRange(extraNewline + indentAndText + "\n", {
    line: targetLine,
    ch: 0,
  });

  editor.setCursor(targetLine, indentAndText.length);
  activeWindow.CodeMirrorAdapter.Vim.enterInsertMode(editor.cm.cm); // Enter insert mode (`a`).
}

// Merge the current line with the next line,
// stripping indentation, lists, and blockquotes.
function smartMerge() {
  const lnum = editor.getCursor().line;
  const curLine = editor.getLine(lnum);
  const nextLine = editor.getLine(lnum + 1);

  // Trim spaces from the current line.
  const curLineCleaned = curLine.replace(/ +$/, "");

  // Remove list, blockquote, and indentation from the next line.
  const nextLineCleaned = nextLine
    .replace(/^\s*- \[[x ]\] /, "") // Task list
    .replace(/^\s*[-*+] /, "") // Unordered list
    .replace(/^\s*>+ /, "") // Blockquote
    .replace(/^\s*\d+[.)] /, "") // Ordered list
    .trim();

  const mergedLine = curLineCleaned + " " + nextLineCleaned;

  const prevCursor = editor.getCursor(); // Preserve cursor position.
  editor.replaceRange(
    mergedLine,
    { line: lnum, ch: 0 },
    { line: lnum + 1, ch: nextLine.length },
  );
  editor.setCursor(prevCursor);
}

//──────────────────────────────────────────────────────────────────────────────
// COPY & PATH UTILITIES
//──────────────────────────────────────────────────────────────────────────────

/**
 * Copy different segments of the current file's path.
 * @param {"absolute"|"relative"|"filename"|"parent"} segment - The type of path to copy.
 */
function copyPathSegment(segment) {
  let toCopy;
  if (segment === "absolute")
    toCopy = view.app.vault.adapter.getFullPath(view.file.path);
  else if (segment === "relative") toCopy = view.file.path;
  else if (segment === "filename") toCopy = view.file.name;
  else if (segment === "parent") toCopy = view.file.parent.path;
  else toCopy = "Invalid segment argument";

  navigator.clipboard.writeText(toCopy);
  new Notice(`Copied ${segment}:\n` + toCopy);
}

// Copy the current file's Obsidian URI as a Markdown link.
function copyObsidianUriMdLink() {
  const app = view.app;
  const activeFile = app.workspace.getActiveFile();
  if (!activeFile) return;

  const filePathEnc = encodeURIComponent(activeFile.path);
  const basename = activeFile.basename;
  const vaultName = app.vault.getName();
  const vaultNameEnc = encodeURIComponent(vaultName);

  const obsidianUri = `obsidian://open?vault=${vaultNameEnc}&file=${filePathEnc}`;
  const mdLink = `[${basename} (${vaultName})](${obsidianUri})`;

  navigator.clipboard.writeText(mdLink);
  new Notice("Copied Obsidian URI:\n" + basename);
}

// Toggle a word between lowercase and title case.
function toggleLowercaseTitleCase() {
  const cursor = editor.getCursor();
  const { from, to } = editor.wordAt(cursor);
  const word = editor.getRange(from, to);

  // Toggle the first letter case, make the rest lowercase
  const newFirstChar =
    word[0] === word[0].toUpperCase()
      ? word[0].toLowerCase()
      : word[0].toUpperCase();
  const newWord = newFirstChar + word.slice(1).toLowerCase();

  editor.replaceRange(newWord, from, to);
  editor.setCursor(cursor); // Restore cursor position
}

//──────────────────────────────────────────────────────────────────────────────
// LINK UTILITIES
//──────────────────────────────────────────────────────────────────────────────

/**
 * Open the next link in the file, seeking forward if the cursor isn't already on one.
 * @param {"current-tab"|"new-tab"} where - Whether to open in the current or new tab.
 */
function openNextLink(where) {
  /**
   * Find the first occurrence of a link in a given text.
   * @param {string} text - The text to search for a link.
   * @returns {{start: number, end: number}} - The start and end positions of the link.
   */
  function rangeOfFirstLink(text) {
    const linkRegex =
      /(https?|obsidian):\/\/[^ )]+|\[\[.+?\]\]|\[[^\]]*?\]\(.+?\)/;
    const linkMatch = text.match(linkRegex);
    if (!linkMatch || linkMatch.index === undefined)
      return { start: -1, end: -1 };
    return {
      start: linkMatch.index,
      end: linkMatch.index + linkMatch[0].length,
    };
  }

  // Check if the cursor is already on a link
  const cursor = editor.getCursor();
  const fullLine = editor.getLine(cursor.line);
  let linkStart, linkEnd;
  let posInLine = 0;
  let cursorIsOnLink = false;

  while (true) {
    const { start, end } = rangeOfFirstLink(fullLine.slice(posInLine));
    if (start === -1 && end === -1) break;
    linkStart = posInLine + start;
    linkEnd = posInLine + end;
    cursorIsOnLink = linkStart <= cursor.ch && linkEnd >= cursor.ch;
    if (cursorIsOnLink) break;
    posInLine += end;
  }

  // If not on a link, seek forward
  if (!cursorIsOnLink) {
    const offset = editor.posToOffset(cursor);
    const textAfterCursor = editor.getValue().slice(offset);
    const linkAfterCursorOffset = rangeOfFirstLink(textAfterCursor).start;
    if (linkAfterCursorOffset === -1) {
      new Notice("No link found.");
      return;
    }
    const linkPosition = editor.offsetToPos(offset + linkAfterCursorOffset);
    linkPosition.ch++; // Adjust for Obsidian's off-by-one link detection
    _setCursorAndAddToJumplist(editor, cursor, linkPosition);
  }

  const commandId =
    where === "new-tab" ? "open-link-in-new-leaf" : "follow-link";
  view.app.commands.executeCommandById("editor:" + commandId);
}

/**
 * Accept or reject changes from OpenAI-based rephraser tool.
 * @param {"accept"|"reject"} mode - Whether to accept or reject the suggested changes.
 */
function highlightsAndStrikethrus(mode) {
  const { line: lnum, ch: col } = editor.getCursor();
  const lineText = editor.getLine(lnum);

  // Remove highlights (==text==) and strikethroughs (~~text~~) based on mode
  let updatedLine =
    mode === "accept"
      ? lineText.replace(/==/g, "").replace(/~~.*?~~/g, "")
      : lineText.replace(/~~/g, "").replace(/==.*?==/g, "");

  updatedLine = updatedLine.replaceAll("  ", " "); // Remove double spaces from markup removal
  editor.setLine(lnum, updatedLine);

  const charsLess = lineText.length - updatedLine.length;
  editor.setCursor({ line: lnum, ch: col - charsLess });
}

/**
 * Save or load a workspace using the Workspaces Core Plugin.
 * Enables the plugin temporarily and disables it after execution.
 * @param {"load"|"save"} action - Whether to load or save the workspace.
 * @param {string} workspaceName - The name of the workspace.
 */
async function workspace(action, workspaceName) {
  const workspacePlugin = view.app.internalPlugins.plugins.workspaces;
  await workspacePlugin.enable();

  if (action === "load") {
    workspacePlugin.instance.loadWorkspace(workspaceName);
  } else if (action === "save") {
    workspacePlugin.instance.saveWorkspace(workspaceName);
  }

  new Notice(
    `${action === "load" ? "Loaded" : "Saved"} workspace "${workspaceName}".`,
  );

  // Disable plugin after a short delay to minimize performance impact
  setTimeout(() => workspacePlugin.disable(), 3000);
}

//──────────────────────────────────────────────────────────────────────────────
// FOLDING NAVIGATION (Origami-like behavior)
//──────────────────────────────────────────────────────────────────────────────

// Navigate left or toggle fold if at the beginning of the line.
// CAVEAT: Slightly breaks `h` and `l` behavior in tables.
function origamiH() {
  const isAtBoL = editor.getCursor().ch === 0; // Check if at beginning of line
  const action = isAtBoL ? "toggleFold" : "goLeft";
  editor.exec(action);
}

// Navigate right or toggle fold if the current line is folded.
function origamiL() {
  const currentLn = editor.getCursor().line;
  const folds = editor.getFoldOffsets(); // Get all fold positions
  const foldedLines = [...folds].map(
    (offset) => editor.offsetToPos(offset).line,
  );

  const action = foldedLines.includes(currentLn) ? "toggleFold" : "goRight";
  editor.exec(action);
}

//──────────────────────────────────────────────────────────────────────────────
// DATAVIEW-JS UTILITIES
//──────────────────────────────────────────────────────────────────────────────

// Toggle `//` comment at the beginning of a line.
// If the line is already commented, it removes the `//`.
function toggleJsLineComment() {
  const cursor = editor.getCursor();
  const lineText = editor.getLine(cursor.line);

  // Extract indentation, comment markers, and actual content
  const [_, indent, hasComment, textWithoutComment] =
    lineText.match(/^(\s*)(\/\/ )?(.*)/) || [];

  // Toggle comment prefix
  const updatedText = indent + (hasComment ? "" : "// ") + textWithoutComment;
  cursor.ch += hasComment ? -3 : 3; // Adjust cursor position

  editor.setLine(cursor.line, updatedText);
  editor.setCursor(cursor);
}

// Append a `//` comment at the end of the current line.
// Moves cursor to the end, ready for text input.
function appendJsComment() {
  const lnum = editor.getCursor().line;
  const text = editor.getLine(lnum);
  const updatedText = text + " // ";
  editor.setLine(lnum, updatedText);
  editor.setCursor(lnum, updatedText.length);

  // Enter insert mode at the end of the line
  activeWindow.CodeMirrorAdapter.Vim.enterInsertMode(editor.cm.cm);
}

// Insert a `console.log(variable)` statement for the word under the cursor.
// Moves cursor to the next line, maintaining proper indentation.
function consoleLogFromWordUnderCursor() {
  const cursor = editor.getCursor();
  const cursorWordRange = editor.wordAt(cursor);
  const cursorWord = editor.getRange(cursorWordRange.from, cursorWordRange.to);
  const indent = editor.getLine(cursor.line).match(/^\s*/)?.[0] || "";
  const logLine = indent + `console.log(${cursorWord});`;

  // Insert console.log below the current line
  editor.replaceRange(logLine + "\n", { line: cursor.line + 1, ch: 0 });

  // Restore cursor position
  editor.setCursor(cursor);
}
