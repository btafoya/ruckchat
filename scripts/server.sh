#!/usr/bin/env bash
set -euo pipefail

# Start, stop, and inspect the RuckChat server Docker Compose stack.
#
# Usage: ./scripts/server.sh {start|stop|restart|status|logs} [options]
#
# Commands:
#   start    Start the stack (always recreates containers)
#   stop     Stop the stack (default: down; use --keep to stop only)
#   restart  Stop then start the stack
#   status   Show running containers
#   logs     Follow container logs
#
# Options:
#   -b, --build           Use docker-compose.build.yml (source build)
#   -f, --file PATH       Use a custom compose file (overrides --build)
#   -c, --config PATH     Path to ruckchat.yaml config file (default: ./ruckchat.yaml)
#   -k, --keep            On stop, stop containers instead of removing them
#   -h, --help            Show this help message
#
# Examples:
#   ./scripts/server.sh start
#   ./scripts/server.sh start --build
#   ./scripts/server.sh start --config /etc/ruckchat/ruckchat.yaml
#   ./scripts/server.sh stop --keep

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

DEFAULT_COMPOSE_FILE="${PROJECT_ROOT}/docker-compose.yml"
BUILD_COMPOSE_FILE="${PROJECT_ROOT}/docker-compose.build.yml"
DEFAULT_CONFIG_FILE="${PROJECT_ROOT}/ruckchat.yaml"

COMPOSE_FILE="${DEFAULT_COMPOSE_FILE}"
CONFIG_FILE="${DEFAULT_CONFIG_FILE}"
USE_BUILD=0
KEEP=0
COMMAND=""

usage() {
    cat <<EOF
Usage: $0 {start|stop|restart|status|logs} [options]

Commands:
  start    Start the RuckChat server stack (always recreates containers)
  stop     Stop the RuckChat server stack (default: down; use --keep to stop only)
  restart  Stop then start the stack
  status   Show running containers
  logs     Follow container logs

Options:
  -b, --build           Use docker-compose.build.yml (source build)
  -f, --file PATH       Use a custom compose file (overrides --build)
  -c, --config PATH     Path to ruckchat.yaml config file (default: ./ruckchat.yaml)
  -k, --keep            On stop, stop containers instead of removing them
  -h, --help            Show this help message

Examples:
  $0 start
  $0 start --build
  $0 start --config /etc/ruckchat/ruckchat.yaml
  $0 stop --keep
EOF
}

resolve_path() {
    local path="$1"
    if [[ "${path}" = /* ]]; then
        printf '%s\n' "${path}"
    else
        printf '%s\n' "$(pwd)/${path}"
    fi
}

require_command() {
    local name="$1"
    if ! command -v "${name}" >/dev/null 2>&1; then
        printf 'Error: "%s" is not installed or not in PATH.\n' "${name}" >&2
        exit 1
    fi
}

require_docker_compose() {
    require_command docker
    if ! docker compose version >/dev/null 2>&1; then
        printf 'Error: "docker compose" plugin is not available.\n' >&2
        exit 1
    fi
}

validate_compose() {
    require_docker_compose

    if [[ ! -f "${COMPOSE_FILE}" ]]; then
        printf 'Error: compose file not found: %s\n' "${COMPOSE_FILE}" >&2
        exit 1
    fi
}

validate_config() {
    if [[ ! -f "${CONFIG_FILE}" ]]; then
        printf 'Error: config file not found: %s\n' "${CONFIG_FILE}" >&2
        exit 1
    fi

    if [[ ! -r "${CONFIG_FILE}" ]]; then
        printf 'Error: config file is not readable: %s\n' "${CONFIG_FILE}" >&2
        exit 1
    fi
}

config_port() {
    awk -F':' '/^base_url:/ {gsub(/[^0-9]/,"",$NF); print $NF; exit}' "${CONFIG_FILE}"
}

compose_target_port() {
    local port=""
    if [[ -f "${COMPOSE_FILE}" ]]; then
        # Short form "HOST:CONTAINER" under a ports: list.
        port=$(awk '
            /^  server:/ { in_server=1 }
            in_server && /^  [^ ]/ && !/^  server:/ { exit }
            in_server && /^    ports:/ { in_ports=1 }
            in_ports && /^    [^ ]/ && !/^    ports:/ { exit }
            in_ports && /- "[0-9]+:[0-9]+"/ {
                gsub(/"/,"");
                split($0, parts, ":");
                gsub(/[^0-9]/,"",parts[length(parts)]);
                print parts[length(parts)];
                exit;
            }
        ' "${COMPOSE_FILE}")
    fi
    printf '%s\n' "${port}"
}

warn_port_alignment() {
    local cfg_port target_port
    cfg_port="$(config_port)"
    : "${cfg_port:=3000}"
    target_port="$(compose_target_port)"

    if [[ -n "${target_port}" && "${target_port}" != "${cfg_port}" ]]; then
        printf 'Warning: ruckchat.yaml base_url port is %s, but %s exposes container port %s.\n' "${cfg_port}" "${COMPOSE_FILE}" "${target_port}" >&2
        printf '         External traffic will not reach the server. Change the right side of the ports mapping to %s.\n' "${cfg_port}" >&2
    fi
}

: "${RUCKCHAT_IMAGE:=ruckchat-server}"
export RUCKCHAT_IMAGE

compose() {
    cd "${PROJECT_ROOT}"
    docker compose -f "${COMPOSE_FILE}" "$@"
}

do_start() {
    printf 'Starting RuckChat server...\n'
    printf '  compose file: %s\n' "${COMPOSE_FILE}"
    printf '  config file:  %s\n' "${CONFIG_FILE}"
    warn_port_alignment
    compose up -d --force-recreate
}

do_stop() {
    if [[ "${KEEP}" -eq 1 ]]; then
        printf 'Stopping RuckChat server containers (keeping state)...\n'
        compose stop
    else
        printf 'Stopping and removing RuckChat server stack...\n'
        compose down
    fi
}

do_restart() {
    do_stop
    do_start
}

do_status() {
    compose ps
}

do_logs() {
    compose logs -f
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        start|stop|restart|status|logs)
            if [[ -n "${COMMAND}" ]]; then
                printf 'Error: multiple commands specified (%s and %s).\n' "${COMMAND}" "$1" >&2
                exit 1
            fi
            COMMAND="$1"
            shift
            ;;
        -b|--build)
            USE_BUILD=1
            shift
            ;;
        -f|--file)
            if [[ -z "${2:-}" ]]; then
                printf 'Error: --file requires a path.\n' >&2
                exit 1
            fi
            COMPOSE_FILE="$(resolve_path "$2")"
            USE_BUILD=0
            shift 2
            ;;
        -c|--config)
            if [[ -z "${2:-}" ]]; then
                printf 'Error: --config requires a path.\n' >&2
                exit 1
            fi
            CONFIG_FILE="$(resolve_path "$2")"
            shift 2
            ;;
        -k|--keep)
            KEEP=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            printf 'Error: unknown option "%s"\n' "$1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

if [[ "${USE_BUILD}" -eq 1 ]]; then
    COMPOSE_FILE="${BUILD_COMPOSE_FILE}"
fi

if [[ -z "${COMMAND}" ]]; then
    usage >&2
    exit 1
fi

export RUCKCHAT_CONFIG="${CONFIG_FILE}"

validate_compose

case "${COMMAND}" in
    start|restart)
        validate_config
        ;;
esac

case "${COMMAND}" in
    start)
        do_start
        ;;
    stop)
        do_stop
        ;;
    restart)
        do_restart
        ;;
    status)
        do_status
        ;;
    logs)
        do_logs
        ;;
    *)
        printf 'Error: unknown command "%s"\n' "${COMMAND}" >&2
        usage >&2
        exit 1
        ;;
esac
