---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# DropdownComponent

Extends `ValueComponent<string>`

## Constructor

```ts
constructor(containerEl: HTMLElement);
```

## Properties

### selectEl

```ts
selectEl: HTMLSelectElement
```

## Methods

### setDisabled

```ts
setDisabled(disabled: boolean): this;
```

### addOption

```ts
addOption(value: string, display: string): this;
```

### addOptions

```ts
addOptions(options: Record<string, string>): this;
```

### getValue

```ts
getValue(): string;
```

### setValue

```ts
setValue(value: string): this;
```

### onChange

```ts
onChange(callback: (value: string) => any): this;
```
