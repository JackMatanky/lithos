---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# Window

A window containing a DOM document; the document property points to the DOM document loaded in that window.

## Properties

### activeWindow

```ts
activeWindow: Window
```

The actively focused Window object. This is usually the same as `window` but  
it will be different when using popout windows.

### activeDocument

```ts
activeDocument: Document
```

The actively focused Document object. This is usually the same as `document` but  
it will be different when using popout windows.
