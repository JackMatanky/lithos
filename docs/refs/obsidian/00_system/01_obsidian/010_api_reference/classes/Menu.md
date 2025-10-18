---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# Menu

Extends `Component`

Implements `CloseableComponent`

## Constructor

```ts
constructor();
```

## Methods

### setNoIcon

```ts
setNoIcon(): this;
```

### setUseNativeMenu

```ts
setUseNativeMenu(useNativeMenu: boolean): this;
```

Force this menu to use native or DOM.  
(Only works on the desktop app)

### addItem

```ts
addItem(cb: (item: MenuItem) => any): this;
```

Adds a menu item. Only works when menu is not shown yet.

### addSeparator

```ts
addSeparator(): this;
```

Adds a separator. Only works when menu is not shown yet.

### showAtMouseEvent

```ts
showAtMouseEvent(evt: MouseEvent): this;
```

### showAtPosition

```ts
showAtPosition(position: MenuPositionDef, doc?: Document): this;
```

### Hide

```ts
hide(): this;
```

### Close

```ts
close(): void;
```

### onHide

```ts
onHide(callback: () => any): void;
```
