---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# MarkdownEditView

Implements `MarkdownSubView`, `HoverParent`, `MarkdownFileInfo`

This is the editor for Obsidian Mobile as well as the upcoming WYSIWYG editor.

## Constructor

```ts
constructor(view: MarkdownView);
```

## Properties

### App

```ts
app: App
```

### hoverPopover

```ts
hoverPopover: HoverPopover
```

## Methods

### Clear

```ts
clear(): void;
```

### Get

```ts
get(): string;
```

### Set

```ts
set(data: string, clear: boolean): void;
```

### getSelection

```ts
getSelection(): string;
```

### getScroll

```ts
getScroll(): number;
```

### applyScroll

```ts
applyScroll(scroll: number): void;
```
