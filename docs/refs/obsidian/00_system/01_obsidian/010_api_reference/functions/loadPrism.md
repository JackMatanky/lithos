---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# loadPrism

```ts
export function loadPrism(): Promise<any>;
```

Load Prism.js and return a promise to the global Prism object.  
Can also use `Prism` after this promise resolves to get the same reference.
