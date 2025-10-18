---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# WorkspaceLeaf

Extends `WorkspaceItem`

## Constructor

```ts
constructor();
```

## Properties

### View

```ts
view: View
```

## Methods

### openFile

```ts
openFile(file: TFile, openState?: OpenViewState): Promise<void>;
```

By default, `openFile` will also make the leaf active.  
Pass in `{ active: false }` to override.

### Open

```ts
open(view: View): Promise<View>;
```

### getViewState

```ts
getViewState(): ViewState;
```

### setViewState

```ts
setViewState(viewState: ViewState, eState?: any): Promise<void>;
```

### getEphemeralState

```ts
getEphemeralState(): any;
```

### setEphemeralState

```ts
setEphemeralState(state: any): void;
```

### togglePinned

```ts
togglePinned(): void;
```

### setPinned

```ts
setPinned(pinned: boolean): void;
```

### setGroupMember

```ts
setGroupMember(other: WorkspaceLeaf): void;
```

### setGroup

```ts
setGroup(group: string): void;
```

### Detach

```ts
detach(): void;
```

### getIcon

```ts
getIcon(): IconName;
```

### getDisplayText

```ts
getDisplayText(): string;
```

### onResize

```ts
onResize(): void;
```

### On

```ts
on(name: 'pinned-change', callback: (pinned: boolean) => any, ctx?: any): EventRef;
```

### On

```ts
on(name: 'group-change', callback: (group: string) => any, ctx?: any): EventRef;
```
