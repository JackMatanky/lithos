---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# Scope

## Constructor

```ts
constructor(parent: Scope);
```

## Methods

### Register

```ts
register(modifiers: Modifier[], key: string | null, func: KeymapEventListener): KeymapEventHandler;
```

### Unregister

```ts
unregister(handler: KeymapEventHandler): void;
```
