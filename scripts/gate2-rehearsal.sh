#!/bin/sh
# Gate 2's rehearsal on one machine (docs/05 §Gate 2's rehearsal): builds the process test, then
# runs it in a user, network and mount namespace of its own, where the test makes a namespace per
# node and injects the faults. Linux with unprivileged user namespaces; no root.
set -eu
cd "$(dirname "$0")/.."
cargo test -p trial --test gate2_rehearsal --no-run
exec unshare --user --map-root-user --net --mount sh -c '
    set -eu
    mount -t tmpfs tmpfs /run
    ISEGORIA_REHEARSAL=1 exec cargo test --offline -p trial --test gate2_rehearsal -- \
        --ignored --nocapture "$@"' sh "$@"
