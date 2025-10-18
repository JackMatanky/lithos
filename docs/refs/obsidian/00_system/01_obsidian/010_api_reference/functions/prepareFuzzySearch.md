---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# prepareFuzzySearch

```ts
export function prepareFuzzySearch(query: string): (text: string) => SearchResult | null;
```

Construct a fuzzy search callback that runs on a target string.  
Performance may be an issue if you are running the search for more than a few thousand times.  
If performance is a problem, consider using `prepareSimpleSearch` instead.

## Parameters

| Parameter | Description |
|-----------|-------------|
| `query` | the fuzzy query. |
