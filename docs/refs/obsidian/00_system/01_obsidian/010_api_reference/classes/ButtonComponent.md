---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# ButtonComponent

Extends `BaseComponent`

## Constructor

```ts
constructor(containerEl: HTMLElement);
```

## Properties

### buttonEl

```ts
buttonEl: HTMLButtonElement
```

## Methods

### setDisabled

```ts
setDisabled(disabled: boolean): this;
```

### setCta

```ts
setCta(): this;
```

### removeCta

```ts
removeCta(): this;
```

### setWarning

```ts
setWarning(): this;
```

### setTooltip

```ts
setTooltip(tooltip: string, options?: TooltipOptions): this;
```

### setButtonText

```ts
setButtonText(name: string): this;
```

### setIcon

```ts
setIcon(icon: IconName): this;
```

### setClass

```ts
setClass(cls: string): this;
```

### onClick

```ts
onClick(callback: (evt: MouseEvent) => any): this;
```
