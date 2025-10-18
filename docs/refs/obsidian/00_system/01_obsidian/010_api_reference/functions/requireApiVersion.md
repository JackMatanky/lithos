---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# requireApiVersion

```ts
export function requireApiVersion(version: string): boolean;
```

Returns true if the API version is equal or higher than the requested version.  
Use this to limit functionality that require specific API versions to avoid  
crashing on older Obsidian builds.

## Parameters

| Parameter | Description |
|-----------|-------------|
| `version` | |
