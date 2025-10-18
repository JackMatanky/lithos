---
aliases:
  - Text Formatting_Obsidian Markdown Syntax
title: Format your notes - Obsidian Help
author: 
date_published: 
publisher: Obsidian Help
url: https://publish.obsidian.md/
type: documentation
file_class: lib_documentation
cssclasses:
date_created: 2023-03-21T12:22
date_modified: 2023-09-05T19:18
tags: markdown/obsidian, obsidian, markdown
---
# Text Formatting

## Italic

```
*This text will be italic*

_This will also be italic_
```

*This text will be italic*

*This will also be italic*

## Bold

```
**This text will be bold**

__This will also be bold__
```

**This text will be bold**

**This will also be bold**

## Italicized Bold

```
_You **can** combine them_
```

*You **can** combine them*

## Strike-through

```
Any word wrapped with two tildes (like ~~this~~) will appear crossed out.
```

Any word wrapped with two tildes (like ~~this~~) will appear crossed out.

## Highlighting

```
Use two equal signs to ==highlight text==.
```

Use two equal signs to ==highlight text==.

## Horizontal Bar

Use three stars \*\*\*, hyphens ---, or underscores \_\_\_ in a new line to produce an horizontal bar.

## Blockquotes

```
> Human beings face ever more complex and urgent problems, and their effectiveness in dealing with these problems is a matter that is critical to the stability and continued progress of society.

\- Doug Engelbart, 1961
```

> Human beings face ever more complex and urgent problems, and their effectiveness in dealing with these problems is a matter that is critical to the stability and continued progress of society.

\- Doug Engelbart, 1961

## Footnotes

```
Here's a simple footnote,[^1] and here's a longer one.[^bignote]

[^1]: meaningful!

[^bignote]: Here's one with multiple paragraphs and code.
    Indent paragraphs to include them in the footnote.

    `{ my code }`

    Add as many paragraphs as you like.
```

Here's a simple footnote,<sup data-footnote-id="fnref-1-a0898fd4b1575159" id="fnref-1-a0898fd4b1575159"><a href="https://publish.obsidian.md/#fn-1-a0898fd4b1575159" target="_blank" rel="noopener">[1]</a></sup> and here's a longer one.<sup data-footnote-id="fnref-2-a0898fd4b1575159" id="fnref-2-a0898fd4b1575159"><a href="https://publish.obsidian.md/#fn-2-a0898fd4b1575159" target="_blank" rel="noopener">[2]</a></sup>

```
You can also use inline footnotes. ^[notice that the caret goes outside of the brackets on this one.]
```

You can also use inline footnotes. <sup data-footnote-id="fnref-3-a0898fd4b1575159" id="fnref-3-a0898fd4b1575159"><a href="https://publish.obsidian.md/#fn-3-a0898fd4b1575159" target="_blank" rel="noopener">[3]</a></sup>

## Tables

You can create tables by assembling a list of words and dividing the header from the content with hyphens, `-`, and then separating each column with a pipe `|`:

```
|First Header | Second Header|
|------------ | ------------|
|Content from cell 1 | Content from cell 2|
|Content in the first column | Content in the second column|
```

| First Header | Second Header |
| --- | --- |
| Content from cell 1 | Content from cell 2 |
| Content in the first column | Content in the second column |

The vertical bars at the start and end of a line are optional.

```
First Header | Second Header
------------ | ------------
Content from cell 1 | Content from cell 2
Content in the first column | Content in the second column
```

This results in the same table as the one above.

```
Tables can be justified with a colon | Another example with a long title | And another long title as a example
:----------------|-------------:|:-------------:
because of the `:` | these will be justified |this is centered
```

| Tables can be justified with a colon | Another example with a long title | And another long title as a example |
| --- | --- | --- |
| because of the `:` | these will be justified | this is centered |

If you put links in tables, they will work, but if you use [aliases](https://help.obsidian.md/Linking+notes+and+files/Aliases), the pipe must be escaped with a `\` to prevent it being read as a table element.

```
First Header | Second Header
------------ | ------------
[[Format your notes\|Formatting]]|  [[Keyboard shortcuts\|hotkeys]]
```

If you want to resize images in tables, you need to escape the pipe with a `\`:

```
Image | Description
----- | -----------
![[og-image.png\|200]] | Obsidian
```

| Image | Description |
| --- | --- |
|![](https://publish-01.obsidian.md/access/f786db9fac45774fa4f0d8112e232d67/og-image.png) | Obsidian |

## Math

```
$$\begin{vmatrix}a & b\\
c & d
\end{vmatrix}=ad-bc$$
```

```
You can also do inline math like $e^{2i\pi} = 1$ .
```

You can also do inline math like e2iπ\=1.

## Code

### Inline Code

```
Text inside `backticks` on a line will be formatted like code.
```

Text inside `backticks` on a line will be formatted like code.

### Code Blocks

You can add syntax highlighting to a code block by adding a language code after the first set of backticks.

Obsidian uses Prism for syntax highlighting. For more information, refer to [Supported languages](https://prismjs.com/#supported-languages).

[Live Preview mode](https://help.obsidian.md/Live+preview+update) doesn't support PrismJS and may render syntax highlighting differently.

```js
function fancyAlert(arg) {
  if(arg) {
    $.facebox({div:'#foo'})
  }
}
```

```
function fancyAlert(arg) {
  if(arg) {
    $.facebox({div:'#foo'})
  }
}
```

```
    Text indented with a tab is formatted like this, and will also look like a code block in preview.
```

```
Text indented with a tab is formatted like this, and will also look like a code block in preview.
```

## Comments

Use `%%` to enclose comments, which will be parsed as Markdown, but won't show up in the preview.

```
Here is some inline comments: %%You can't see this text%% (Can't see it in Reading mode)

Here is a block comment: (can't see it in Reading mode either)
%%
It can span
multiple lines
%%
```

Here is some inline comments: (can't see it in Reading mode)

Here is a block comment: (can't see it in Reading mode either)
