#!/usr/bin/env bash

set -ex

DPKG_VERSION="$(dpkg-parsechangelog --show-field Version)"
fakeroot debian/rules binary
sudo dpkg -i ../pop-cosmic_"${DPKG_VERSION}"_amd64.deb
killall pop-cosmic
journalctl -f -t pop-cosmic.desktop
