---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# Element

Element is the most general base class from which all objects in a Document inherit. It only has methods and properties common to all kinds of elements. More specific classes inherit from Element.

## Methods

### Find

```ts
find: (selector: string) => Element
```

### findAll

```ts
findAll: (selector: string) => HTMLElement[]
```

### findAllSelf

```ts
findAllSelf: (selector: string) => HTMLElement[]
```
