---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# FileView

Extends `ItemView`

## Constructor

```ts
constructor(leaf: WorkspaceLeaf);
```

## Properties

### allowNoFile

```ts
allowNoFile: boolean
```

### File

```ts
file: TFile
```

### Navigation

```ts
navigation: boolean
```

Whether or not the view is intended for navigation.  
If your view is a static view that is not intended to be navigated away, set this to false.  
(For example: File explorer, calendar, etc.)  
If your view opens a file or can be otherwise navigated, set this to true.  
(For example: Markdown editor view, Kanban view, PDF view, etc.)  
File views can be navigated by default.

## Methods

### getDisplayText

```ts
getDisplayText(): string;
```

### Onload

```ts
onload(): void;
```

Override this to load your component

### getState

```ts
getState(): any;
```

### setState

```ts
setState(state: any, result: ViewStateResult): Promise<void>;
```

### onLoadFile

```ts
onLoadFile(file: TFile): Promise<void>;
```

### onUnloadFile

```ts
onUnloadFile(file: TFile): Promise<void>;
```

### onRename

```ts
onRename(file: TFile): Promise<void>;
```

### canAcceptExtension

```ts
canAcceptExtension(extension: string): boolean;
```
