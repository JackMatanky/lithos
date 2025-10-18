---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# Events

## Constructor

```ts
constructor();
```

## Methods

### On

```ts
on(name: string, callback: (...data: any) => any, ctx?: any): EventRef;
```

### Off

```ts
off(name: string, callback: (...data: any) => any): void;
```

### Offref

```ts
offref(ref: EventRef): void;
```

### Trigger

```ts
trigger(name: string, ...data: any[]): void;
```

### tryTrigger

```ts
tryTrigger(evt: EventRef, args: any[]): void;
```
