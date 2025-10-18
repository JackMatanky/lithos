---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# AjaxOptions

## Properties

### Method

```ts
method: "GET" | "POST"
```

### Url

```ts
url: string
```

### Success

```ts
success: (response: any, req: XMLHttpRequest) => any
```

### Error

```ts
error: (error: any, req: XMLHttpRequest) => any
```

### Data

```ts
data: string | object | ArrayBuffer
```

### Headers

```ts
headers: Record<string, string>
```

### withCredentials

```ts
withCredentials: boolean
```

### Req

```ts
req: XMLHttpRequest
```
