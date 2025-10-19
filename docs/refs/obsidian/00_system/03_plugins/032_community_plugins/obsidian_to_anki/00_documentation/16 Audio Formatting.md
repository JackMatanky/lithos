---
title:
  - 16 Audio Formatting
aliases:
  - 16 Audio Formatting
  - Audio Formatting
  - audio_formatting
  - obsidian_to_anki_audio_formatting
application: obsidian_to_anki
url: https://github.com/Pseudonium/Obsidian_to_Anki/wiki/Audio-formatting
file_class: lib_documentation
date_created: 2023-05-30T19:13
date_modified: 2023-10-25T16:22
tags:
---
# Audio Formatting

## Obsidian Plugin Users

Embedded audio is supported via doing Obsidian's standard `![[embed]]` syntax.

## Python Script Users

Embedded audio is supported if the following criteria are met:

1. The audio file is stored locally
2. It is embedded using the syntax `[sound:{path_to_file}]`. So, for example, if the filename was `record.wav` and it was in a `Media` folder, you'd write `[sound:Media/record.wav]`
