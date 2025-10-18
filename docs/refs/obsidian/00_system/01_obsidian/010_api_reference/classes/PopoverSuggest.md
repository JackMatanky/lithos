---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# PopoverSuggest

Implements `ISuggestOwner<T>`, `CloseableComponent`

## Constructor

```ts
constructor(app: App, scope: Scope);
```

## Methods

### Open

```ts
open(): void;
```

### Close

```ts
close(): void;
```

### renderSuggestion

```ts
abstract renderSuggestion(value: T, el: HTMLElement): void;
```

Render the suggestion item into DOM.

### selectSuggestion

```ts
abstract selectSuggestion(value: T, evt: MouseEvent | KeyboardEvent): void;
```

Called when the user makes a selection.
