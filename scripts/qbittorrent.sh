#! /usr/bin/env bash

# qbittorrent-nox.sh start|stop DIR PORT
#   DIR will contain the qbittorrent-nox profile, as well as "pid" and "qBottorrent.log"
#   PORT configures port for qbittorrent-nox web API

set -eu

# finds a free TCP port
# https://stackoverflow.com/questions/28989069/how-to-find-a-free-tcp-port/45539101#45539101
function findFreePort() {
    BASE_PORT=16998
    INCREMENT=1

    port=$BASE_PORT
    isfree=$(netstat -taln | grep $port)

    while [[ -n "$isfree" ]]; do
        port=$[port+INCREMENT]
        isfree=$(netstat -taln | grep $port)
    done

    echo "$port"
}

function configureqBittorrent() {
    QBTNOX_DIR="$1"
    QBTNOX_PORT="$2"
    # cat "$(dirname "$0")"/qbittorrent.toml | envsubst > "$QBTNOX_DIR"/config.toml
    mkdir -p "$QBTNOX_DIR"/qBittorrent/config/
    echo "$QBTNOX_PORT" > /tmp/qbittorrent.port

    cat <<EOF > "$QBTNOX_DIR"/qBittorrent/config/qBittorrent.conf
[BitTorrent]
Session\Port=23405
Session\QueueingSystemEnabled=false
Session\SSL\Port=43419
Session\Interface=lo
Session\InterfaceName=lo

[Meta]
MigrationVersion=8

[Network]
Cookies=@Invalid()

[Preferences]
WebUI\Password_PBKDF2="@ByteArray(ARQ77eY1NUZaQsuDHbIMCA==:0WMRkYTUWVT9wVvdDtHAjU9b3b7uB8NR1Gur2hmQCvCDpm39Q+PsJRJPaCU51dEiz+dTzh8qbPsL8WkFljQYFQ==)"
EOF

}

# Wait for qBittorrent WEB API to come online at address $1
# waitforqBittorrentStart "http://localhost:1312"
function waitforqBittorrentStart() {
    #set +x
    TIMESTAMP=$(date +%s)
    END=$((TIMESTAMP+10))
    ERR=0
    while true; do
        NEWTIMESTAMP=$(date +%s)
        if [ $NEWTIMESTAMP -gt $END ]; then
            ERR=1
            break
        fi
        if curl --silent "$1" 2>&1 > /dev/null; then
            break
        else
            sleep 0.1
        fi
    done
    return $ERR
}

# Wait for qBittorrent to be done cleanly exiting
# Necessary because otherwise it will leave temporary files behind!
# waitforQbittorrentStop 1234
function waitforqBittorrentStop() {
    TIMESTAMP=$(date +%s)
    END=$((TIMESTAMP+15))
    ERR=0
    while true; do
        NEWTIMESTAMP=$(date +%s)
        if [ $NEWTIMESTAMP -gt $END ]; then
            ERR=1
            break
        fi
        if ! ps x | grep -P "^\s+$PID\s+" 2>&1 > /dev/null; then
            # process died successfully
            break
        else
            sleep 0.1
        fi
    done
    return $ERR
}

start() {
    echo "y" | qbittorrent-nox --profile="$1" --webui-port="$2" 2>&1 > "$1"/qBittorrent.log &
    echo $! > "$1"/pid
    echo "$2" > "$1"/port
    if waitforqBittorrentStart "http://localhost:$2"; then
        return 0
    else
        return 1
    fi
}

stop() {
    PID="$(cat "$1"/pid)"
    kill $PID
    if ! waitforqBittorrentStop $PID; then
        echo "qBittorrent does not quit. Using SIGKILL"
        kill -9 $PID
    fi
}

case "$1" in
    "start")
        QBTNOX_DIR="$2"
        if [ -z ${3+x} ]; then
            QBTNOX_PORT="$(findFreePort)"
        else
            QBTNOX_PORT=$3
        fi

        configureqBittorrent "$QBTNOX_DIR" "$QBTNOX_PORT"
        
        start "$QBTNOX_DIR" "$QBTNOX_PORT"
        STATUS=$?
        ;;
    "stop")
        QBTNOX_DIR="$2"
        stop "$QBTNOX_DIR"
        STATUS=$?
        ;;
esac

exit $STATUS
