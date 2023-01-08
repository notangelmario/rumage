# Rumage

Rumage is a simple web framework. It converts Markdown files to plain HTML.

⚠️ Rumage is not fully production ready! It is pretty stable,
but it needs more error handling. You could use it, but keep
in mind that if you encounter an error, it might be harder
to troubleshoot. I use it for my [personal website](https://angelmario.eu).

## Features

* ⚡️ Blazingly fast
* ✨ Simple by design
* 📄 Markdown and HTML
* 💔 ZERO JavaScript
* 🧑‍💻 Customizable

## Installation

Make sure you have [installed cargo](https://rustup.rs/) and install rumage like this:
```
cargo install rumage
```

[or check releases](https://github.com/notangelmario/rumage/releases)

## Usage

For building:
```
rumage build
```

## Style

To add style to your website, add a `style.css` in your source folder. Rumage
will automatically pick it up and include it in all your pages.

## Head tag

By default, the head tag contains a customisable title and description
property, a favicon and stylesheet.

Default name for favicon is `favicon.png` and the global stylesheet is `style.css`.

To customise the head of every page, add a `_head.html` in your source folder.

You can use `%property%` in the html to replace it with custom values from markdown
pages.

Example:

`_head.html`
```html
<head>
    <title>%title</title>
    <meta name="description" content="%description%" />
    <meta name="tags" content="%tags%" />
</head>
```

`index.md`
```html
---
title: Home
description: The homepage
tags: tech, typescript, web, rust
---

# Hello world
```

## Routing

The routing is very simple, all files represent a route.

## Custom components

> Planned

## Nav & Footer

To add a navbar or a footer, create a `_nav.html` or `_nav.md` for a navbar,
and `_footer.html` or `_footer.md` for a footer.

If those files are present in the source folder, all pages will contain navbar
and footer by default. To disable this, add `nav: false` or `footer: false`
at the top of the file.

```
---
nav: false
footer: false
---
```
