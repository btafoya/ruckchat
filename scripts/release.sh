#!/usr/bin/env bash
set -euo pipefail

# Release automation script for RuckChat.
#
# Creates and pushes a vx.x.x (or vx.x.x-prerelease) release tag.
#
# Usage:
#   ./scripts/release.sh [OPTIONS] <version>
#
# Options:
#   --dry-run     Print planned actions without executing them.
#   --no-confirm  Skip the interactive confirmation prompt.
#   --no-checks   Skip fmt/clippy/test and local builds.
#   --no-bump     Skip modifying version files and CHANGELOG.md.
#   --no-build    Skip local builds but still run checks.
#   -h, --help    Show this help message.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

DRY_RUN=0
NO_CONFIRM=0
NO_CHECKS=0
NO_BUMP=0
NO_BUILD=0

VERSION=""

AUTHOR_NAME="Brian Tafoya"
AUTHOR_EMAIL="btafoya@briantafoya.com"

usage() {
    cat <<USAGE
Usage: $0 [OPTIONS] <version>

Release a new RuckChat version. The version must be in the form
vx.x.x or vx.x.x-<prerelease>, e.g. v0.2.0 or v1.0.0-rc.1.

Options:
  --dry-run     Print planned actions without executing them.
  --no-confirm  Skip the interactive confirmation prompt.
  --no-checks   Skip fmt/clippy/test and local builds.
  --no-bump     Skip modifying version files and CHANGELOG.md.
  --no-build    Skip local builds but still run checks.
  -h, --help    Show this help message.

Examples:
  $0 --dry-run v0.2.0
  $0 v0.2.0
  $0 --no-confirm v1.0.0-rc.1
USAGE
}

log() {
    echo "[release] $*"
}

run() {
    if [[ "${DRY_RUN}" == "1" ]]; then
        echo "[dry-run] $*"
    else
        log "$*"
        "$@"
    fi
}

run_in() {
    local dir="$1"
    shift
    if [[ "${DRY_RUN}" == "1" ]]; then
        echo "[dry-run] (cd ${dir} && $*)"
    else
        log "(cd ${dir} && $*)"
        (cd "${dir}" && "$@")
    fi
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --dry-run)
                DRY_RUN=1
                ;;
            --no-confirm)
                NO_CONFIRM=1
                ;;
            --no-checks)
                NO_CHECKS=1
                ;;
            --no-bump)
                NO_BUMP=1
                ;;
            --no-build)
                NO_BUILD=1
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            -*)
                echo "Unknown option: $1" >&2
                usage >&2
                exit 1
                ;;
            *)
                if [[ -n "${VERSION}" ]]; then
                    echo "Only one version argument is allowed." >&2
                    usage >&2
                    exit 1
                fi
                VERSION="$1"
                ;;
        esac
        shift
    done

    if [[ -z "${VERSION}" ]]; then
        echo "Missing version argument." >&2
        usage >&2
        exit 1
    fi

    if [[ ! "${VERSION}" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?$ ]]; then
        echo "Invalid version format: ${VERSION}" >&2
        echo "Expected vx.x.x or vx.x.x-<prerelease>, e.g. v0.2.0 or v1.0.0-rc.1" >&2
        exit 1
    fi
}

gpg_configured() {
    git config --get user.signingkey &>/dev/null || git config --global --get user.signingkey &>/dev/null
}

require_gpg() {
    if ! gpg_configured; then
        echo "GPG signing is required, but no git user.signingkey is configured." >&2
        exit 1
    fi
}

ensure_clean_tree() {
    if [[ -n "$(git status --porcelain)" ]]; then
        echo "Working tree is not clean. Commit or stash changes before releasing." >&2
        git status --short >&2
        exit 1
    fi
}

ensure_on_main() {
    local current_branch
    current_branch="$(git rev-parse --abbrev-ref HEAD)"
    if [[ "${current_branch}" != "main" ]]; then
        echo "Currently on branch '${current_branch}'. Switching to main..." >&2
        if [[ "${DRY_RUN}" == "0" ]]; then
            git checkout main
        else
            echo "[dry-run] git checkout main"
        fi
    fi

    log "Fetching origin/main..."
    if [[ "${DRY_RUN}" == "0" ]]; then
        git fetch origin main
        local local_sha remote_sha
        local_sha="$(git rev-parse HEAD)"
        remote_sha="$(git rev-parse origin/main)"
        if [[ "${local_sha}" != "${remote_sha}" ]]; then
            if git merge-base --is-ancestor HEAD origin/main; then
                log "Fast-forwarding main to origin/main..."
                git merge --ff-only origin/main
            else
                echo "Local main has diverged from origin/main. Resolve before releasing." >&2
                exit 1
            fi
        fi
    else
        echo "[dry-run] git fetch origin main && git merge --ff-only origin/main"
    fi
}

check_tag_does_not_exist() {
    if git rev-parse "refs/tags/${VERSION}" >/dev/null 2>&1; then
        echo "Tag ${VERSION} already exists locally." >&2
        exit 1
    fi

    local remote_tag
    remote_tag="$(git ls-remote --tags origin "refs/tags/${VERSION}" 2>/dev/null || true)"
    if [[ -n "${remote_tag}" ]]; then
        echo "Tag ${VERSION} already exists on origin." >&2
        exit 1
    fi
}

strip_v() {
    echo "${VERSION#v}"
}

bump_cargo_toml() {
    local version="$1"
    local cargo_toml="${PROJECT_ROOT}/Cargo.toml"
    log "Bumping Cargo.toml workspace version to ${version}..."
    if [[ "${DRY_RUN}" == "0" ]]; then
        if ! grep -qE '^version = "[^"]+"' "${cargo_toml}"; then
            echo "Could not find workspace version line in ${cargo_toml}" >&2
            exit 1
        fi
        sed -i -E "s/^version = \"[^\"]+\"/version = \"${version}\"/" "${cargo_toml}"
    fi
}

bump_package_json() {
    local version="$1"
    local package_json="$2"
    log "Bumping $(basename "${package_json}") version to ${version}..."
    if [[ "${DRY_RUN}" == "0" ]]; then
        if ! grep -qE '"version":\s*"[^"]+"' "${package_json}"; then
            echo "Could not find version field in ${package_json}" >&2
            exit 1
        fi
        sed -i -E "s/\"version\":\s*\"[^\"]+\"/\"version\": \"${version}\"/" "${package_json}"
    fi
}

bump_tauri_conf() {
    local version="$1"
    local tauri_conf="${PROJECT_ROOT}/desktop/src-tauri/tauri.conf.json"
    log "Bumping tauri.conf.json version to ${version}..."
    if [[ "${DRY_RUN}" == "0" ]]; then
        if ! grep -qE '"version":\s*"[^"]+"' "${tauri_conf}"; then
            echo "Could not find version field in ${tauri_conf}" >&2
            exit 1
        fi
        sed -i -E "s/\"version\":\s*\"[^\"]+\"/\"version\": \"${version}\"/" "${tauri_conf}"
    fi
}

generate_changelog_entry() {
    local version="$1"
    local date
    date="$(date +%Y-%m-%d)"
    local changelog="${PROJECT_ROOT}/CHANGELOG.md"

    log "Generating CHANGELOG.md entry for ${version}..."

    local previous_tag
    previous_tag="$(git describe --tags --abbrev=0 2>/dev/null || true)"

    local commits
    if [[ -n "${previous_tag}" ]]; then
        commits="$(git log "${previous_tag}..HEAD" --pretty=format:'- %s' --no-merges)"
    else
        commits="$(git log --pretty=format:'- %s' --no-merges)"
    fi

    if [[ -z "${commits}" ]]; then
        commits="- No changes recorded."
    fi

    local entry
    entry="## [${version}] - ${date}

${commits}
"

    if [[ "${DRY_RUN}" == "0" ]]; then
        {
            echo "# Changelog"
            echo
            echo "${entry}"
            if [[ -f "${changelog}" ]]; then
                tail -n +3 "${changelog}"
            fi
        } > "${changelog}.tmp"
        mv "${changelog}.tmp" "${changelog}"
    fi
}

bump_versions() {
    local version
    version="$(strip_v)"
    bump_cargo_toml "${version}"
    bump_package_json "${version}" "${PROJECT_ROOT}/desktop/package.json"
    bump_package_json "${version}" "${PROJECT_ROOT}/web/package.json"
    bump_tauri_conf "${version}"
    generate_changelog_entry "${VERSION}"
}

run_checks() {
    if [[ "${NO_CHECKS}" == "1" ]]; then
        log "Skipping validation checks (--no-checks)."
        return 0
    fi

    log "Running cargo fmt --check..."
    run cargo fmt --all -- --check

    log "Running cargo clippy..."
    run cargo clippy --workspace --all-targets --all-features -- -D warnings

    log "Running cargo test --workspace..."
    run cargo test --workspace
}

rust_target_installed() {
    local target="$1"
    rustup target list --installed 2>/dev/null | grep -qx "${target}"
}

tauri_build() {
    local target="$1"
    local bundles="$2"
    shift 2
    local extra_args=("$@")

    if [[ -n "${target}" ]]; then
        run_in "${PROJECT_ROOT}/desktop" pnpm tauri build --target "${target}" --bundles "${bundles}" "${extra_args[@]}"
    else
        run_in "${PROJECT_ROOT}/desktop" pnpm tauri build --bundles "${bundles}" "${extra_args[@]}"
    fi
}

build_desktop_bundles() {
    if [[ "${NO_CHECKS}" == "1" || "${NO_BUILD}" == "1" ]]; then
        log "Skipping local desktop builds."
        return 0
    fi

    log "Installing desktop frontend dependencies..."
    run_in "${PROJECT_ROOT}/desktop" pnpm install --frozen-lockfile

    local host_os
    host_os="$(uname -s)"

    case "${host_os}" in
        Linux)
            log "Building Linux desktop bundles (deb, appimage)..."
            tauri_build "" "deb,appimage"

            if rust_target_installed "x86_64-pc-windows-msvc"; then
                log "Building Windows desktop bundles (msi, nsis) via cross-compilation..."
                tauri_build "x86_64-pc-windows-msvc" "msi,nsis"
            else
                log "Skipping Windows cross-compilation: x86_64-pc-windows-msvc target not installed."
                log "The GitHub Actions release workflow will build Windows installers."
            fi

            if rust_target_installed "universal-apple-darwin" || \
               rust_target_installed "x86_64-apple-darwin" || \
               rust_target_installed "aarch64-apple-darwin"; then
                log "Building macOS desktop bundle (dmg) via cross-compilation..."
                if rust_target_installed "universal-apple-darwin"; then
                    tauri_build "universal-apple-darwin" "dmg"
                else
                    tauri_build "x86_64-apple-darwin" "dmg"
                    tauri_build "aarch64-apple-darwin" "dmg"
                fi
            else
                log "Skipping macOS cross-compilation: Apple target(s) not installed."
                log "The GitHub Actions release workflow will build the macOS installer."
            fi
            ;;
        Darwin)
            log "Building macOS desktop bundle (dmg)..."
            tauri_build "" "dmg"

            if rust_target_installed "x86_64-pc-windows-msvc"; then
                log "Building Windows desktop bundles (msi, nsis) via cross-compilation..."
                tauri_build "x86_64-pc-windows-msvc" "msi,nsis"
            else
                log "Skipping Windows cross-compilation: x86_64-pc-windows-msvc target not installed."
                log "The GitHub Actions release workflow will build Windows installers."
            fi
            ;;
        MINGW*|MSYS*|CYGWIN*)
            log "Building Windows desktop bundles (msi, nsis)..."
            tauri_build "" "msi,nsis"
            ;;
        *)
            echo "Unsupported host OS for local desktop builds: ${host_os}" >&2
            echo "Run with --no-build to skip local desktop builds." >&2
            exit 1
            ;;
    esac
}

run_builds() {
    if [[ "${NO_CHECKS}" == "1" || "${NO_BUILD}" == "1" ]]; then
        log "Skipping local builds."
        return 0
    fi

    log "Running server Docker build (scripts/build-server.sh)..."
    IMAGE_TAG="ruckchat-server:${VERSION}" run "${PROJECT_ROOT}/scripts/build-server.sh"

    build_desktop_bundles
}

commit_and_tag() {
    local version="$1"
    local sign_flags=()
    if gpg_configured; then
        sign_flags=("-S")
    fi

    log "Committing version bump and changelog..."
    if [[ "${DRY_RUN}" == "0" ]]; then
        git add -A
        GIT_AUTHOR_NAME="${AUTHOR_NAME}" GIT_AUTHOR_EMAIL="${AUTHOR_EMAIL}" \
        GIT_COMMITTER_NAME="${AUTHOR_NAME}" GIT_COMMITTER_EMAIL="${AUTHOR_EMAIL}" \
        git commit "${sign_flags[@]}" -m "Release ${version}"
    else
        echo "[dry-run] git commit -m \"Release ${version}\""
    fi

    log "Creating annotated tag ${version}..."
    if [[ "${DRY_RUN}" == "0" ]]; then
        git tag "${sign_flags[@]}" -a "${version}" -m "Release ${version}"
    else
        echo "[dry-run] git tag -a ${version} -m \"Release ${version}\""
    fi
}

push_release() {
    if [[ "${DRY_RUN}" == "1" ]]; then
        echo "[dry-run] git push origin main"
        echo "[dry-run] git push origin ${VERSION}"
        return 0
    fi

    log "Pushing release commit and tag to origin..."
    git push origin main
    git push origin "${VERSION}"
}

confirm() {
    if [[ "${DRY_RUN}" == "1" || "${NO_CONFIRM}" == "1" ]]; then
        return 0
    fi

    echo
    echo "Release plan:"
    echo "  Version:    ${VERSION}"
    echo "  Branch:     main"
    echo "  Bump files: $([[ ${NO_BUMP} == 1 ]] && echo no || echo yes)"
    echo "  Run checks: $([[ ${NO_CHECKS} == 1 ]] && echo no || echo yes)"
    echo "  Run builds: $([[ ${NO_CHECKS} == 1 || ${NO_BUILD} == 1 ]] && echo no || echo yes)"
    echo "  Commit/tag: $([[ ${NO_BUMP} == 1 ]] && echo skip || echo yes)"
    echo "  Push:       $([[ ${NO_BUMP} == 1 ]] && echo skip || echo yes)"
    echo
    read -r -p "Proceed? [y/N] " response
    case "${response}" in
        [yY][eE][sS]|[yY])
            ;;
        *)
            echo "Aborted."
            exit 1
            ;;
    esac
}

main() {
    cd "${PROJECT_ROOT}"

    parse_args "$@"

    if [[ "${DRY_RUN}" == "0" ]]; then
        require_gpg
        ensure_clean_tree
    fi

    ensure_on_main
    check_tag_does_not_exist

    if [[ "${NO_BUMP}" == "0" ]]; then
        bump_versions
    fi

    run_checks
    run_builds

    if [[ "${NO_BUMP}" == "0" ]]; then
        commit_and_tag "${VERSION}"
        confirm
        push_release
    else
        log "Version bump skipped (--no-bump); nothing to commit or push."
    fi

    log "Release ${VERSION} complete."
}

main "$@"
