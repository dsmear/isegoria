#!/bin/sh
# Gate 1 of Milestone 2 on one machine (docs/05 §Gate 1 on one machine): builds the node, the
# client and the test issuer, runs the gate's process test and prints its evidence directory.
set -eu
cd "$(dirname "$0")/.."
exec cargo test -p trial --test gate1 -- --nocapture "$@"
