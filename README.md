# Ducker

A terminal app for managing docker containers, inpired by [K9s](https://k9scli.io/)

This is perhaps obviously very much a work in progress...


![](https://raw.githubusercontent.com/robertpsoane/ducker/master/demo.gif?raw=true)


## Installation

There isn't currently a downloadable build; to install you will need cargo installed:

```
cargo install --git https://github.com/robertpsoane/ducker
```

There is also a convenience script `install.sh` that in essence does just this, however this script clones the code to ~/.ducker to benefit from incremental build:

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
