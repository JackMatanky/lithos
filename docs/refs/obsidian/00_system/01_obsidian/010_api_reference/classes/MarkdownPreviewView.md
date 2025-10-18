---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# MarkdownPreviewView

Extends `MarkdownRenderer`

Implements `MarkdownSubView`, `MarkdownPreviewEvents`

## Constructor

```ts
constructor(containerEl: HTMLElement);
```

## Properties

### containerEl

```ts
containerEl: HTMLElement
```

## Methods

### Get

```ts
get(): string;
```

### Set

```ts
set(data: string, clear: boolean): void;
```

### Clear

```ts
clear(): void;
```

### Rerender

```ts
rerender(full?: boolean): void;
```

### getScroll

```ts
getScroll(): number;
```

### applyScroll

```ts
applyScroll(scroll: number): void;
```
