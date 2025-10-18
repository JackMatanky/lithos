---
title:
  - Obsidian_to_Anki
aliases:
  - Obsidian_to_Anki
  - obsidian_to_anki
  - obsidian_obsidian_to_anki
  - obsidian_plugin_community_obsidian_to_anki
application: obsidian_to_anki
url: https://github.com/Pseudonium/Obsidian_to_Anki
file_class: lib_documentation
date_created: 2023-05-30T19:03
date_modified: 2023-10-25T16:22
tags: obsidian, obsidian/plugin, obsidian/obsidian_to_anki, anki, spaced_repetition, obsidian/spaced_repetition, flashcard
---
# Obsidian_to_Anki

link:: <https://github.com/Pseudonium/Obsidian_to_Anki>

Plugin to add flashcards from a text or markdown file to Anki. Run in Obsidian as a plugin, or from the command-line as a python script. Built with [Obsidian](https://obsidian.md/) markdown syntax in mind. Supports **user-defined custom syntax for flashcards.**  
See the [Trello](https://trello.com/b/6MXEizGg/obsidiantoanki) for planned features.

## Getting Started

Check out the [Wiki](https://github.com/Pseudonium/Obsidian_to_Anki/wiki)! It has a ton of information, including setup instructions for new users. I will include a copy of the instructions here:

## Setup

### All Users

1. Start up [Anki](https://apps.ankiweb.net/), and navigate to your desired profile.
2. Ensure that you've installed [AnkiConnect](https://github.com/FooSoft/anki-connect).

### Obsidian Plugin Users

1. Have [Obsidian](https://obsidian.md/) downloaded
2. Search the 'Community plugins' list for this plugin
3. Install the plugin.
4. In Anki, navigate to Tools->Addons->AnkiConnect->Config, and change it to look like this:

```
{
    "apiKey": null,
    "apiLogPath": null,
    "webBindAddress": "127.0.0.1",
    "webBindPort": 8765,
    "webCorsOrigin": "http://localhost",
    "webCorsOriginList": [
        "http://localhost",
        "app://obsidian.md"
    ]
}
```

1. Restart Anki to apply the above changes
2. With Anki running in the background, load the plugin. This will generate the plugin settings.

You shouldn't need Anki running to load Obsidian in the future, though of course you will need it for using the plugin!

To run the plugin, look for an Anki icon on your ribbon (the place where buttons such as 'open Graph view' and 'open Quick Switcher' are).  
For more information on use, please check out the [Wiki](https://github.com/Pseudonium/Obsidian_to_Anki/wiki)!

### Python Script Users

1. Install the latest version of [Python](https://www.python.org/downloads/).
2. If you are a new user, download `obstoanki_setup.py` from the [releases page](https://github.com/Pseudonium/Obsidian_to_Anki/releases), and place it in the folder you want the script installed (for example your notes folder).  
3. Run `obstoanki_setup.py`, for example by double-clicking it in a file explorer. This will download the latest version of the script and required dependencies automatically. Existing users should be able to run their existing `obstoanki_setup.py` to get the latest version of the script.  
4. Check the Permissions tab below to ensure the script is able to run.
5. Run `obsidian_to_anki.py`, for example by double-clicking it in a file explorer. This will generate a config file, `obsidian_to_anki_config.ini`.

#### Permissions

The script needs to be able to:

- Make a config file in the directory the script is installed.
- Read the file in the directory the script is used.
- Make a backup file in the directory the script is used.
- Rename files in the directory the script is used.
- Remove a backup file in the directory the script is used.
- Change the current working directory temporarily (so that local image paths are resolved correctly).

## Features

Current features (check out the wiki for more details):

- **Custom note types** - You're not limited to the 6 built-in note types of Anki.
- **Updating notes from file** - Your text files are the canonical source of the notes.
- **Tags**, including **tags for an entire file**.
- **Adding to user-specified deck** on a *per-file* basis.
- **Markdown formatting**.
- **Math formatting**.
- **Embedded images**. GIFs should work too.
- **Audio**.
- **Auto-deleting notes from the file**.
- **Reading from all files in a directory automatically** - recursively too!
- **Inline Notes** - Shorter syntax for typing out notes on a single line.
- **Easy cloze formatting** - A more compact syntax to do Cloze text
- **Frozen Fields**
- **Obsidian integration** - A link to the file that made the flashcard, full link and image embed support.
- **Custom syntax** - Using **regular expressions**, add custom syntax to generate **notes that make sense for you.** Some examples:
  - RemNote single-line style. `This is how to use::Remnote single-line style`  
    ![[Remnote_1.webp]]
    
   - Header paragraph style.

  ```
  # Style
  This style is suitable for having the header as the front, and the answer as the back
  ```  

  ![[Header_1.webp]]

  - Question answer style.

  ```
  Q: How do you use this style?
  A: Just like this.
  ```  

  ![[Question_1.webp]]

  - Neuracache #flashcard style.  

  ```
  In Neuracache style, to make a flashcard you do #flashcard
  The next lines then become the back of the flashcard
  ```  

  ![[Neuracache_1.webp]]

  - Ruled style  

  ```
  How do you use ruled style?
  ---
  You need at least three '-' between the front and back of the card.
  ```  

  ![[Ruled_1.webp]]

  - Markdown table style  

  ```

  | Why might this style be useful? |
  | ------ |
  | It looks nice when rendered as HTML in a markdown editor. |

  ```

  ![[Table_2.webp]]
  
  - Cloze paragraph style  

  ```

  The idea of {cloze paragraph style} is to be able to recognise any paragraphs that contain {cloze deletions}.

  ```

  ![[Cloze_1.webp]]

Note that **all custom syntax is off by default**, and must be programmed into the script via the config file - see the Wiki for more details.

---

## Settings

### Syntax Settings

Begin Note: START
End Note: END
Begin Inline Note: START|
End Inline Note: END|
Target Deck Line: TARGET DECK
File Tags Line: TAGS
Delete Note Line: DELETE
Frozen Fields Line: FROZEN

### Defaults

Tag: anki_flashcard
Deck: Default
Scheduling Interval:
Add File Link: True
Add Context: True
CurlyCloze: False
CurlyCloze - Highlights to Clozes: False
ID Comments: True
Add Obsidian Tags: False

- Nested Deck Syntax
	- Parent Deck::Child Deck

## Note Types and Fields

1. basic_center
	1. front
	2. back
	3. reference
	4. tags
	5. context
	6. obsidian_link
2. basic_left
	1. front
	2. back
	3. reference
	4. tags
	5. context
	6. obsidian_link
3. basic_rtl
	1. front
	2. back
	3. reference
	4. tags
	5. context
	6. obsidian_link
4. cloze_center
	1. front
	2. reference
	3. tags
	4. context
	5. obsidian_link
5. cloze_left
	1. front
	2. reference
	3. tags
	4. context
	5. obsidian_link
6. cloze_code
	1. front
	2. reference
	3. tags
	4. context
	5. obsidian_link
7. cloze_image
	1. front
	2. image
	3. reference
	4. tags
	5. context
	6. obsidian_link
8. spelling
	1. front
	2. reference
	3. tags
	4. context
	5. obsidian_link
