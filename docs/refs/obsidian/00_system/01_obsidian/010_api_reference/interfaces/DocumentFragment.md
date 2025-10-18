---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# DocumentFragment

A minimal document object that has no parent. It is used as a lightweight version of Document that stores a segment of a document structure comprised of nodes just like a standard document. The key difference is that because the document fragment isn't part of the active document tree structure, changes made to the fragment don't affect the document, cause reflow, or incur any performance impact that can occur when changes are made.

## Methods

### Find

```ts
find: (selector: string) => HTMLElement
```

### findAll

```ts
findAll: (selector: string) => HTMLElement[]
```
