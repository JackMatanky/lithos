---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# WorkspaceItem

Extends `Events`

## Constructor

```ts
constructor();
```

## Methods

### getRoot

```ts
getRoot(): WorkspaceItem;
```

### getContainer

```ts
getContainer(): WorkspaceContainer;
```

Get the root container parent item, which can be one of:

- {@link WorkspaceRoot}
- {@link WorkspaceWindow}
