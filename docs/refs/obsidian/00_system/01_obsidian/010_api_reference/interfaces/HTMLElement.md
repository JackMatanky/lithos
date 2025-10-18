---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# HTMLElement

Any HTML element. Some elements directly implement this interface, while others implement it via an interface that inherits it.

## Properties

### _EVENTS

```ts
_EVENTS: { fullscreenchange?: EventListenerInfo[]; fullscreenerror?: EventListenerInfo[]; abort?: EventListenerInfo[]; animationcancel?: EventListenerInfo[]; ... 87 more ...; paste?: EventListenerInfo[]; }
```

## Methods

### On

```ts
on: <K extends "input" | "progress" | "select" | "fullscreenchange" | "fullscreenerror" | "abort" | "animationcancel" | "animationend" | "animationiteration" | "animationstart" | "auxclick" | ... 80 more ... | "paste">(this: HTMLElement, type: K, selector: string, listener: (this: HTMLElement, ev: HTMLElementEventMap[K]...
```

### Off

```ts
off: <K extends "input" | "progress" | "select" | "fullscreenchange" | "fullscreenerror" | "abort" | "animationcancel" | "animationend" | "animationiteration" | "animationstart" | "auxclick" | ... 80 more ... | "paste">(this: HTMLElement, type: K, selector: string, listener: (this: HTMLElement, ev: HTMLElementEventMap[K]...
```

### onClickEvent

```ts
onClickEvent: (this: HTMLElement, listener: (this: HTMLElement, ev: MouseEvent) => any, options?: boolean | AddEventListenerOptions) => void
```

### onNodeInserted

```ts
onNodeInserted: (this: HTMLElement, listener: () => any, once?: boolean) => () => void
```

### onWindowMigrated

```ts
onWindowMigrated: (this: HTMLElement, listener: (win: Window) => any) => () => void
```

### Trigger

```ts
trigger: (eventType: string) => void
```
