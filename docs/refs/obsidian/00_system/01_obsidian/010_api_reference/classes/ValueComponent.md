---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# ValueComponent

Extends `BaseComponent`

## Constructor

```ts
constructor();
```

## Methods

### registerOptionListener

```ts
registerOptionListener(listeners: Record<string, (value?: T) => T>, key: string): this;
```

### getValue

```ts
abstract getValue(): T;
```

### setValue

```ts
abstract setValue(value: T): this;
```
