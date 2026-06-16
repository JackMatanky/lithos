# Discovery

The Discovery context locates the runtime filesystem context needed before configuration can be loaded. It answers: where is the vault, and where are its config files?

## Language

**Vault Root**:
The directory that bounds a Lithos vault and serves as the base for local config path resolution.
_Avoid_: project root, workspace root

**Root Marker**:
A conventional config filename whose presence in a directory establishes that directory as the Vault Root.
_Avoid_: config location, index marker, marker file

**Candidate Marker**:
A config file found during traversal or global resolution, ordered by source precedence but not yet confirmed as the config input.
_Avoid_: discovered config, selected config, result file

**Ascending Walk**:
The traversal strategy that searches parent directories upward from a starting anchor until a Root Marker is found or a boundary stops the walk.
_Avoid_: upward scan, directory crawl, recursive search

**Ceiling**:
A directory boundary that stops the Ascending Walk, preventing discovery from searching above a declared limit.
_Avoid_: stop path, upper bound, limit dir

**Override**:
An explicit Vault Root path supplied via CLI flag or environment variable that preempts traversal entirely.
_Avoid_: forced path, hardcoded path, explicit config

**Discovery Result**:
The output of a discovery run: the located Vault Root and all ranked Candidate Markers, ready for consumption by Config.
_Avoid_: discovery output, found configs, resolved paths

**Discovery Report**:
Process metadata produced alongside the Discovery Result, capturing skipped Overrides, skipped Ceilings, and why the Ascending Walk stopped. Consumed by the Bootstrapper for diagnostics only; downstream contexts never see it.
_Avoid_: discovery log, diagnostics, discovery warnings

## Example Dialogue

> **Dev**: The user ran `lithos sync` from inside their vault. How does Discovery find the vault root?
>
> **Domain expert**: Discovery starts an Ascending Walk from the current directory. It checks each parent directory for a Root Marker — a file like `lithos.toml`. The first directory where it finds one becomes the Vault Root.
>
> **Dev**: What if the user also set `LITHOS_VAULT`?
>
> **Domain expert**: That's an Override. It preempts the Ascending Walk entirely. Discovery validates the path and returns it as the Vault Root without walking at all.
>
> **Dev**: And if the walk goes too far up?
>
> **Domain expert**: A Ceiling stops it. If the walk reaches a Ceiling directory — or a project boundary like `.git` — it stops, even if no Root Marker was found.
>
> **Dev**: What does Discovery hand off to Config?
>
> **Domain expert**: A Discovery Result: the Vault Root and the ordered list of Candidate Markers. Config picks the winner and parses it. The Discovery Report goes to the Bootstrapper for surfacing diagnostics — Config never sees it.
