---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# ColorComponent

Extends `ValueComponent<string>`

Color picker component. Values are by default 6-digit hash-prefixed hex strings like `#000000`.

## Constructor

```ts
constructor(containerEl: HTMLElement);
```

## Methods

### getValue

```ts
getValue(): HexString;
```

### getValueRgb

```ts
getValueRgb(): RGB;
```

### getValueHsl

```ts
getValueHsl(): HSL;
```

### setValue

```ts
setValue(value: HexString): this;
```

### setValueRgb

```ts
setValueRgb(rgb: RGB): this;
```

### setValueHsl

```ts
setValueHsl(hsl: HSL): this;
```

### onChange

```ts
onChange(callback: (value: string) => any): this;
```
