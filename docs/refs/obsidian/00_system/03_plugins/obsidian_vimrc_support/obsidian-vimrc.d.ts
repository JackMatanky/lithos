//──────────────────────────────────────────────────────────────────────────────
// OBSIDIAN VIMRC GLOBALS
//──────────────────────────────────────────────────────────────────────────────
// These global objects are available in JavaScript files used by the
// Obsidian vimrc plugin. More details:
// https://github.com/esm7/obsidian-vimrc-support?tab=readme-ov-file#jscommand---jsfunction

declare const selection: EditorSelection; // Current selection in the editor
declare const editor: Editor; // Active editor instance
declare const view: View; // Current view, contains file metadata and app controls

// Notification system for displaying messages in Obsidian
declare class Notice {
    constructor(msg: string, duration?: number);
    setMessage(msg: string): void;
}

//──────────────────────────────────────────────────────────────────────────────
// ELECTRON GLOBALS
//──────────────────────────────────────────────────────────────────────────────
// Provides access to Electron features, including debugging tools and process info.

declare const process: { versions: Record<string, string> }; // Node.js process version info
declare const electronWindow: { openDevTools(): void }; // Opens Electron DevTools

// Electron-specific DOM objects for accessing the document and window
declare const activeDocument: any; // Electron document object (DOM manipulation)
declare const activeWindow: any; // Electron window object (UI interactions)

//──────────────────────────────────────────────────────────────────────────────
// TYPE DECLARATIONS FOR OBSIDIAN OBJECTS
//──────────────────────────────────────────────────────────────────────────────
// These types define various objects available within the Obsidian API.

declare type EditorPosition = { ch: number; line: number }; // Cursor position
declare type EditorRange = { from: EditorPosition; to: EditorPosition }; // Text range
declare type EditorSelection = { head: EditorPosition; anchor: EditorPosition }; // Selected text range
declare type TFile = {
    path: string;
    name: string;
    basename: string;
    parent: TFile;
}; // File object

//──────────────────────────────────────────────────────────────────────────────
// EDITOR API - INTERACTIONS WITH THE TEXT EDITOR
//──────────────────────────────────────────────────────────────────────────────
// Defines methods for interacting with the active text editor in Obsidian.

declare type Editor = {
    exec(action: string): void; // Execute editor command (e.g., move cursor)
    getCursor(): EditorPosition; // Get current cursor position
    setCursor(pos: EditorPosition | number, ch?: number): void; // Move cursor
    wordAt(pos: EditorPosition): EditorRange; // Get word at position
    lineCount(): number; // Get total number of lines
    getValue(): string; // Get full text content
    setValue(value: string): void; // Set full text content
    getFoldOffsets(): number[]; // Get folded sections
    getLine(line: number): string; // Get text of a specific line
    setLine(line: number, text: string): void; // Modify a specific line
    replaceSelection(replacement: string): void; // Replace selected text
    replaceRange(
        replacement: string,
        from: EditorPosition,
        to?: EditorPosition,
        origin?: string,
    ): void; // Replace text in a range
    setSelection(anchor: EditorPosition, head: EditorPosition): void; // Set text selection
    getSelection(): string; // Get selected text
    getRange(from: EditorPosition, to: EditorPosition): string; // Get text within a range
    offsetToPos(offset: number): EditorPosition; // Convert text offset to position
    posToOffset(pos: EditorPosition): number; // Convert position to text offset
    lastLine(): number; // Get last line number
    scrollIntoView(range: EditorRange, center?: boolean): void; // Scroll to a specific range
    cm: any; // CodeMirror instance (for Vim mode)
};

//──────────────────────────────────────────────────────────────────────────────
// VIEW API - INTERACTIONS WITH THE CURRENT FILE AND METADATA
//──────────────────────────────────────────────────────────────────────────────
// The `View` object provides access to the current file and various app functionalities.

declare type View = {
    file: TFile; // Current file in the editor
    app: {
        fileManager: {
            // Process YAML frontmatter in the file
            processFrontMatter(
                file: TFile,
                fn: (
                    frontmatter: Record<string, string | number | boolean>,
                ) => void,
                options?: object,
            ): Promise<void>;
        };
        commands: {
            executeCommandById(id: string): void; // Execute a command by its ID
        };
        metadataCache: {
            // Retrieve file metadata (headings, blocks, frontmatter)
            getFirstLinkpathDest(
                linkpath: string,
                sourcePath: string,
            ): TFile | null;
            getFileCache(file: TFile): {
                headings: {
                    heading: string;
                    position: { start: EditorPosition };
                }[];
                blocks: Record<string, { position: { start: EditorPosition } }>;
                frontmatter: Record<string, string | number | boolean>;
            };
        };
        customCss: {
            theme: string; // Current theme
            themes: Record<string, string>; // Installed themes
            oldThemes: string[]; // Previous themes
            setTheme(theme: string): void; // Change theme
        };
        workspace: {
            protocolHandlers: {
                get(protocol: string): ({ id: string }) => void; // Handle custom protocols
            };
            getLeaf(): {
                openFile(file: TFile): Promise<void>; // Open a file in a new tab
            };
            getActiveFile: () => TFile; // Get currently open file
        };
        openWithDefaultApp(path: string): void; // Open file with system default app
        vault: {
            getName: () => string; // Get vault name
            getConfig(key: string): boolean | string | number; // Get vault setting
            setConfig(key: string, value: boolean | string | number): void; // Modify vault setting
            configDir: string; // Path to config directory
            adapter: {
                getFullPath(path: string): string; // Get absolute path of a file
            };
            getFileByPath(path: string): TFile; // Get file by path
            getMarkdownFiles(): TFile[]; // Get all markdown files
        };
        plugins: {
            checkForUpdates(): Promise<void>; // Check for plugin updates
            updates: Record<string, object>; // Plugin update status
            getPlugin(id: string): any; // Get specific plugin instance
            disablePlugin(id: string): Promise<void>; // Disable a plugin
            enablePlugin(id: string): Promise<void>; // Enable a plugin
        };
        setting: {
            open(): void; // Open settings UI
            openTabById(id: string): void; // Open specific settings tab
            activeTab: any; // Active settings tab instance
        };
        internalPlugins: {
            plugins: {
                workspaces: {
                    disable(): Promise<void>; // Disable workspace plugin
                    enable(): Promise<void>; // Enable workspace plugin
                    instance: {
                        loadWorkspace(name: string): void; // Load a saved workspace
                        saveWorkspace(name: string): void; // Save current workspace
                    };
                };
            };
        };
    };
};
