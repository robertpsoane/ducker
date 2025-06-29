
<p align="center">
<a href="https://github.com/robertpsoane/ducker"><img src="https://raw.githubusercontent.com/robertpsoane/ducker/master/assets/ducker.png?raw=true" width="250" height="250" /></a>
<h1 align="center">Ducker</h1>
</p>

<p align="center">
🐋 A terminal app for managing docker containers, inspired by <a href="https://k9scli.io">K9s</a>
<br/>
<a href="#installation">Installation</a> · <a href="#usage">Usage</a> · <a href="#configuration">Configuration</a>
</p>


<a href="https://github.com/robertpsoane/ducker"><img src="https://raw.githubusercontent.com/robertpsoane/ducker/master/assets/demo.gif?raw=true" width="100%" /></a>

<p align="center">
🦀 Written in Rust 🦀
</p>

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

### Brew

For macOS, you can install `ducker` using by `homebrew`.

```sh
brew install ducker
```

### Unstable

To install the latest unstable version of Ducker, run the following command:

```
cargo install --git https://github.com/robertpsoane/ducker
```

## Usage

Ducker is comprised of a set of **pages**, each of which display specific information about and/or allow interaction with the docker containers and images on the host system.

Top level **pages** can be navigated to with **commands**, input via the **prompt**.  **Pages** can be interacted with using **actions**; these are input via hotkey inputs.

A legend for common global hotkey inputs is displayed at the bottom of the screen; one for contextual (eg different on each page) hotkey inputs are displayed in the top right.

### Commands

The following commands are supported:

| Command      | Aliases     | Description                          |
| ------------ | ----------- | ------------------------------------ |
| `images`     | `image`     | Open the `Images` top level page     |
| `containers` | `container` | Open the `Containers` top level page |
| `volumes`    | `volume`    | Open the `Volumes` top level page    |
| `networks`   | `network`   | Open the `Networks` top level page   |
| `quit`       | `q`         | Close the application                |


### Actions

#### Global

The following global actions are available on all pages:

| Hotkey  | Action                                    |
| ------- | ----------------------------------------- |
| `k`/`↑` | Navigate up in a list/table               |
| `j`/`↓` | Navigate down in a list/table             |
| `Q`/`q` | Close the application                     |
| `:`     | Open the command prompt                   |
| `G`     | Navigate to the bottom of a list or table |
| `g`     | Navigate to the top of a list or table    |

#### Containers

The following actions are available on the Containers page:

| Hotkey   | Action                                                                |
| -------- | --------------------------------------------------------------------- |
| `Ctrl+d` | Delete the currently selected container                               |
| `a`      | Exec into the currently selected container (if container is running)* |
| `l`      | View the logs for the currently selected container                    |
| `r`      | Run the currently selected container                                  |
| `s`      | Stop the currently selected container                                 |

***NB**: exec currently only supports containers with bash installed.  The intention is that this will be updated to provide a user option.

#### Images

The following actions are available on the Images page:

| Hotkey   | Action                                                         |
| -------- | -------------------------------------------------------------- |
| `Ctrl+d` | Delete the currently selected image                            |
| `d`      | Describe the currently selected image                          |
| `D`      | Toggle whether or not to show dangling images (off by default) |

#### Volumes

The following actions are available on the Volumes page:

| Hotkey   | Action                                 |
| -------- | -------------------------------------- |
| `Ctrl+d` | Delete the currently selected volume   |
| `d`      | Describe the currently selected volume |

#### Networks

The following actions are available on the Networks page:

| Hotkey   | Action                                  |
| -------- | --------------------------------------- |
| `Ctrl+d` | Delete the currently selected network   |
| `d`      | Describe the currently selected network |

> :warning: **Network deletion isn't entirely complete**: A failed deletion currently results in a yes/no modal telling you that it couldn't be deleted.  There is no difference between the yes and no results.  This is due to the current modal story and a quick and dirty hack to get them set up.  Once a generic modal exists this will be patched up!

#### Logs

The following actions are available on the Logs page:

| Hotkey | Action                        |
| ------ | ----------------------------- |
| `Esc`  | Return to the containers page |

## Configuration

Ducker is configured via a yaml file found in the relevant config directory for host platform.  On linux this is `~/.config/ducker/config.yaml`.

The following table summarises the available config values:

| Key                         | Default                       | Description                                                                                                                   |
| --------------------------- | ----------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| prompt                      | 🦆                             | The default prompt to display in the command pane                                                                             |
| default_exec                | `/bin/bash`                   | The default prompt to display in the command pane. NB - currently uses this for all exec's; it is planned to offer a choice   |
| docker_path                 | `unix:///var/run/docker.sock` | The location of the socket on which the docker daemon is exposed (defaults to `npipe:////./pipe/docker_engine` on windows)    |
| check_for_update            | `true`                        | When true, checks whether there is a newer version on load.  If a newer version is found, indicates via note in bottom right. |
| autocomplete_minimum_length | 2                             | The default minimum length before autocompletion in prompt.                                                                   |
| theme                       | [See below]                   | The colour theme configuration                                                                                                |

If a value is unset or if the config file is unfound, Ducker will use the default values.  If a value is malformed, Ducker will fail to run.

To create a fully populated default config, run ducker with the `-e/--export-default-config` flag; this will write the default config to the default location, overwriting any existing config.

### Themes

By default, ducker uses the terminal emulator's preset colours.  However, it is possible to set a custom colour theme in config.  This is set in the `theme` section of the config file.  The following table describes the theme options.  The default theme provides the colours provided in the GIF in this README.

| Key                | Default   | Description                                                                                          |
| ------------------ | --------- | ---------------------------------------------------------------------------------------------------- |
| use_theme          | `false`   | When `true` uses the colour scheme defined in config, when `false` uses the default terminal colours |
| title              | `#96E072` | The colour used for the Ducker font in the header                                                    |
| help               | `#EE5D43` | The colour used in the help prompts in the header                                                    |
| background         | `#23262E` | The colour used in the background                                                                    |
| footer             | `#00E8C6` | The colour used for the text in the footer                                                           |
| success            | `#96E072` | The colour used for a successful result                                                              |
| error              | `#EE5D43` | The colour used for an error result                                                                  |
| positive_highlight | `#96E072` | The colour used for highlighting in a happy state                                                    |
| negative_highlight | `#FF00AA` | The colour used for highlighting in a sad state                                                      |

### Tmux

Some characters in ducker use italics/boldface.  This doesn't work by default when running in tmux.  To fix this, add the following to your add to tmux.conf
```
set -g default-terminal "tmux-256color"
set -as terminal-overrides ',xterm*:sitm=\E[3m'
```
