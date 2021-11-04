# COSMIC

Computer Operating System Main Interface Components

COSMIC is the name for the main shell components in Pop_Shell (separate from the tiling and window-management components). It makes the following changes to the standard GNOME Shell environment:

* [Dock](https://github.com/pop-os/cosmic-dock)
* [Workspaces](https://github.com/pop-os/cosmic-workspaces)
* [Configuration options](https://github.com/pop-os/desktop-widget)
* Separated Workspaces overview from Applications.

Most components can be configured to fit the user's workflow and preferences, with two main presets for both keyboard-focused and mouse-focused navigation and use.

## Requirements

COSMIC requires the following components:

* [Pop Theme](https://github.com/pop-os/gtk-theme) >= 5.4.0
* [GNOME Shell](https://gitlab.gnome.org/GNOME/gnome-shell) == 3.38.*
* [Pop_Shell](https://github.com/pop-os/shell) >= 1.1.0


## Installation

The recommended way to install COSMIC is through the package manager on Pop!_OS. To install COSMIC on Pop!_OS 21.04 and higher:

```
sudo apt update
sudo apt install pop-cosmic libpop-desktop-widget gnome-shell-extension-cosmic-dock gnome-shell-extension-cosmic-workspaces
```

Next restart GNOME Shell using Alt+F2, type `r`, and press Enter. Then enable the "Cosmic Dock", "Cosmic Workspaces", and "Pop COSMIC" extensions in GNOME Extensions or GNOME Tweaks. You will also need to enable "Pop Shell" if it is not enabled.

### Installation from Source

Installation from source code is possible for testing changes, but is not recommended for general use. 

The following COSMIC components need to be installed separately:

* [COSMIC Desktop Widget](https://github.com/pop-os/desktop-widget)
* [COSMIC Dock](https://github.com/pop-os/gnome-shell-extension-ubuntu-dock)
* [COSMIC Workspaces](https://github.com/pop-os/cosmic-workspaces)

Following that, install COSMIC from source:

```
git clone https://github.com/pop-os/cosmic
cd cosmic
make && make install
```

##### Note
Use of `sudo` is not required nor recommended for COSMIC.

## Removal

To remove COSMIC, remove each component listed above, then:

```
rm -r ~/.local/share/gnome-shell/extensions/pop-cosmic@system76.com
```

## License
COSMIC is available under the terms of the GNU General Public License Version 3. For full license terms, see [LICENSE](./LICENSE).
