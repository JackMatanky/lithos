---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# UIEvent

Simple user interface events.

## Properties

### targetNode

```ts
targetNode: Node
```

### Win

```ts
win: Window
```

### Doc

```ts
doc: Document
```

## Methods

### instanceOf

```ts
instanceOf: <T>(type: new (...data: any[]) => T) => this is T
```

Cross-window capable instanceof check, a drop-in replacement  
for instanceof checks on UIEvents.
