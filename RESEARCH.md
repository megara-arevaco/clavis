# Clavis — terminal reference selection

## What this research can and cannot establish

There is no comparable, public global usage dataset covering every terminal program. Repository stars, package downloads and surveys measure different audiences, not command invocation frequency. These ten entries are a **curated practical selection**, not a measured top-ten ranking. They cover everyday development jobs without requiring the user to install any of the referenced programs.

Official upstream documentation was fetched and consulted while building this catalog. The local research snapshots are development-only; the shipped app embeds reference entries and links, not entire manuals. It performs no runtime network requests and does not inspect shell history.

Source archive details and the evidence-supported version/date fields are in [`research/RESEARCH.md`](research/RESEARCH.md) and [`research/sources.json`](research/sources.json). The shipped UI identifies these as archived references and explicitly says the currently installed tool version has not been checked. It does not query the network or claim that moving upstream pages or local packages match the archive. Versions and source dates remain **Not recorded** unless the included source text itself states them; local review dates are not inferred from file timestamps.

| Tool | Job covered | Primary reference | Important qualification |
|---|---|---|---|
| Git | Version control | https://git-scm.com/docs | Commands can modify a repository; placeholders need replacing. |
| SSH | Remote sessions | https://man.openbsd.org/ssh | Escape commands require a preceding newline. |
| curl | HTTP requests | https://curl.se/docs/manpage.html | `--json` needs curl 7.82+; POST can modify server state. |
| tmux | Sessions and panes | https://github.com/tmux/tmux/blob/master/tmux.1 | Keys assume the default Ctrl+B prefix. |
| Vim | Modal editing | https://github.com/vim/vim/blob/master/runtime/doc/quickref.txt | Keys assume Normal mode. Basic entries also work in Neovim. |
| fzf | Interactive selection | https://github.com/junegunn/fzf#usage | Shell bindings require integration; multi-selection requires multi mode. |
| ripgrep | Search file contents | https://github.com/BurntSushi/ripgrep/blob/master/GUIDE.md | Ignores hidden/ignored files unless explicitly requested. |
| fd | Find file paths | https://github.com/sharkdp/fd#usage | Ubuntu/Debian may expose `fdfind` instead of `fd`. |
| jq | Inspect JSON | https://jqlang.org/manual/ | Examples assume POSIX shell quoting. |
| htop | Inspect processes | https://github.com/htop-dev/htop/blob/master/htop.1.in | Signal actions can terminate processes. |

Git, SSH and curl anchor the general workflow; tmux and Vim cover interactive keyboard use; fzf, ripgrep, fd and jq cover searching and structured data; htop covers process inspection. This choice is intentionally developer-oriented, rather than claiming that newer tools are used more often than traditional `grep`, `find`, `sed`, `awk`, or `less`.

## Ghostty corrections

The installed Ghostty's `ghostty +list-keybinds --default` output was checked. The reference now uses Ctrl+Shift+O for a left/right split, Ctrl+Shift+E for a top/bottom split, Ctrl+Enter for fullscreen, Ctrl+, for configuration, and Ctrl+Shift+, for reload. The earlier illustrative Ctrl+Shift+D and F11 entries were removed.

Explicit `keybind` lines in the Ghostty config replace the default reference in the panel. This remains a lightweight parser: includes, reset directives, conditional bindings and the complete effective keymap are not resolved. The Docs tooltip says so.

## Curated risk review

All 94 rows in `catalog.json` and all 15 bundled Ghostty rows were inspected as part of the local catalog review. The snapshot fingerprints and curated high-impact warning coverage are asserted in Rust tests; changing either data file requires explicitly re-reviewing the full snapshot and updating its fingerprint. Exact warnings are attached to the reviewed Git, SSH, curl, Vim, fzf, ripgrep, fd, htop and Ghostty entries that can alter repository or file state, transmit remote data, expose private details, or close/paste terminal content. Placeholder highlighting has a separate generic reminder. This is a conservative, curated warning set, not a command parser or a safety guarantee; personal references may carry their own author-written warning.

## Product boundaries

- The app displays and copies references; it does not install or execute these tools.
- No terminal content, shell history or window titles are read to guess the active tool.
- Tool selection is manual and remembered between launches.
- Commands and keys have separate labels. Custom bindings may differ from documented defaults.
- Add another bundled reference by editing `catalog.json` and re-reviewing the complete catalog snapshot; end users can create/edit local personal entries in the Info screen.
