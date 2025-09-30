---
outline: deep
---

# Usage

Ducker is comprised of a set of **pages**, each of which display specific information about and/or allow interaction with the docker containers and images on the host system.

Top level **pages** can be navigated to with **commands**, input via the **prompt**.  **Pages** can be interacted with using **actions**; these are input via hotkey inputs.

A legend for common global hotkey inputs is displayed at the bottom of the screen; one for contextual (eg different on each page) hotkey inputs are displayed in the top right.

## Commands

The following commands are supported:

| Command      | Aliases     | Description                          |
| ------------ | ----------- | ------------------------------------ |
| `images`     | `image`     | Open the `Images` top level page     |
| `containers` | `container` | Open the `Containers` top level page |
| `volumes`    | `volume`    | Open the `Volumes` top level page    |
| `networks`   | `network`   | Open the `Networks` top level page   |
| `help`       | `h`         | Open the `Help` page                 |
| `quit`       | `q`         | Close the application                |


## Actions

### Global

The following global actions are available on all pages:

| Hotkey  | Action                                    |
| ------- | ----------------------------------------- |
| `k`/`↑` | Navigate up in a list/table               |
| `j`/`↓` | Navigate down in a list/table             |
| `Q`/`q` | Close the application                     |
| `:`     | Open the command prompt                   |
| `G`     | Navigate to the bottom of a list or table |
| `g`     | Navigate to the top of a list or table    |

### Containers

The following actions are available on the Containers page:

| Hotkey   | Action                                                                |
| -------- | --------------------------------------------------------------------- |
| `Ctrl+d` | Delete the currently selected container                               |
| `a`      | Exec into the currently selected container (if container is running)* |
| `l`      | View the logs for the currently selected container                    |
| `r`      | Run the currently selected container                                  |
| `s`      | Stop the currently selected container                                 |

***NB**: exec currently only supports containers with bash installed.  The intention is that this will be updated to provide a user option.

### Images

The following actions are available on the Images page:

| Hotkey   | Action                                |
| -------- | ------------------------------------- |
| `Ctrl+d` | Delete the currently selected image   |
| `Alt+d`  | Toggle dangling images                |
| `d`      | Describe the currently selected image |

### Volumes

The following actions are available on the Volumes page:

| Hotkey   | Action                                 |
| -------- | -------------------------------------- |
| `Ctrl+d` | Delete the currently selected volume   |
| `Alt+d`  | Toggle dangling volumes                |
| `d`      | Describe the currently selected volume |

### Networks

The following actions are available on the Networks page:

| Hotkey   | Action                                  |
| -------- | --------------------------------------- |
| `Ctrl+d` | Delete the currently selected network   |
| `d`      | Describe the currently selected network |

> :warning: **Network deletion isn't entirely complete**: A failed deletion currently results in a yes/no modal telling you that it couldn't be deleted.  There is no difference between the yes and no results.  This is due to the current modal story and a quick and dirty hack to get them set up.  Once a generic modal exists this will be patched up!

### Logs

The following actions are available on the Logs page:

| Hotkey | Action                        |
| ------ | ----------------------------- |
| `Esc`  | Return to the containers page |

### Sorting Hotkeys

> **Tip:** Use `Shift` + the indicated key to sort columns.
> Pressing the same sorting key again will sort the same column in the opposite order (toggle ascending/descending).
> See the table below for each page's sort options.

#### Containers
| Hotkey    | Action          |
| --------- | --------------- |
| `Shift+N` | Sort by name    |
| `Shift+I` | Sort by image   |
| `Shift+S` | Sort by status  |
| `Shift+C` | Sort by created |
| `Shift+P` | Sort by ports   |

#### Images
| Hotkey    | Action          |
| --------- | --------------- |
| `Shift+N` | Sort by name    |
| `Shift+C` | Sort by created |
| `Shift+T` | Sort by tag     |
| `Shift+S` | Sort by size    |

#### Networks
| Hotkey    | Action          |
| --------- | --------------- |
| `Shift+N` | Sort by name    |
| `Shift+C` | Sort by created |
| `Shift+S` | Sort by scope   |
| `Shift+D` | Sort by driver  |

#### Volumes
| Hotkey    | Action             |
| --------- | ------------------ |
| `Shift+N` | Sort by name       |
| `Shift+C` | Sort by created    |
| `Shift+D` | Sort by driver     |
| `Shift+M` | Sort by mountpoint |