---
title:
  - 21 Config Module
aliases:
  - 21 Config Module
  - templater_documentation_21_config_module
application: templater
url: 
file_class: lib_documentation
date_created: 2023-03-10T15:44
date_modified: 2023-10-25T16:22
tags: 
---
# Config Module

This module exposes Templater's running configuration.

This is mostly useful when writing scripts requiring some context information.

## Documentation

### `tp.config.active_file?`

The active file (if existing) when launching Templater.

### `tp.config.run_mode`

The `RunMode`, representing the way Templater was launched (Create new from template, Append to active file, …)

### `tp.config.target_file`

The `TFile` object representing the target file where the template will be inserted.

### `tp.file.template_file`

The `TFile` object representing the template file.
