---
title: side_panel_snippet_reference_latex
aliases:
  - Side Panel LaTeX Snippet Reference 
  - LaTeX Snippet Reference
  - side_panel_snippet_reference_latex
cssclasses:
  - inline_title_hide
  - side_panel_narrow
  - side_panel_table
  - read_hide_properties
  - read_narrow_margin
  - font_size_small
  - list_narrow
date_created: 2023-09-20T11:09
date_modified: 2024-08-28T11:54
---

### LazyVim Keybindings

#### Core Navigation

| Command         | Action                        |
| --------------- | ----------------------------- |
| `<C-f>`         | Forward (Page down)           |
| `<C-d>`         | Move down (Half a page)       |
| `<C-b>`         | Backward (Page up)            |
| `<C-u>`         | Move up (Half a page)         |
| `<C-o>`         | Jump back                     |
| `<C-i>`         | Jump forward                  |
| `gg`            | Go to first line              |
| `Shift-G`       | Go to last line               |
| `:10` or `10gg` | Jump to line #10              |
| `10j`           | (Relative) Jump down 10 lines |
| `10k`           | (Relative) Jump up 10 lines   |
| `J`             | Join lines                    |

#### UI/UX 

| Command      | Action                    |
| ------------ | ------------------------- |
| `<leader>uC` | Colorscheme with preview  |
| `<leader>uD` | Enable code block dimming |
| `<leader>ul` | Toggle line number        |
| `<leader>uL` | Toggle relative number    |
| `<leader>uw` | Toggle word wrap          |
| `<C-/>`      | Toggle Terminal window    |
| `:Neotree`   | Neotree file explorer     |

#### Buffer Management

| Command             | Action               |
| ------------------- | -------------------- |
| `<leader>fb`        | List open buffers    |
| `Shift-l`           | Next buffer          |
| `Shift-h`           | Prev buffer          |
| <code>&#93;b</code> | Next buffer          |
| `[b`                | Prev buffer          |
| `<leader>bd`        | Close current buffer |
| `<C-w>v`            | Split vertical       |
| `<C-w>s`            | Split horizontal     |
| `<C-w>h/j/k/l`      | Navigate splits      |

#### Text Objects

| Command | Action                    |
| ------- | ------------------------- |
| `viw`   | Select inner word         |
| `vi"`   | Select inner quotes       |
| `vi{`   | Select inner curly braces |
| `vip`   | Select inner paragraph    |
| `va[`   | Select around `[]` braces |
| `dap`   | Delete around paragraph   |

#### Code Folding

| Command      | Action                        |
| ------------ | ----------------------------- |
| `zR` or `zi` | Open all folds                |
| `zM`         | Close all folds               |
| `za`         | Toggle fold                   |
| `zA`         | Toggle all folds under cursor |
| `zc`         | Close fold                    |
| `zo`         | Open fold                     |
| `zO`         | Open all folds under cursor   |

#### Marks & Bookmarks

| Command         | Action                           |
| --------------- | -------------------------------- |
| `<leader>sm`    | View all marks                   |
| `m[a-z]`        | Set local mark                   |
| `'[a-z]`        | Jump to mark                     |
| `' '`           | Jump to last position            |
| `` `[a-z]` ``   | Jump to exact position           |
| `:delmarks a-z` | Delete lowercase marks (a-z)     |
| `:delmarks ax`  | Delete marks `a` and `x`         |
| `:delmarks!`    | Delete all marks except A-Z, 0-9 |

#### Functions & Symbols (LSP)

| Command      | Action                    |
| ------------ | ------------------------- |
| `:LspInfo`   | Show attached LSP info    |
| `<leader>cs` | Document symbols          |
| `gr`         | Find all references       |
| `gd`         | Go to definition          |
| `gD`         | Go to declaration         |
| `gy`         | Go to Type definition     |
| `K`          | Show docstring/type hints |
| `[[` or `]]` | Prev/Next reference       |

#### Diagnostics

| Command             | Action                |
| ------------------- | --------------------- |
| <code>&#93;d</code> | Next diagnostic       |
| `[d`                | Prev diagnostic       |
| `<leader>sd`        | Document diagnostics  |
| `<leader>sD`        | Workspace diagnostics |

#### Code Actions

| Command      | Action         |
| ------------ | -------------- |
| `<leader>cr` | Rename symbols |
| `<leader>cf` | Format code    |
| `<leader>ca` | Code actions   |

#### Indentation

| Command | Action                      |
| ------- | --------------------------- |
| `>`     | Indent right                |
| `<`     | Indent left                 |
| `=`     | Auto-indent as per language |
| `=ip`   | Indent current paragraph    |
| `gg=G`  | Auto-indent entire file     |

#### Search

| Command      | Action                   |
| ------------ | ------------------------ |
| `<leader>sr` | Search and Replace       |
| `<leader>fc` | Find Config files        |
| `<leader>ff` | Find files (Root dir)    |
| `<leader>/`  | Grep (Root dir)          |
| `<leader>sG` | Grep (CWD)               |
| `<leader>ss` | Symbol search            |
| `<leader>sc` | Command history          |
| `<leader>sw` | Search word under cursor |
| `<leader>sk` | Search all keymaps       |
| `<leader>st` | Search TODO \| WARNING   |

#### Git (fzf-lua)

| Command      | Action                  |
| ------------ | ----------------------- |
| `<leader>gc` | Commit log texts search |
| `<leader>gs` | Status (file search)    |
| `<leader>ge` | Git explorer (Neotree)  |
| `<leader>gf` | Current file history    |

#### LazyGit

| Command      | Action                 |
| ------------ | ---------------------- |
| `<leader>gg` | Open LazyGit window    |
| `<C-r>`      | Switch to recent repo  |
| `<C-b>`      | Filter files by status |
| `p`          | Git pull               |
| `P`          | Git push               |
| `<space>`    | Stage                  |
| `a`          | Stage all              |
| `c`          | Commit                 |
| `s`          | Stash                  |
| `z`          | Undo                   |
| `<C-z>`      | Redo                   |
| `i`          | Add to.gitignore      |
| `q`          | Quit                   |

### LaTeX Snippets

| Trigger | Result                    | Mode |
|:-------:| ------------------------- |:----:|
|   @\w   | Greek Letters             | Math |
| AND/OR  | `\quad\text{and/or}\quad` | Math |
|   pw    | `^{$0}`                   | Math |
|   `^`   | `^{$0}$1`                 | Math |
|   sr    | `^{2}`                    | Math |
