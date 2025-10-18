---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# prepareSimpleSearch

```ts
export function prepareSimpleSearch(query: string): (text: string) => SearchResult | null;
```

Construct a simple search callback that runs on a target string.

## Parameters

| Parameter | Description |
|-----------|-------------|
| `query` | the space-separated words |
