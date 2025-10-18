---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# Debounce

```ts
export function debounce<T extends unknown[], V>(cb: (...args: [
    ...T
]) => V, timeout?: number, resetTimer?: boolean): Debouncer<T, V>;
```

A standard debounce function.

## Parameters

| Parameter | Description |
|-----------|-------------|
| `cb` | The function to call. |
| `timeout` | The timeout to wait. |
| `resetTimer` | Whether to reset the timeout when the debouncer is called again. |
