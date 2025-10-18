---
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

```js
dataviewjs
const {update} = this.app.plugins.plugins["metaedit"].api
const {createButton} = app.plugins.plugins["buttons"]

dv.table(["Name", "Status", "Project", "Due Date", ""], dv.pages("#tasks")
    .sort(t => t["due-date"], 'desc')
    .where(t => t.status != "Completed")
    .map(t => [t.file.link, t.status, t.project, t["due-date"], 
    createButton({app, el: this.container, args: {name: "Done!"}, clickOverride: {click: update, params: ['Status', 'Completed', t.file.path]}})])
    )
```

![CBrFA0qHr4](https://user-images.githubusercontent.com/29108628/119342641-ab003a80-bc95-11eb-8f0a-15a6ced6b36d.gif)
