---
aliases:
  - 
title: Embedding web pages
subtitle: 
author: 
date_published: 
publisher: Obsidian Help
url: https://publish.obsidian.md/
type: documentation
file_class: lib_documentation
cssclasses:
date_created: 2023-03-21T12:12
date_modified: 2023-09-05T19:18
tags: obsidian, obsidian/markdown
---
# Embedding Web Pages

Learn how to use the [iframe](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/iframe) HTML element to embed web pages in your notes.

To embed a web page, add the following in your note and replace the placeholder text with the URL of the web page you want to embed:

```
<iframe src="INSERT YOUR URL HERE"></iframe>
```

> [!Note]  
> Some websites don't allow you to embed them. Instead, they may provide URLs that are meant for embedding them. If the website doesn't support embedding, try searching for the name of the website followed by "embed iframe". For example, "youtube embed iframe".

> [!Tip]  
> If you're using [Canvas](https://help.obsidian.md/Plugins/Canvas), you can embed a web page in a card. For more information, refer to [Canvas > Add cards from web pages](https://help.obsidian.md/Plugins/Canvas#Add%20cards%20from%20web%20pages).

## Embed a YouTube Video

To embed a YouTube video, use the same Markdown syntax as [external images](https://help.obsidian.md/Editing+and+formatting/Basic+formatting+syntax#External%20images):

```md
![](https://www.youtube.com/watch?v=NnTvZWp5Q7o)
```

YouTube doesn't allow you to embed a video using the regular URL. Instead, use `https://www.youtube.com/embed/VIDEO_ID`.

You can find the video ID by browsing to the video and looking in the address bar in your browser. The video ID is the text that comes after `?v=`.

For example, to embed the video at `https://www.youtube.com/watch?v=NnTvZWp5Q7o`, add the following to your note:

```
<iframe src="https://www.youtube.com/embed/NnTvZWp5Q7o"></iframe>
```

<iframe src="https://www.youtube.com/embed/NnTvZWp5Q7o" sandbox="allow-forms allow-presentation allow-same-origin allow-scripts allow-modals"></iframe>

## Embed a Tweet

While Twitter doesn't have an official way to embed tweets using iframe, you can use services like [TwitFrame](https://twitframe.com/) to generate an embeddable URL. For more information, refer to TwitFrame's own documentation.

```
<iframe
  border="0"
  frameborder="0"
  height="763"
  width="550"
  src="https://twitframe.com/show?url=https%3A%2F%2Ftwitter.com%2Fobsdmd%2Fstatus%2F1580548874246443010"
>
</iframe>
```
