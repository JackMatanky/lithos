---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# Command

## Properties

### Id

```ts
id: string
```

Globally unique ID to identify this command.

### Name

```ts
name: string
```

Human friendly name for searching.

### Icon

```ts
icon: string
```

Icon ID to be used in the toolbar.

### mobileOnly

```ts
mobileOnly: boolean
```

### Repeatable

```ts
repeatable: boolean
```

Whether holding the hotkey should repeatedly trigger this command. Defaults to false.

### Callback

```ts
callback: () => any
```

Simple callback, triggered globally.

### checkCallback

```ts
checkCallback: (checking: boolean) => boolean | void
```

Complex callback, overrides the simple callback.  
Used to "check" whether your command can be performed in the current circumstances.  
For example, if your command requires the active focused pane to be a MarkdownSourceView, then  
you should only return true if the condition is satisfied. Returning false or undefined causes  
the command to be hidden from the command palette.

### editorCallback

```ts
editorCallback: (editor: Editor, ctx: MarkdownView | MarkdownFileInfo) => any
```

A command callback that is only triggered when the user is in an editor.  
Overrides `callback` and `checkCallback`

### editorCheckCallback

```ts
editorCheckCallback: (checking: boolean, editor: Editor, ctx: MarkdownView | MarkdownFileInfo) => boolean | void
```

A command callback that is only triggered when the user is in an editor.  
Overrides `editorCallback`, `callback` and `checkCallback`

### Hotkeys

```ts
hotkeys: Hotkey[]
```

Sets the default hotkey. It is recommended for plugins to avoid setting default hotkeys if possible,  
to avoid conflicting hotkeys with one that's set by the user, even though customized hotkeys have higher priority.
