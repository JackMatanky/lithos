---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# loadPdfJs

```ts
export function loadPdfJs(): Promise<any>;
```

Load PDF.js and return a promise to the global pdfjsLib object.  
Can also use `window.pdfjsLib` after this promise resolves to get the same reference.
