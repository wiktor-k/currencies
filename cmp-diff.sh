#!/usr/bin/bash

set -Eeuxo pipefail

~/bin/diffoci pull ghcr.io/wiktor-k/currencies:latest
~/bin/diffoci pull wiktork/currencies:latest
~/bin/diffoci diff ghcr.io/wiktor-k/currencies:latest wiktork/currencies:latest
docker pull ghcr.io/wiktor-k/currencies:latest
docker pull wiktork/currencies:latest
docker save ghcr.io/wiktor-k/currencies > /tmp/ghcr.tar
docker save wiktork/currencies > /tmp/dh.tar
