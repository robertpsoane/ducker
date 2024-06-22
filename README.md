# Ducker

A terminal app for managing docker containers, inpired by [K9s](https://k9scli.io/)

This is perhaps obviously very much a work in progress...

## Installation

There isn't currently a downloadable build; to install you will need rustc & cargo installed.

Clone the repo, build with cargo (`cargo build -r`) and put the resultant binary on your path.

For convenience, the `install.sh` script does just this:

```bash
curl -sS https://raw.githubusercontent.com/robertpsoane/ducker/master/install.sh | sh
```


## Tmux

To fix in tmux:
add to tmux.conf
```
set -g default-terminal "tmux-256color"
set -as terminal-overrides ',xterm*:sitm=\E[3m'
```
