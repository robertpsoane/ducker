---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
  tagline: "A slightly quackers Docker TUI based on k9s 🦆"
  image: ./assets/ducker-logo.png
  actions:
    - theme: brand
      text: Getting Started
      link: /docs/020-using-ducker

features:
  - title: Docker & Podman
    details: Supports Docker & Podman (and probably other OCI-compliant container runtimes!)
  - title: Rust
    details: 🦀 It's written in Rust so it must be good! 🦀
  - title: K9s
    details: Inspired by the K9s TUI for managing Kubernetes clusters
    link: https://k9scli.io/
footer: Foo
---

<img class="demo-gif" src="./assets/demo.gif" />

## Installation

### Cargo

There isn't currently a downloadable pre-built binary; to install you will need cargo installed:

```bash
cargo install --locked ducker
```
> :warning: **Make sure you use --locked**: if ducker is installed without `--locked` it is susceptible to changes in upstream dependencies, which could break the build.

### Arch Linux

You can install `ducker` from the [official repositories](https://archlinux.org/packages/extra/x86_64/ducker/) with using [pacman](https://wiki.archlinux.org/title/pacman).

```sh
pacman -S ducker
```

### Homebrew

If you have homebrew:

```sh
brew install ducker
```

### Unstable

To install the latest unstable version of Ducker, run the following command:

```bash
cargo install --git https://github.com/robertpsoane/ducker
```


