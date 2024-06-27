# Ducker

A terminal app for managing docker containers, inpired by [K9s](https://k9scli.io/)

This is perhaps obviously very much a work in progress...


![](https://raw.githubusercontent.com/robertpsoane/ducker/master/demo.gif?raw=true)


## Installation

There isn't currently a downloadable build; to install you will need cargo installed:

```bash
cargo install --locked ducker
```

### AUR

You can install `ducker` from the [AUR](https://aur.archlinux.org/packages/ducker) with using an [AUR helper](https://wiki.archlinux.org/title/AUR_helpers).

```sh
paru -S ducker
```

## Tmux

To fix in tmux:
add to tmux.conf
```
set -g default-terminal "tmux-256color"
set -as terminal-overrides ',xterm*:sitm=\E[3m'
```
