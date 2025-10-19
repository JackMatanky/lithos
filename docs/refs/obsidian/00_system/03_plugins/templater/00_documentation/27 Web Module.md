---
title:
  - 27 Web Module
aliases:
  - 27 Web Module
  - templater_documentation_27_web_module
application: templater
url:
file_class: lib_documentation
date_created: 2023-03-10T15:45
date_modified: 2023-10-25T16:22
tags:
---
# Web Module

This modules contains every internal function related to the web (making web requests).

## Documentation

Function documentation is using a specific syntax. More information [here](../../syntax.md#function-documentation-syntax)

### `tp.web.daily_quote()`

Retrieves and parses the daily quote from the API <https://api.quotable.io>

### `tp.web.random_picture(size?: String, Query?: String, include_size?: boolean)`

Gets a random image from <https://unsplash.com/>

#### Arguments

- `include_size`: Optional argument to include the specified size in the image link markdown. Defaults to false

- `query`: Limits selection to photos matching a search term. Multiple search terms can be passed separated by a comma `,`

- `size`: Image size in the format `<width>x<height>`

## Examples

```javascript
Web Daily quote:
 > There are basically two types of people. People who accomplish things, and people who claim to have accomplished things. The first group is less crowded.
> — <cite>Mark Twain</cite>

Web Random picture:
 ![photo by Rian A. Saputro on Unsplash](https://images.unsplash.com/photo-1692422986086-365ddb07c504?crop=entropy&cs=srgb&fm=jpg&ixid=M3wzNjM5Nzd8MHwxfHJhbmRvbXx8fHx8fHx8fDE2OTI4Njg3NjB8&ixlib=rb-4.0.3&q=85)

Web Random picture with size:
 ![photo by Lisa van Vliet on Unsplash](https://images.unsplash.com/photo-1690456042533-9359b8d12222?crop=entropy&cs=srgb&fm=jpg&ixid=M3wzNjM5Nzd8MHwxfHJhbmRvbXx8fHx8fHx8fDE2OTI4Njg3NjB8&ixlib=rb-4.0.3&q=85&w=200&h=200)

Web random picture with size + query:
 ![photo by Dylan Gialanella on Unsplash](https://images.unsplash.com/photo-1447757052757-5391690dca67?crop=entropy&cs=srgb&fm=jpg&ixid=M3wzNjM5Nzd8MHwxfHJhbmRvbXx8fHx8fHx8fDE2OTI4Njg3NjB8&ixlib=rb-4.0.3&q=85&w=200&h=200)
```
