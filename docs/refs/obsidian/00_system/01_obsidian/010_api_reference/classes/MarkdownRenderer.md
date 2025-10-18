---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# MarkdownRenderer

Extends `MarkdownRenderChild`

Implements `MarkdownPreviewEvents`, `HoverParent`

## Constructor

```ts
constructor(containerEl: HTMLElement);
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

### renderMarkdown

```ts
static renderMarkdown(markdown: string, el: HTMLElement, sourcePath: string, component: Component): Promise<void>;
```

Renders markdown string to an HTML element.
