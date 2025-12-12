For file specific ccs, add `ccsclass` as a metadata field with a value referencing a user-defined CSS class.

User-defined CSS snippets are saved in the `.obsidian/snippets/` directory.

[Obsidian Forum](https://forum.obsidian.md/t/custom-css-to-disable-inline-title/53645/2?u=beanstock13) post about hiding an inline title:

At the top of your note, you could add a front-matter `yaml` block w/ a defined CSS class. For example:

```
---
cssclasses: my-class
---`
```

Then, you could add a custom CSS snippet, with the CSS rule:

```css
.my-class .inline-title{
  display: none; }`
```

That should disable the display of the inline title. In addition, you can use the same `cssclass` in any other notes.

Hope that helps.
