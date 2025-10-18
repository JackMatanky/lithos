---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# String

Allows manipulation and formatting of text strings and determination and location of substrings within strings.

## Methods

### Contains

```ts
contains: (target: string) => boolean
```

### startsWith

```ts
startsWith: { (searchString: string, position?: number): boolean; (searchString: string, position?: number): boolean; }
```

Returns true if the sequence of elements of searchString converted to a String is the  
same as the corresponding elements of this object (converted to a String) starting at  
position. Otherwise returns false.

### endsWith

```ts
endsWith: { (searchString: string, endPosition?: number): boolean; (target: string, length?: number): boolean; }
```

Returns true if the sequence of elements of searchString converted to a String is the  
same as the corresponding elements of this object (converted to a String) starting at  
endPosition – length(this). Otherwise returns false.

### Format

```ts
format: (...args: string[]) => string
```
