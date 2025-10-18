---
date_created: 2023-08-24T21:58
date_modified: 2023-10-25T16:22
---
# Problem

Every time I try syncing Obsidian and Anki with the [[Obsidian_to_Anki]] plugin, the console shows the following error:

```
plugin:obsidian-to-anki-plugin:589 Uncaught (in promise) TypeError: this.field_names is not iterable
    at Note.getFields (plugin:obsidian-to-anki-plugin:589:32)
    at AllFile.setup_frozen_fields_dict (plugin:obsidian-to-anki-plugin:30315:151)
    at AllFile.setupScan (plugin:obsidian-to-anki-plugin:30435:14)
    at AllFile.scanFile (plugin:obsidian-to-anki-plugin:30539:14)
    at FileManager.initialiseFiles (plugin:obsidian-to-anki-plugin:30665:22)
    at async MyPlugin.scanVault (plugin:obsidian-to-anki-plugin:30958:9)
    at async HTMLDivElement.eval (plugin:obsidian-to-anki-plugin:30995:13)
getFields @ plugin:obsidian-to-anki-plugin:589
setup_frozen_fields_dict @ plugin:obsidian-to-anki-plugin:30315
setupScan @ plugin:obsidian-to-anki-plugin:30435
scanFile @ plugin:obsidian-to-anki-plugin:30539
initialiseFiles @ plugin:obsidian-to-anki-plugin:30665
```

I tried the solution proposed by [dmantula](https://github.com/Pseudonium/Obsidian_to_Anki/issues/265#issuecomment-1257520568) for Linux:

> What did I do to fix the issue:
> 1. changed my note types and fields so that they don't have any special characters other than dashes.
> 2. Uninstalled and installed the plugin from scratch, synchronized everything again.  
> And it worked!

However, this solution did not work in my case.

# Proposed Solution

After searching some more, I found the Obsidian_to_Anki thread on the Obsidian [forum](https://forum.obsidian.md/t/obsidian-to-anki-v3-4-0-a-feature-rich-plugin-that-allows-you-to-add-notes-from-obsidian-to-anki/5030?u=beanstock13). [There](https://forum.obsidian.md/t/obsidian-to-anki-v3-4-0-a-feature-rich-plugin-that-allows-you-to-add-notes-from-obsidian-to-anki/5030/156?u=beanstock13), I found another person received the same error and was directed to an [issue](https://github.com/Pseudonium/Obsidian_to_Anki/issues/177) on the Obsidian_to_Anki GitHub page. In that case, a user was not able to create cards for a note type they had renamed earlier.

The three solutions given:

1. Manually going into `.obsidian/plugins/obsidian-to-anki-plugin/data.json`, removing instances of the new name and renaming instances of the old name to the new one.
2. As a temporary workaround, open the `data.json` file located in `.obsidian/plugins/obsidian-to-anki-plugin` and edit it manually, save the file, and restart obsidian.
3. Replace `const field_names = plugin.fields_dict[note_type];` by `const field_names = await invoke('modelFieldNames', { modelName: note_type });` in `main.js`. ***This is not an ideal fix but is one that works for the time being.***

# Result

I synced Obsidian to Anki to recreate the bug and, instead, the sync worked…
