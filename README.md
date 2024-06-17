# Ducker

A terminal app for managing docker containers, inpired by [K9s](https://k9scli.io/)

This is a work in progress; main aim is to get more to grips with rust.

## Build/Run

This is built with cargo (`cargo build`)

To fix in tmux:
add to tmux.conf
```
set -g default-terminal "tmux-256color"
set -as terminal-overrides ',xterm*:sitm=\E[3m'
```