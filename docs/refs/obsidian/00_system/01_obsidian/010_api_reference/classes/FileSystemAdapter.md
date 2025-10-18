---
date_created: 2023-05-19T13:24
date_modified: 2023-09-05T19:18
---
# FileSystemAdapter

Implements `DataAdapter`

## Constructor

```ts
constructor();
```

## Methods

### getName

```ts
getName(): string;
```

### getBasePath

```ts
getBasePath(): string;
```

### Mkdir

```ts
mkdir(normalizedPath: string): Promise<void>;
```

### trashSystem

```ts
trashSystem(normalizedPath: string): Promise<boolean>;
```

### trashLocal

```ts
trashLocal(normalizedPath: string): Promise<void>;
```

### Rmdir

```ts
rmdir(normalizedPath: string, recursive: boolean): Promise<void>;
```

### Read

```ts
read(normalizedPath: string): Promise<string>;
```

### readBinary

```ts
readBinary(normalizedPath: string): Promise<ArrayBuffer>;
```

### Write

```ts
write(normalizedPath: string, data: string, options?: DataWriteOptions): Promise<void>;
```

### writeBinary

```ts
writeBinary(normalizedPath: string, data: ArrayBuffer, options?: DataWriteOptions): Promise<void>;
```

### Append

```ts
append(normalizedPath: string, data: string, options?: DataWriteOptions): Promise<void>;
```

### Process

```ts
process(normalizedPath: string, fn: (data: string) => string, options?: DataWriteOptions): Promise<string>;
```

### getResourcePath

```ts
getResourcePath(normalizedPath: string): string;
```

### getFilePath

```ts
getFilePath(normalizedPath: string): string;
```

Returns the file:// path of this file

### Remove

```ts
remove(normalizedPath: string): Promise<void>;
```

### Rename

```ts
rename(normalizedPath: string, normalizedNewPath: string): Promise<void>;
```

### Copy

```ts
copy(normalizedPath: string, normalizedNewPath: string): Promise<void>;
```

### Exists

```ts
exists(normalizedPath: string, sensitive?: boolean): Promise<boolean>;
```

### Stat

```ts
stat(normalizedPath: string): Promise<Stat | null>;
```

### List

```ts
list(normalizedPath: string): Promise<ListedFiles>;
```

### getFullPath

```ts
getFullPath(normalizedPath: string): string;
```

### readLocalFile

```ts
static readLocalFile(path: string): Promise<ArrayBuffer>;
```

### Mkdir

```ts
static mkdir(path: string): Promise<void>;
```
