---
outline: deep
---

# Installation

## Cargo

There isn't currently a downloadable pre-built binary; to install you will need cargo installed:

```bash
cargo install --locked ducker
```
> :warning: **Make sure you use --locked**: if ducker is installed without `--locked` it is susceptible to changes in upstream dependencies, which could break the build.

## Arch Linux

You can install `ducker` from the [official repositories](https://archlinux.org/packages/extra/x86_64/ducker/) with using [pacman](https://wiki.archlinux.org/title/pacman).

```sh
pacman -S ducker
```

## Homebrew

If you have homebrew:

```sh
brew install ducker
```

## Unstable

To install the latest unstable version of Ducker, run the following command:

```
cargo install --git https://github.com/robertpsoane/ducker
```


