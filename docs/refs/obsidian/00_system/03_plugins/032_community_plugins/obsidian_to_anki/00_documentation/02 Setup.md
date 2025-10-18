---
title:
  - 02 Setup
aliases:
  - 02 Setup
  - Setup
  - setup
  - obsidian_to_anki_setup
application: obsidian_to_anki
url: https://github.com/Pseudonium/Obsidian_to_Anki/wiki/Setup
file_class: lib_documentation
date_created: 2023-05-30T19:13
date_modified: 2023-10-25T16:22
tags: 
---
# Setup

## All Users

1. Start up [Anki](https://apps.ankiweb.net/), and navigate to your desired profile.
2. Ensure that you've installed [AnkiConnect](https://github.com/FooSoft/anki-connect).

## Obsidian Plugin Users

1. Have [Obsidian](https://obsidian.md/) downloaded
2. Search the 'Community plugins' list for this plugin
3. Install the plugin.
4. In Anki, navigate to Tools->Addons->AnkiConnect->Config, and change it to look like this: ![[AnkiConnect_ConfigREAL.webp]]
5. Restart Anki to apply the above changes.
6. With Anki running in the background, load the plugin. This will generate the plugin settings.

You shouldn't need Anki running to load Obsidian in the future, though of course you will need it for using the plugin!

## Python Script Users

1. Install the latest version of [Python](https://www.python.org/downloads/).
2. If you are a new user, download `obstoanki_setup.py` from the [releases page](https://github.com/Pseudonium/Obsidian_to_Anki/releases), and place it in the folder you want the script installed (for example your notes folder).  
3. Run `obstoanki_setup.py`, for example by double-clicking it in a file explorer. This will download the latest version of the script and required dependencies automatically. Existing users should be able to run their existing `obstoanki_setup.py` to get the latest version of the script.  
4. Check the Permissions tab below to ensure the script is able to run.
5. Run `obsidian_to_anki.py`, for example by double-clicking it in a file explorer. This will generate a config file, `obsidian_to_anki_config.ini`.

### Permissions

The script needs to be able to:

- Make a config file in the directory the script is installed.
- Read the file in the directory the script is used.
- Make a backup file in the directory the script is used.
- Rename files in the directory the script is used.
- Remove a backup file in the directory the script is used.
- Change the current working directory temporarily (so that local image paths are resolved correctly).
