# Archive provenance and version limits

This file qualifies the bundled references against the source snapshots already stored in `research/*.txt`. The application does not make network requests to refresh them, inspect local package databases, run the referenced tools, or compare the references with an installed version. The documentation links in the app may resolve to newer upstream content than these archived files.

## Snapshot manifest

`research/sources.json` maps each upstream URL to the local snapshot and records version/date fields only when the snapshot itself states them. The application repeats the supported identifiers in its source/version display; a local review date remains **Not recorded** for every upstream tool.

| Tool | Included source | Evidence embedded in snapshot | UI version value | Local review date |
|---|---|---|---|---|
| Git | `git.txt` | No immutable release/commit identifier in the saved documentation page | Not recorded | Not recorded |
| SSH | `openssh.txt` | No immutable release identifier in the saved man page | Not recorded | Not recorded |
| curl | `curl.txt` | The manual says: “This man page describes curl 8.23.0.” No source date is stated | `curl 8.23.0 man page snapshot` | Not recorded |
| tmux | `tmux.txt` | Header identifies `tmux.1` revision `1.1181`, dated `2026-10-08 07:50:05` | `tmux.1 revision 1.1181 (2026-10-08)`; this is a man-page revision, not a claimed package version | Not recorded |
| Vim | `vim.txt` | Header says Vim `9.2`, last change `2026 May 17` | `Vim 9.2 reference (last change 2026-05-17)` | Not recorded |
| fzf | `fzf.txt` | Saved README points to moving `master`/release badge; no pinned release or commit | Not recorded | Not recorded |
| ripgrep | `ripgrep.txt` | Saved guide has no pinned release or commit identifier | Not recorded | Not recorded |
| fd | `fd.txt` | README contains release examples, but no exact identity for the saved snapshot; examples are not treated as version evidence | Not recorded | Not recorded |
| jq | `jq.txt` | Manual/development page does not identify an exact snapshot release | Not recorded | Not recorded |
| htop | `htop.txt` | `@PACKAGE_STRING@` is an unexpanded package placeholder; copyright years are not release versions | Not recorded | Not recorded |
| Ghostty | `../shortcuts.json` | Bundled shortcut list has no pinned Ghostty release identifier. User-config mode parses explicit lines only | Not recorded | Not recorded |

The tmux date and Vim date above are source revision/change dates, **not** Clavis review dates. No date is synthesized from this checkout's filesystem timestamps. `Not recorded` is preferable to presenting a moving source as current or guessing from an example/package badge.

## Curated safety notes

Every one of the 94 tool rows in `catalog.json` and 15 bundled Ghostty rows is classified entry by entry in `catalog-risk-review.json`: 27 exact-key inline warnings and 82 explicit no-warning decisions. A Rust test verifies a one-to-one match against both bundled snapshots, verifies all warning text against `src/catalog.rs`, and checks the recorded content fingerprints. Changing catalog rows requires a deliberate re-review and an explicit inventory/fingerprint update.

Curated inline warnings cover selected repository changes, remote access/port forwarding and resolved SSH configuration, curl output overwrite and POST behavior, Vim deletion/write actions, fzf shell-history/preview behavior, hidden/ignored file search, tmux session attachment, htop signals, and terminal close/paste shortcuts. Placeholder highlighting/reminders are generic; personal references may also include a user-written warning. This list is intentionally conservative and incomplete: it is not shell parsing, command validation, or a guarantee that an unmarked item is safe. Clavis copies references only and never executes them.
