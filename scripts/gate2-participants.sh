#!/bin/sh
# Gate 2's scripted, synthetic participants (docs/23, stages 4 and 5): fourteen
# persons' credentials, events and reads through isegoria-client, one step per call. The persons'
# files stay under $DIR; only issuance requests and replies cross $EXCHANGE.
#
# Reads ./participants.env or $PARTICIPANTS_ENV, a shell file setting:
#   CLIENT      path of isegoria-client
#   GENESIS     path of the genesis document
#   RELAY       the relay's address, ending in /p2p/<peer id>
#   ANNOUNCE_0 .. ANNOUNCE_3   each member's --announce line
#   DIR         the persons' directory (secrets; never shared)
#   EXCHANGE    the directory shared with the issuer's holder
set -eu

. "${PARTICIPANTS_ENV:-./participants.env}"
PERSONS=14
TRIES="${TRIES:-60}"

announce() { eval "printf '%s' \"\$ANNOUNCE_$1\""; }

# send <person> <event> <member>: repeated while the relay or the gateway cannot take it.
send() {
    n=0
    until "$CLIENT" --send "$GENESIS" "$(announce "$3")" "$RELAY" "$DIR/p$1/$2" \
        >"$DIR/p$1/$2.receipt" 2>"$DIR/p$1/$2.error"; do
        n=$((n + 1))
        [ "$n" -lt "$TRIES" ] || { echo "p$1 $2 to member $3: no receipt" >&2; exit 1; }
        sleep 2
    done
    echo "p$1 $2 member $3: $(cat "$DIR/p$1/$2.receipt")"
}

# read_as <person> <member> <ask>...: one read, its output on standard output.
read_as() {
    k=$1 m=$2
    shift 2
    "$CLIENT" --read "$GENESIS" "$DIR/p$k/credential" "$(announce "$m")" "$RELAY" "$@"
}

# settled <person> <event> <member>: waits until no entry of the event is pending.
settled() {
    n=0
    while :; do
        if out=$(read_as "$1" "$3" status "$DIR/p$1/$2" 2>/dev/null); then
            case $out in
            "entries 0"* | *pending*) ;;
            *) break ;;
            esac
        fi
        n=$((n + 1))
        [ "$n" -lt "$TRIES" ] || { echo "p$1 $2: still pending at member $3" >&2; exit 1; }
        sleep 2
    done
    echo "p$1 $2: $(echo "$out" | tr '\n' ';')"
}

# Members to use in turn, from the command's remaining arguments (default 0 1 2 3).
turn() {
    i=$1
    shift
    [ "$#" -gt 0 ] || set -- 0 1 2 3
    shift $((i % $#))
    echo "$1"
}

step=${1:?usage: gate2-participants.sh requests|finalize|join|deposit|drawn|commit|reveal|author|pending ...}
shift
case $step in
requests)
    mkdir -p "$EXCHANGE/requests"
    k=0
    while [ "$k" -lt "$PERSONS" ]; do
        mkdir -p "$DIR/p$k"
        "$CLIENT" --request "$GENESIS" "$k" "$DIR/p$k/pending" "$EXCHANGE/requests/slot-$k.request"
        k=$((k + 1))
    done
    ;;
finalize)
    k=0
    while [ "$k" -lt "$PERSONS" ]; do
        "$CLIENT" --finalize "$GENESIS" "$DIR/p$k/pending" "$EXCHANGE/replies/slot-$k.reply" \
            "$DIR/p$k/credential"
        k=$((k + 1))
    done
    echo "credentials $PERSONS"
    ;;
join)
    k=1
    while [ "$k" -lt "$PERSONS" ]; do
        "$CLIENT" --join "$GENESIS" "$DIR/p$k/credential" "$DIR/p$k/join"
        send "$k" join "$(turn "$k" "$@")"
        k=$((k + 1))
    done
    send 1 join "$(turn 2 "$@")"
    k=1
    while [ "$k" -lt "$PERSONS" ]; do
        settled "$k" join "$(turn $((k + 1)) "$@")"
        k=$((k + 1))
    done
    ;;
deposit)
    epoch=${1:?deposit <epoch> [member]}
    cat >"$DIR/p0/question.toml" <<'EOF'
context = "gate 2, a synthetic question"
stem = "Which of these numbers is even?"
alternatives = ["two, synthetic", "three, synthetic", "five, synthetic"]
key = 0
EOF
    "$CLIENT" --deposit "$GENESIS" "$DIR/p0/credential" "$epoch" "$DIR/p0/question.toml" \
        "$DIR/p0/deposit"
    send 0 deposit "${2:-0}"
    settled 0 deposit "${2:-0}"
    ;;
drawn)
    question=${1:?drawn <question> [members...]}
    shift
    : >"$DIR/drawn"
    k=1
    while [ "$k" -lt "$PERSONS" ]; do
        out=$(read_as "$k" "$(turn "$k" "$@")" reviewer "$question")
        case $out in
        *"question $question"*) echo "$k" >>"$DIR/drawn" ;;
        esac
        k=$((k + 1))
    done
    echo "drawn $(wc -l <"$DIR/drawn" | tr -d ' ')"
    ;;
commit)
    question=${1:?commit <question> [members...]}
    shift
    i=0
    for k in $(cat "$DIR/drawn"); do
        "$CLIENT" --commit "$GENESIS" "$DIR/p$k/credential" "$question" 0 \
            "$DIR/p$k/commitment" "$DIR/p$k/commit"
        send "$k" commit "$(turn "$i" "$@")"
        i=$((i + 1))
    done
    i=0
    for k in $(cat "$DIR/drawn"); do
        settled "$k" commit "$(turn $((i + 1)) "$@")"
        i=$((i + 1))
    done
    ;;
reveal)
    # reveal first [member]: only the first drawn reviewer's, sent and not waited for.
    if [ "${1:-}" = first ]; then
        k=$(head -n 1 "$DIR/drawn")
        "$CLIENT" --reveal "$GENESIS" "$DIR/p$k/credential" "$DIR/p$k/commitment" "$DIR/p$k/reveal"
        send "$k" reveal "${2:-0}"
        exit 0
    fi
    i=0
    for k in $(cat "$DIR/drawn"); do
        if [ ! -f "$DIR/p$k/reveal" ]; then
            "$CLIENT" --reveal "$GENESIS" "$DIR/p$k/credential" "$DIR/p$k/commitment" \
                "$DIR/p$k/reveal"
            send "$k" reveal "$(turn "$i" "$@")"
        fi
        i=$((i + 1))
    done
    i=0
    for k in $(cat "$DIR/drawn"); do
        settled "$k" reveal "$(turn $((i + 1)) "$@")"
        i=$((i + 1))
    done
    ;;
pending)
    # pending <member>: the first drawn reviewer's reveal as that member reads it.
    k=$(head -n 1 "$DIR/drawn")
    read_as "$k" "${1:-0}" status "$DIR/p$k/reveal"
    ;;
author)
    read_as 0 "${1:-0}" author
    ;;
*)
    echo "unknown step $step" >&2
    exit 2
    ;;
esac
