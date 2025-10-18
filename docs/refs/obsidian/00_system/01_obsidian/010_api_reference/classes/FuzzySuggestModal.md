---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# FuzzySuggestModal

Extends `SuggestModal<FuzzyMatch<T>>`

## Constructor

```ts
constructor(app: App);
```

## Methods

### getSuggestions

```ts
getSuggestions(query: string): FuzzyMatch<T>[];
```

### renderSuggestion

```ts
renderSuggestion(item: FuzzyMatch<T>, el: HTMLElement): void;
```

Render the suggestion item into DOM.

### onChooseSuggestion

```ts
onChooseSuggestion(item: FuzzyMatch<T>, evt: MouseEvent | KeyboardEvent): void;
```

### getItems

```ts
abstract getItems(): T[];
```

### getItemText

```ts
abstract getItemText(item: T): string;
```

### onChooseItem

```ts
abstract onChooseItem(item: T, evt: MouseEvent | KeyboardEvent): void;
```
