" ========================================================================
"  Author: https://github.com/chrisgrieser
"  Source: https://github.com/chrisgrieser/.config/blob/main/obsidian/vimrc/obsidian-vimrc.vim
" ========================================================================

" REQUIRED: Enable `Support JS commands` in the Vimrc plugin settings.
" This script enhances Vim mode in Obsidian with additional navigation,
" editing, and clipboard functionalities.

"───────────────────────────────────────────────────────────────────────────────
" LEADER KEY CONFIGURATION
"───────────────────────────────────────────────────────────────────────────────

" Can't set leader keys in Obsidian Vim mode, so we must use a consistent key manually.
" To avoid conflicts, we unmap `,` so it doesn't trigger unwanted behaviors.
" Source: https://github.com/esm7/obsidian-vimrc-support#some-help-with-binding-space-chords-doom-and-spacemacs-fans
unmap ,

"───────────────────────────────────────────────────────────────────────────────
" META COMMANDS
"───────────────────────────────────────────────────────────────────────────────

" Open this vimrc file using the system's default text editor.
exmap openThisVimrc jscommand { view.app.openWithDefaultApp("/00_system/03_plugins/obsidian_vimrc_support/obsidian.vimrc") }
nnoremap g, :openThisVimrc<CR>

" Open Developer Tools (useful for debugging Obsidian plugins and custom scripts).
exmap openDevTools jscommand { electronWindow.openDevTools() }
nnoremap ? :obcommand<CR>:openDevTools<CR>

"───────────────────────────────────────────────────────────────────────────────
" CLIPBOARD MANAGEMENT
"───────────────────────────────────────────────────────────────────────────────

" Use the system clipboard for copying and pasting.
set clipboard=unnamed

" Make `Y` behave like `D` and `C` (copy until the end of the line).
nnoremap Y y$

" Prevent certain operations from modifying the default register.
nnoremap c "_c
nnoremap C "_C
nnoremap x "_x
vnoremap p P

" Paste at the end of the current line.
nnoremap P mzA<Space><Esc>p`z

" Paste URL into selection or current word (uses the `pasteinto` command).
" NOTE: On macOS, command-key mappings are `<M-*>`, not `<D-*>`.
noremap <M-k> :pasteinto<CR>

" Ensure undo point when pasting by inserting a dummy character and deleting it.
" This prevents unintended behavior in Alfred workflows that modify text.
vnoremap <M-v> <Esc>ix<Esc>xgv"+p

"───────────────────────────────────────────────────────────────────────────────
" COPY PATH SEGMENTS
"───────────────────────────────────────────────────────────────────────────────

" Copy various path formats using JavaScript commands.
exmap copyAbsolutePath jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { copyPathSegment("absolute") }
exmap copyRelativePath jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { copyPathSegment("relative") }
exmap copyFilename jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { copyPathSegment("filename") }
exmap copyParentPath jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { copyPathSegment("parent") }
exmap copyObsidianUriMdLink jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { copyObsidianUriMdLink() }

" Keybindings for quickly copying different path formats.
noremap ,ya :copyAbsolutePath<CR>
noremap ,yr :copyRelativePath<CR>
noremap ,yp :copyParentPath<CR>
noremap ,yn :copyFilename<CR>
noremap ,yo :copyObsidianUriMdLink<CR>

"───────────────────────────────────────────────────────────────────────────────
" NAVIGATION ENHANCEMENTS
"───────────────────────────────────────────────────────────────────────────────

" Move by **visual lines** instead of actual lines (useful with soft wrapping).
nnoremap j gj
nnoremap k gk
nnoremap I g0i
nnoremap A g$a

" Increase movement speed with **HJKL**.
noremap H g0
noremap L g$
nnoremap J 6gj
nnoremap K 6gk
vnoremap J 6j
vnoremap K 6k

" Delete multiple lines in one go.
onoremap J 2j

" Jump between previous locations.
nnoremap <C-h> <C-o>
nnoremap <C-l> <C-i>

" Origami-style folding controls (WARNING: slightly breaks `h` and `l` in tables).
exmap origamiH jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { origamiH() }
nnoremap h :origamiH<CR>
exmap origamiL jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { origamiL() }
nnoremap l :origamiL<CR>

"───────────────────────────────────────────────────────────────────────────────
" GOTO LOCATIONS
"───────────────────────────────────────────────────────────────────────────────

" Jump to matching parenthesis (useful for Pandoc citations).
nnoremap gm %

" Language Tool: Jump to next/previous grammar suggestion.
exmap nextSuggestion obcommand obsidian-languagetool-plugin:ltjump-to-next-suggestion
noremap ge :nextSuggestion<CR>

exmap prevSuggestion obcommand obsidian-languagetool-plugin:ltjump-to-previous-suggestion
noremap gE :prevSuggestion<CR>

" Navigate between headings in Markdown files.
exmap gotoNextHeading jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { gotoLineWithPattern("next", /^##+ .*/) }
nnoremap <C-j> :gotoNextHeading<CR>
exmap gotoPrevHeading jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { gotoLineWithPattern("prev", /^##+ .*/) }
nnoremap <C-k> :gotoPrevHeading<CR>

" Open the next link found in the file.
exmap openNextLink jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { openNextLink("current-tab") }
nnoremap gx :openNextLink<CR>
exmap openNextLinkInNewTab jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { openNextLink("new-tab") }
nnoremap gX :openNextLinkInNewTab<CR>

" Jump to footnotes (requires Footnotes Shortcut Plugin).
exmap gotoFootnote obcommand obsidian-footnotes:insert-autonumbered-footnote
nnoremap gf :gotoFootnote<CR>

" Jump to last change (only works once).
nnoremap g; u<C-r>

" Find last link in the file.
exmap gotoLastLinkInFile jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { gotoLastLinkInFile() }
nnoremap g. :gotoLastLinkInFile<CR>

" Jump to next/previous Markdown links (`[[wikilinks]]`).
exmap gotoNextLinkInFile jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { gotoLineWithPattern("next", /\[\[/) }
nnoremap gj :gotoNextLinkInFile<CR>zt<C-y><C-y>
exmap gotoPrevLinkInFile jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { gotoLineWithPattern("prev", /\[\[/) }
nnoremap gk :gotoPrevLinkInFile<CR>zt<C-y><C-y>

" Jump to next/previous unfinished task (`- [ ] Task`).
exmap gotoNextTask jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { gotoLineWithPattern("next", /- \[[x ]\]|TODO/) }
nnoremap gt :gotoNextTask<CR>
exmap gotoPrevTask jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { gotoLineWithPattern("prev", /- $begin:math:display$[x ]$end:math:display$|TODO/) }
nnoremap gT :gotoPrevTask<CR>

"───────────────────────────────────────────────────────────────────────────────
" FILE, TAB, AND WINDOW NAVIGATION
"───────────────────────────────────────────────────────────────────────────────

" Open files quickly using the **Another Quick Switcher** plugin.
" [g]oto [o]pen file (Quick Switcher for fast file search).
exmap quickSwitcher obcommand obsidian-another-quick-switcher:search-command_main-search
noremap go :quickSwitcher<CR>
noremap gr :quickSwitcher<CR>

" Alternative file search (alternate Quick Switcher mode).
exmap altSearch obcommand obsidian-another-quick-switcher:search-command_alt-search
noremap gO :altSearch<CR>

" Navigate back/forward in file history (similar to browser navigation).
exmap goBack obcommand app:go-back
exmap goForward obcommand app:go-forward
noremap <BS> :goBack<CR>
noremap <S-BS> :goForward<CR>

" Close the current tab/window.
exmap closeWindow obcommand workspace:close-window
nnoremap ZZ :closeWindow<CR>

" Split editor **vertically** (like Vim's `:vsplit`).
exmap splitVertical obcommand workspace:split-vertical
noremap <C-w>v :splitVertical<CR>
noremap <C-v> :splitVertical<CR>

" Close all other open windows except the current one.
exmap closeOthers obcommand workspace:close-others
nnoremap <C-w>o :closeOthers<CR>

" Switch to the alternate buffer (emulates `:buffer #` in Vim).
" NOTE: Requires the **Grappling Hook** plugin.
exmap altBuffer obcommand grappling-hook:alternate-note
noremap <CR> :altBuffer<CR>

"───────────────────────────────────────────────────────────────────────────────
" SEARCH COMMANDS
"───────────────────────────────────────────────────────────────────────────────

" Start search mode (`/`).
nnoremap - /

" Clear search highlights and any notices.
exmap clearNotices jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { clearNotices() }
nnoremap <Esc> :clearNotices<CR>:nohl<CR>

" Live grep search (similar to Telescope's live grep).
exmap liveGrep obcommand obsidian-another-quick-switcher:grep
noremap gl :liveGrep<CR>

" Start global search-and-replace mode.
nnoremap ,v :%s///g
nnoremap ,rs :%s///g

"───────────────────────────────────────────────────────────────────────────────
" EDITING
"───────────────────────────────────────────────────────────────────────────────

" Indentation
" NOTE: `<Tab>` indentation is already handled natively by Obsidian.

" Move entire lines up or down (does not work in visual mode).
exmap lineUp obcommand editor:swap-line-up
exmap lineDown obcommand editor:swap-line-down
nnoremap <M-Up> :lineUp<CR>
nnoremap <M-Down> :lineDown<CR>

" Move character under the cursor left or right.
nnoremap <M-Right> dlp
nnoremap <M-Left> dlhhp

" Increment/decrement Markdown headings (`#` → `##` → `###`).
" <M-h> (cmd+h) increases heading level.
exmap headingIncrement jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { headingIncrementor(1) }
nnoremap <M-h> :headingIncrement<CR>
inoremap <M-h> <Esc>:headingIncrement<CR>a

" <M-S-h> (cmd+Shift+h) decreases heading level.
exmap headingDecrement jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { headingIncrementor(-1) }
nnoremap <M-S-h> :headingDecrement<CR>
inoremap <M-S-h> <Esc>:headingDecrement<CR>a

" Open spelling suggestion menu (emulates `z=` in Vim).
exmap contextMenu obcommand editor:context-menu
noremap zl :contextMenu<CR>

" Consistent undo/redo behavior across commands.
nnoremap U <C-r>  " Remap `U` to redo
nnoremap ,ur 1000<C-r>  " "Redo all" in case of multiple undo steps.

" Toggle lowercase/title case.
exmap toggleLowercaseTitleCase jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { toggleLowercaseTitleCase() }
nnoremap < :toggleLowercaseTitleCase<CR>

" Convert the current word to Hiragana.
exmap hiraganafyCword jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { hiraganafyCword() }
nnoremap > :hiraganafyCword<CR>

" Keep cursor position unchanged when toggling case (`~`).
nnoremap ~ v~

" Change word/selection without affecting registers.
nnoremap <Space> "_ciw
vnoremap <Space> "_c
onoremap <Space> iw
nnoremap <S-Space> "_daw

" Merge lines (removes list formatting or blockquotes).
exmap smartMerge jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { smartMerge() }
nnoremap m :smartMerge<CR>

" Split lines at cursor position.
nnoremap ,s i<CR><CR><Esc>

" Ensure `o` and `O` respect list items and blockquotes.
exmap openBelow jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { smartOpenLine("below") }
nnoremap o :openBelow<CR>
exmap openAbove jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { smartOpenLine("above") }
nnoremap O :openAbove<CR>

" Insert blank lines above/below the cursor.
nnoremap = mzO<Esc>`z
nnoremap _ mzo<Esc>`z

" Increment (`+`) and decrement (`ü`) numbers.
nnoremap + <C-a>
nnoremap ü <C-x>

"───────────────────────────────────────────────────────────────────────────────
" JAVASCRIPT COMMENTING
"───────────────────────────────────────────────────────────────────────────────

" Unmap `q` for recording macros (to avoid conflicts).
nunmap q

" Toggle single-line JavaScript comments (`//`).
exmap toggleJsLineComment jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { toggleJsLineComment() }
nnoremap qq :toggleJsLineComment<CR>

" Append `//` JavaScript comments at the end of a line.
exmap appendJsComment jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { appendJsComment() }
nnoremap Q :appendJsComment<CR>

"───────────────────────────────────────────────────────────────────────────────
" MARKDOWN-SPECIFIC COMMANDS
"───────────────────────────────────────────────────────────────────────────────

" Toggle task status (`- [ ]` → `- [x]`).
exmap checkList obcommand editor:toggle-checklist-status
nnoremap ,x :checkList<CR>

" Uncheck all tasks (`- [x]` → `- [ ]`).
nnoremap ,X :%s/-<Space>\[x\]<Space>/-<Space>[<Space>]<Space>/<CR>

" Toggle blockquotes (`>`).
exmap toggleBlockquote obcommand editor:toggle-blockquote
nnoremap ,< :toggleBlockquote<CR>

" Append punctuation without moving the cursor.
nnoremap ,, mzA,<Esc>`z  " Append comma
nnoremap ,. mzA.<Esc>`z  " Append period

" Insert horizontal rule (`---`).
exmap insertHr jscommand { editor.replaceSelection("\n---\n"); }
nnoremap qw :insertHr<CR>

" Delete the last character on the current line.
exmap deleteLastChar jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { deleteLastChar() }
nnoremap X :deleteLastChar<CR>

"───────────────────────────────────────────────────────────────────────────────
" LEADER MAPPINGS
"───────────────────────────────────────────────────────────────────────────────

" Log the variable under the cursor (useful for debugging).
exmap consoleLogFromWordUnderCursor jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { consoleLogFromWordUnderCursor() }
nnoremap ,ll :consoleLogFromWordUnderCursor<CR>

" Enhance URLs with titles (same as `[c]ode action` in Neovim).
exmap enhanceUrlWithTitle obcommand obsidian-auto-link-title:enhance-url-with-title
nnoremap ,cc :enhanceUrlWithTitle<CR>

" Freeze the interface (potentially useful in certain plugin workflows).
exmap freezeInterface jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { freezeInterface() }
nnoremap ,if :freezeInterface<CR>

" Accept language tool or rephraser suggestions.
exmap acceptHighlightsAndStrikethrus jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { highlightsAndStrikethrus("accept") }
exmap acceptLtSuggestion obcommand obsidian-languagetool-plugin:ltaccept-suggestion-1
noremap ga :acceptHighlightsAndStrikethrus<CR>:acceptLtSuggestion<CR>

" Reject rephraser suggestions.
exmap rejectHighlightsAndStrikethrus jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { highlightsAndStrikethrus("reject") }
noremap gA :rejectHighlightsAndStrikethrus<CR>

" Mark the current data file as read.
exmap markAsRead obcommand quadro:mark-datafile-as-read
nnoremap ,rr :markAsRead<CR>

" Inspect Chrome version.
exmap inspectChromeVersion jscommand { new Notice ('Chrome version: ' + process.versions.chrome.split('.')[0], 4000) }
nnoremap ,iv :inspectChromeVersion<CR>

"───────────────────────────────────────────────────────────────────────────────
" META: PLUGIN AND SETTING-RELATED BINDINGS
"───────────────────────────────────────────────────────────────────────────────

" Update all installed plugins.
exmap updatePlugins jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { updatePlugins() }
nnoremap ,pp :updatePlugins<CR>

" Open the Obsidian **plugin directory** in the system file browser.
exmap openPluginDir jscommand { view.app.openWithDefaultApp(view.app.vault.configDir + '/plugins'); }
nnoremap ,pd :openPluginDir<CR>

" Open the **Meta folder** (assumed to contain configuration files).
exmap openMetaDir jscommand { view.app.openWithDefaultApp('/Meta'); }
nnoremap ,pm :openMetaDir<CR>

" Open the **CSS snippet directory**.
exmap openSnippetDir jscommand { view.app.openWithDefaultApp(view.app.vault.configDir + '/snippets'); }
nnoremap ,ps :openSnippetDir<CR>

" Open the **theme directory**.
exmap openThemeDir jscommand { view.app.openWithDefaultApp(view.app.vault.configDir + '/themes'); }
nnoremap ,pt :openThemeDir<CR>

" Open the **Appearance settings** menu.
exmap openAppearanceSettings jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { openAppearanceSettings() }
nnoremap ,pa :openAppearanceSettings<CR>

" Open the **Community Plugins settings** menu.
exmap openCommunityPluginsSettings jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { openCommunityPluginsSettings() }
nnoremap ,pl :openCommunityPluginsSettings<CR>

" Open the **plugin installation menu**.
exmap installPlugins jscommand { view.app.workspace.protocolHandlers.get("show-plugin")({ id: ' ' }); }
nnoremap ,pi :installPlugins<CR>

" Open **dynamic highlights settings**.
" exmap openDynamicHighlightsSettings jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { openDynamicHighlightsSettings() }
" nnoremap ,ph :openDynamicHighlightsSettings<CR>

" Cycle between installed color themes.
exmap cycleColorscheme jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { cycleColorscheme() }
nnoremap ,pc :cycleColorscheme<CR>

"───────────────────────────────────────────────────────────────────────────────
" WORKSPACE MANAGEMENT
"───────────────────────────────────────────────────────────────────────────────

" Load a workspace layout.
exmap loadWorkspace jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { workspace("load", "Basic") }
nnoremap ,w :loadWorkspace<CR>

" Save the current workspace layout.
exmap saveWorkspace jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { workspace("save", "Basic") }
nnoremap ,W :saveWorkspace<CR>

"───────────────────────────────────────────────────────────────────────────────
" FILESYSTEM COMMANDS
"───────────────────────────────────────────────────────────────────────────────

" Create a **new file** in the selected folder.
exmap new obcommand pseudometa-startup-actions:new-file-in-folder
nnoremap ,fn :new<CR>

" Rename the current file.
exmap rename obcommand workspace:edit-file-title
nnoremap ,fr :rename<CR>

" Move the current file.
exmap move obcommand obsidian-another-quick-switcher:move
nnoremap ,fm :move<CR>

" Duplicate the current file.
exmap duplicate obcommand file-explorer:duplicate-file
nnoremap ,fw :duplicate<CR>

" Delete the current file.
exmap delete obcommand app:delete-file
nnoremap ,fd :delete<CR>

" Open the system **trash bin**.
exmap openTrash jscommand { view.app.openWithDefaultApp("/.trash"); }
nnoremap ,t :openTrash<CR>

"───────────────────────────────────────────────────────────────────────────────
" VISUAL MODE ENHANCEMENTS
"───────────────────────────────────────────────────────────────────────────────

" Repeated `V` selects more lines.
vnoremap V gj

" Pressing `v` twice enters **visual block mode**.
vnoremap v <C-v>

"───────────────────────────────────────────────────────────────────────────────
" TEXT OBJECTS
"───────────────────────────────────────────────────────────────────────────────

" Custom text objects for quicker access to:
" [m]assive word, [q]uote, [z]ingle quote, inline cod[e],
" [r]ectangular bracket, and [c]urly braces.

onoremap am aW  " Select massive word (Word + Punctuation)
onoremap im iW  " Inner massive word
onoremap aq a"  " Around double quotes
onoremap iq i"  " Inside double quotes
onoremap az a'  " Around single quotes
onoremap iz i'  " Inside single quotes
onoremap ae a`  " Around inline code
onoremap ie i`  " Inside inline code
onoremap ir i[  " Inside square brackets
onoremap ar a[  " Around square brackets
onoremap ac a{  " Around curly braces
onoremap ic i{  " Inside curly braces

vnoremap am aW
vnoremap im iW
vnoremap aq a"
vnoremap iq i"
vnoremap ay a'
vnoremap iy i'
vnoremap ae a`
vnoremap ie i`
vnoremap ir i[
vnoremap ar a[
vnoremap ac a{
vnoremap ic i{

" Emulate some text objects from `nvim-various-textobjs`.
nnoremap ygg ggyG  " Yank entire document
nnoremap dgg ggdG  " Delete entire document
nnoremap cgg ggcG  " Change entire document

onoremap rg G  " Select until end of file
vnoremap rg G
onoremap rp }  " Select until paragraph end
vnoremap rp }
onoremap m t]  " Select until `]`
vnoremap m t]
onoremap w t"  " Select until `"`
vnoremap w t"
onoremap k i"  " Inside double quotes
onoremap K a"  " Around double quotes

"───────────────────────────────────────────────────────────────────────────────
" SUBSTITUTE
"───────────────────────────────────────────────────────────────────────────────

" Poor man's `substitute.nvim`: brute-forces text object replacements 💀
" Unmap `s` to avoid conflicts with built-in behavior.
nunmap s

" Substitute selected text objects with the clipboard (`VP` pastes selection).
nnoremap ss VP
nnoremap S vg$P  " Substitute to end of line.

" Replace **multiple lines** quickly.
nnoremap sj VjP
nnoremap sJ VjjP

" Replace common text objects.
nnoremap sim viWP  " [m]assive word.
nnoremap sam vaWP
nnoremap siw viwP  " Inner word.
nnoremap saw vawP  " Around word.
nnoremap sis visP  " Inner sentence.
nnoremap sas vasP  " Around sentence.
nnoremap sip VipP  " Inner paragraph.
nnoremap sap VapP  " Around paragraph.

" Replace inside/around **parentheses, brackets, and quotes**.
nnoremap sib vi)P
nnoremap sab va)P
nnoremap saq va"P
nnoremap siq vi"P
nnoremap sk vi"P
nnoremap saz va'P
nnoremap siz vi'P
nnoremap sae va`P
nnoremap sie vi`P
nnoremap sir vi]P
nnoremap sar va]P
nnoremap sic vi}P
nnoremap sac va}P

" Replace **entire document or regions**.
nnoremap srg vGP  " Replace full document.
nnoremap srp v}P  " Replace paragraph.
nnoremap sgg vggGP  " Replace everything.
nnoremap sm vt]P  " Replace text inside `]`.

"───────────────────────────────────────────────────────────────────────────────
" DUPLICATE TEXT OBJECTS
"───────────────────────────────────────────────────────────────────────────────

" Poor man's `duplicate.nvim`: brute-forces text object duplication 💀
" unmap w
" nunmap w

" Duplicate **current line**.
" nnoremap ww yyp
" nnoremap W y$$p  " Duplicate line to end.

" Duplicate **multiple lines**.
nnoremap wj yjjp  " Duplicate two lines.

" Duplicate common text objects.
nnoremap wim yiWp  " [m]assive word.
nnoremap wam yaWp
nnoremap wiw yiwp  " Inner word.
nnoremap waw yawp  " Around word.
nnoremap wis yisp  " Inner sentence.
nnoremap was yasp  " Around sentence.
nnoremap wip yipP  " Inner paragraph.
nnoremap wap yapP  " Around paragraph.

" Duplicate inside/around **parentheses, brackets, and quotes**.
nnoremap wib yi)p
nnoremap waq ya"p
nnoremap wiq yi"p
nnoremap wk yi"p
nnoremap waz ya'p
nnoremap wiz yi'p
nnoremap wae ya`p
nnoremap wie yi`p
nnoremap wab ya)p
nnoremap wir yi]p
nnoremap war ya]p
nnoremap wic yi}p
nnoremap wac ya}p

" Duplicate **entire document or regions**.
nnoremap wrg yGp  " Duplicate full document.
nnoremap wrp y}p  " Duplicate paragraph.
nnoremap wm yt]p  " Duplicate text inside `]`.

"───────────────────────────────────────────────────────────────────────────────
" FOLDING COMMANDS
"───────────────────────────────────────────────────────────────────────────────

" Toggle folding at the current location.
exmap togglefold obcommand editor:toggle-fold
nnoremap za :togglefold<CR>

" Open (unfold) the current fold.
nnoremap zo :togglefold<CR>

" Close (fold) the current section.
nnoremap zc :togglefold<CR>

" Unfold/fold **all sections**.
exmap unfoldall obcommand editor:unfold-all
exmap foldall obcommand editor:fold-all
nnoremap zm :foldall<CR>
nnoremap zz :foldall<CR>
nnoremap zr :unfoldall<CR>

"───────────────────────────────────────────────────────────────────────────────
" OPTION TOGGLING
"───────────────────────────────────────────────────────────────────────────────

" Toggle **spellcheck**.
exmap spellcheck obcommand editor:toggle-spellcheck
nnoremap ,os :spellcheck<CR>

" Toggle **line numbers**.
exmap toggleLineNumbers jsfile /00_system/03_plugins/obsidian_vimrc_support/vimrc-jsfile.js { toggleLineNumbers() }
nnoremap ,on :toggleLineNumbers<CR>

" Toggle **diagnostics (language tool suggestions)**.
exmap enableDiagnostics obcommand obsidian-languagetool-plugin:ltcheck-text
nnoremap ,od :enableDiagnostics<CR>
exmap disableDiagnostics obcommand obsidian-languagetool-plugin:ltclear
nnoremap ,oD :disableDiagnostics<CR>

" Toggle **AI-based autocompletion**.
exmap toggleAiCompletion obcommand copilot-auto-completion:toggle
nnoremap ,oa :toggleAiCompletion<CR>

" Toggle **Markdown source mode vs. Live Preview**.
exmap sourceModeLivePreview obcommand editor:toggle-source
nnoremap ,oc :sourceModeLivePreview<CR>

" Toggle **soft wrap for long lines**.
exmap lineLength obcommand obsidian-style-settings:style-settings-class-toggle-shimmering-focus-readable-line-length-toggle
nnoremap ,ow :lineLength<CR>

" Toggle **image resizing constraints**.
exmap maxImageSize obcommand obsidian-style-settings:style-settings-class-toggle-shimmering-focus-max-image-size-toggle
nnoremap ,oi :maxImageSize<CR>

"───────────────────────────────────────────────────────────────────────────────
" LINTING & AI COMPLETION
"───────────────────────────────────────────────────────────────────────────────

" Format and lint the current document.
" NOTE: Requires the `obsidian-linter` plugin.
exmap lint obcommand obsidian-linter:lint-file-unless-ignored
nnoremap <M-s> :lint<CR>  " <M-s> (cmd+s) triggers linting in normal mode.

"───────────────────────────────────────────────────────────────────────────────
" AI COMPLETION (Copilot Auto-Completion Plugin)
"───────────────────────────────────────────────────────────────────────────────

" BUG: AI completion suggestion acceptance is currently broken.
" ISSUE: https://github.com/j0rd1smit/obsidian-copilot-auto-completion/issues/45
"
" Uncomment the following lines if this issue gets resolved:
"
" exmap acceptGhostText obcommand copilot-auto-completion:accept
" inoremap <M-s> <Esc>:acceptGhostText<CR>a
