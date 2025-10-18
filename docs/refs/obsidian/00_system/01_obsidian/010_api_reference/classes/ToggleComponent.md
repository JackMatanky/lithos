---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# ToggleComponent

Extends `ValueComponent<boolean>`

## Constructor

```ts
constructor(containerEl: HTMLElement);
```

## Properties

### toggleEl

```ts
toggleEl: HTMLElement
```

## Methods

### setDisabled

```ts
setDisabled(disabled: boolean): this;
```

### getValue

```ts
getValue(): boolean;
```

### setValue

```ts
setValue(on: boolean): this;
```

### setTooltip

```ts
setTooltip(tooltip: string, options?: TooltipOptions): this;
```

### onClick

```ts
onClick(): void;
```

### onChange

```ts
onChange(callback: (value: boolean) => any): this;
```
