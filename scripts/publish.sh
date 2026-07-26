#!/usr/bin/env bash
set -euo pipefail

# Unified build and release script for RuckChat.
#
# Replaces the legacy scripts/release.sh and scripts/build-server.sh workflows
# with a single, flag-driven entry point.
#
# Usage:
#   ./scripts/publish.sh [OPTIONS] <version>
#
# The version must be in the form vX.Y.Z or vX.Y.Z-<prerelease>,
# e.g. v0.2.0 or v1.0.0-rc.1.
#
# Options:
#   --dry-run      Print planned actions without executing them.
#   --no-confirm   Skip interactive confirmation prompts.
#   --no-checks    Skip fmt/clippy/test validation.
#   --no-bump      Skip modifying version files and CHANGELOG.md.
#   --no-build     Skip local builds (Web UI, SQLx prepare, Docker, cargo-deb,
#                  desktop bundles). Implies no local artifacts are produced.
#   --no-publish   Skip GHCR push and GitHub Release upload.
#   --publish-only Equivalent to --no-build --publish; publish existing artifacts.
#   --build-only   Run only the build steps; do not bump, commit, tag, or publish.
#   --no-desktop   Skip desktop frontend deps and desktop bundle builds/uploads.
#   -h, --help     Show this help message.
#
# Examples:
#   ./scripts/publish.sh --dry-run v0.3.0
#   ./scripts/publish.sh v0.3.0
#   ./scripts/publish.sh --no-build --publish v0.3.0
#   ./scripts/publish.sh --build-only v0.3.0
#   ./scripts/publish.sh --no-desktop v0.3.0

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Load local test environment if present and DATABASE_URL is unset.
if [[ -z "${DATABASE_URL:-}" && -f "${PROJECT_ROOT}/.env.testing" ]]; then
    set -a
    # shellcheck source=/dev/null
    source "${PROJECT_ROOT}/.env.testing"
    set +a
fi

DRY_RUN=0
NO_CONFIRM=0
NO_CHECKS=0
NO_BUMP=0
NO_BUILD=0
NO_PUBLISH=0
BUILD_ONLY=0
NO_DESKTOP=0

VERSION=""

AUTHOR_NAME="Brian Tafoya"
AUTHOR_EMAIL="btafoya@briantafoya.com"

usage() {
    cat <<USAGE
Usage: $0 [OPTIONS] <version>

Publish a new RuckChat version. The version must be in the form
vX.Y.Z or vX.Y.Z-<prerelease>, e.g. v0.2.0 or v1.0.0-rc.1.

Options:
  --dry-run      Print planned actions without executing them.
  --no-confirm   Skip interactive confirmation prompts.
  --no-checks    Skip fmt/clippy/test validation.
  --no-bump      Skip modifying version files and CHANGELOG.md.
  --no-build     Skip local builds (Web UI, SQLx, Docker, cargo-deb, desktop).
  --no-publish   Skip GHCR push and GitHub Release upload.
  --publish-only Equivalent to --no-build --publish; publish existing artifacts.
  --build-only   Run only builds; do not bump, commit, tag, or publish.
  --no-desktop   Skip desktop frontend deps and desktop bundle builds/uploads.
  -h, --help     Show this help message.

Examples:
  $0 --dry-run v0.3.0
  $0 v0.3.0
  $0 --no-build --publish v0.3.0
  $0 --build-only v0.3.0
  $0 --no-desktop v0.3.0
USAGE
}

log() {
    echo "[publish] $*"
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
            --no-publish)
                NO_PUBLISH=1
                ;;
            --publish-only)
                NO_BUILD=1
                NO_BUMP=1
                ;;
            --build-only)
                BUILD_ONLY=1
                ;;
            --no-desktop)
                NO_DESKTOP=1
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
        echo "Expected vX.Y.Z or vX.Y.Z-<prerelease>, e.g. v0.2.0 or v1.0.0-rc.1" >&2
        exit 1
    fi

    # Flag conflict checks.
    if [[ "${BUILD_ONLY}" == "1" && "${NO_BUILD}" == "1" ]]; then
        echo "Conflict: --build-only and --no-build cannot be used together." >&2
        exit 1
    fi
    if [[ "${BUILD_ONLY}" == "1" && "${NO_PUBLISH}" == "0" ]]; then
        echo "Conflict: --build-only implies no publishing; do not use with publishing." >&2
        exit 1
    fi
    if [[ "${BUILD_ONLY}" == "1" && "${NO_BUMP}" == "0" ]]; then
        echo "Conflict: --build-only implies no version bump; use --no-bump or --build-only alone." >&2
        exit 1
    fi

    if [[ "${NO_BUILD}" == "1" && "${NO_PUBLISH}" == "1" && "${BUILD_ONLY}" == "0" ]]; then
        log "Warning: --no-build and --no-publish together mean nothing will be built or published."
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

require_gh() {
    if ! command -v gh &>/dev/null; then
        echo "The 'gh' CLI is required for publishing. Install it and run 'gh auth login'." >&2
        exit 1
    fi
    if ! gh auth status &>/dev/null; then
        echo "The 'gh' CLI is not authenticated. Run 'gh auth login' first." >&2
        exit 1
    fi
}

ensure_clean_tree() {
    if [[ -n "$(git status --porcelain)" ]]; then
        echo "Working tree is not clean. Commit or stash changes before publishing." >&2
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
            if git merge-base --is-ancestor origin/main HEAD; then
                log "Local main is ahead of origin/main."
            elif git merge-base --is-ancestor HEAD origin/main; then
                log "Fast-forwarding main to origin/main..."
                git merge --ff-only origin/main
            else
                echo "Local main has diverged from origin/main. Resolve before publishing." >&2
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

ghcr_owner() {
    local remote_url
    remote_url="$(git remote get-url origin 2>/dev/null || true)"
    if [[ "${remote_url}" =~ github\.com[/:]([^/]+)/ ]]; then
        echo "${BASH_REMATCH[1]}"
        return 0
    fi

    local gh_owner
    gh_owner="$(gh repo view --json owner --jq '.owner.login' 2>/dev/null || true)"
    if [[ -n "${gh_owner}" ]]; then
        echo "${gh_owner}"
        return 0
    fi

    echo "Could not determine GitHub owner from git remote or gh CLI." >&2
    exit 1
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

build_web_ui() {
    log "Building Web UI assets..."
    run_in "${PROJECT_ROOT}/web" pnpm install
    run_in "${PROJECT_ROOT}/web" pnpm build
}

prepare_sqlx() {
    log "Preparing SQLx offline query data..."
    run cargo sqlx prepare --workspace
}

build_server_image() {
    log "Building server Docker image ruckchat-server:${VERSION}..."
    run docker build -t "ruckchat-server:${VERSION}" .

    log "Tagging ruckchat-server:latest for local use..."
    run docker tag "ruckchat-server:${VERSION}" "ruckchat-server:latest"
}

ensure_cargo_deb() {
    if command -v cargo-deb &>/dev/null; then
        return 0
    fi
    log "cargo-deb not found; installing..."
    run cargo install cargo-deb
    if [[ "${DRY_RUN}" == "1" ]]; then
        return 0
    fi
    if ! command -v cargo-deb &>/dev/null; then
        echo "cargo-deb installation failed. Install it manually: cargo install cargo-deb" >&2
        exit 1
    fi
}

build_server_deb() {
    ensure_cargo_deb
    log "Building server .deb package..."
    run cargo deb -p ruckchat-server
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
    if [[ "${NO_DESKTOP}" == "1" ]]; then
        log "Skipping desktop bundle builds (--no-desktop)."
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
            fi
            ;;
        MINGW*|MSYS*|CYGWIN*)
            log "Building Windows desktop bundles (msi, nsis)..."
            tauri_build "" "msi,nsis"
            ;;
        *)
            echo "Unsupported host OS for local desktop builds: ${host_os}" >&2
            echo "Run with --no-desktop to skip desktop bundle builds." >&2
            exit 1
            ;;
    esac
}

build_all() {
    if [[ "${NO_BUILD}" == "1" ]]; then
        log "Skipping local builds (--no-build)."
        return 0
    fi

    build_web_ui
    prepare_sqlx
    build_server_image
    build_server_deb
    build_desktop_bundles
}

commit_and_tag() {
    local version="$1"
    local commit_sign_flags=()
    local tag_sign_flags=()
    if gpg_configured; then
        commit_sign_flags=("-S")
        tag_sign_flags=("-s")
    fi

    log "Committing version bump and changelog..."
    if [[ "${DRY_RUN}" == "0" ]]; then
        git add -A
        GIT_AUTHOR_NAME="${AUTHOR_NAME}" GIT_AUTHOR_EMAIL="${AUTHOR_EMAIL}" \
        GIT_COMMITTER_NAME="${AUTHOR_NAME}" GIT_COMMITTER_EMAIL="${AUTHOR_EMAIL}" \
        git commit "${commit_sign_flags[@]}" -m "Release ${version}"
    else
        echo "[dry-run] git commit -m \"Release ${version}\""
    fi

    log "Creating annotated tag ${version}..."
    if [[ "${DRY_RUN}" == "0" ]]; then
        git tag "${tag_sign_flags[@]}" "${version}" -m "Release ${version}"
    else
        echo "[dry-run] git tag -s ${version} -m \"Release ${version}\""
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

changelog_section_for_version() {
    local version="$1"
    local changelog="${PROJECT_ROOT}/CHANGELOG.md"
    if [[ ! -f "${changelog}" ]]; then
        echo "Release ${version}"
        return 0
    fi

    local start_line end_line
    start_line="$(grep -n "^## \[${version}\]" "${changelog}" | head -1 | cut -d: -f1 || true)"
    if [[ -z "${start_line}" ]]; then
        echo "Release ${version}"
        return 0
    fi

    end_line="$(tail -n +$((start_line + 1)) "${changelog}" | grep -n '^## \[' | head -1 | cut -d: -f1 || true)"
    if [[ -n "${end_line}" ]]; then
        end_line=$((start_line + end_line - 1))
    else
        end_line="$(wc -l < "${changelog}")"
    fi

    sed -n "${start_line},${end_line}p" "${changelog}"
}

is_prerelease() {
    local stripped
    stripped="$(strip_v)"
    [[ "${stripped}" == *-* ]]
}

publish_docker_image() {
    if [[ "${NO_PUBLISH}" == "1" ]]; then
        log "Skipping Docker image publish (--no-publish)."
        return 0
    fi

    local owner image_base version_tag latest_tag
    owner="$(ghcr_owner)"
    image_base="ghcr.io/${owner}/ruckchat-server"
    version_tag="${image_base}:${VERSION}"
    latest_tag="${image_base}:latest"

    log "Publishing Docker image to GHCR..."
    run docker tag "ruckchat-server:${VERSION}" "${version_tag}"
    run docker push "${version_tag}"

    run docker tag "ruckchat-server:${VERSION}" "${latest_tag}"
    run docker push "${latest_tag}"
}

server_deb_artifact() {
    # cargo-deb mangles the version (e.g. "0.2.1-alpha-r3" becomes
    # "0.2.1~alpha-r3-1"), so match by name prefix and freshness rather than
    # reconstructing its exact Debian version-string transformation.
    ls -t "${PROJECT_ROOT}"/target/debian/ruckchat-server_*.deb 2>/dev/null | head -1
}

desktop_bundle_files() {
    local bundle_dir="${PROJECT_ROOT}/target/release/bundle"
    if [[ ! -d "${bundle_dir}" ]]; then
        return 0
    fi

    find "${bundle_dir}" -maxdepth 2 -type f \( \
        -name '*.deb' -o \
        -name '*.AppImage' -o \
        -name '*.dmg' -o \
        -name '*.msi' \
    \) 2>/dev/null | sort
}

confirm_publish() {
    if [[ "${DRY_RUN}" == "1" || "${NO_CONFIRM}" == "1" ]]; then
        return 0
    fi

    echo
    echo "External publish plan:"
    echo "  Version:        ${VERSION}"
    echo "  GHCR image:     ghcr.io/$(ghcr_owner)/ruckchat-server:${VERSION}"
    echo "  GHCR latest:    ghcr.io/$(ghcr_owner)/ruckchat-server:latest"
    echo "  Prerelease:     $([[ "$(is_prerelease && echo yes || echo no)" == "yes" ]] && echo yes || echo no)"
    echo "  Server .deb:    $(server_deb_artifact || echo 'not found')"
    echo "  Desktop bundles:"
    local bundle
    while IFS= read -r bundle; do
        [[ -n "${bundle}" ]] && echo "    $(basename "${bundle}")"
    done <<< "$(desktop_bundle_files)"
    echo
    read -r -p "Proceed with publish? [y/N] " response
    case "${response}" in
        [yY][eE][sS]|[yY])
            ;;
        *)
            echo "Aborted."
            exit 1
            ;;
    esac
}

publish_github_release() {
    if [[ "${NO_PUBLISH}" == "1" ]]; then
        log "Skipping GitHub Release publish (--no-publish)."
        return 0
    fi

    local owner repo release_args=() prerelease_flag=0
    owner="$(ghcr_owner)"
    repo="${owner}/ruckchat"

    local body_file
    body_file="$(mktemp)"
    changelog_section_for_version "${VERSION}" > "${body_file}"

    if is_prerelease; then
        release_args+=("--prerelease")
        prerelease_flag=1
    fi

    log "Creating GitHub Release ${VERSION}..."
    if [[ "${DRY_RUN}" == "1" ]]; then
        echo "[dry-run] gh release create ${VERSION} --repo ${repo} --title ${VERSION} --notes-file ${body_file} ${release_args[*]}"
    else
        gh release create "${VERSION}" \
            --repo "${repo}" \
            --title "${VERSION}" \
            --notes-file "${body_file}" \
            "${release_args[@]}"
    fi

    local deb_path
    deb_path="$(server_deb_artifact || true)"
    if [[ -n "${deb_path}" && -f "${deb_path}" ]]; then
        log "Uploading server .deb..."
        run gh release upload "${VERSION}" --repo "${repo}" "${deb_path}"
    else
        log "Warning: server .deb not found at expected path; skipping upload."
    fi

    log "Uploading desktop bundles..."
    local bundle_count=0
    while IFS= read -r bundle; do
        if [[ -n "${bundle}" && -f "${bundle}" ]]; then
            run gh release upload "${VERSION}" --repo "${repo}" "${bundle}"
            bundle_count=$((bundle_count + 1))
        fi
    done <<< "$(desktop_bundle_files)"

    if [[ "${bundle_count}" -eq 0 ]]; then
        log "Warning: no desktop bundles found; skipping desktop uploads."
    fi

    if [[ "${DRY_RUN}" == "0" ]]; then
        rm -f "${body_file}"
    fi
}

confirm_release_plan() {
    if [[ "${DRY_RUN}" == "1" || "${NO_CONFIRM}" == "1" || "${BUILD_ONLY}" == "1" ]]; then
        return 0
    fi

    echo
    echo "Release plan:"
    echo "  Version:      ${VERSION}"
    echo "  Branch:       main"
    echo "  Bump files:   $([[ ${NO_BUMP} == 1 ]] && echo no || echo yes)"
    echo "  Run checks:   $([[ ${NO_CHECKS} == 1 ]] && echo no || echo yes)"
    echo "  Run builds:   $([[ ${NO_BUILD} == 1 ]] && echo no || echo yes)"
    echo "  Desktop:      $([[ ${NO_DESKTOP} == 1 ]] && echo skipped || echo yes)"
    echo "  Commit/tag:   $([[ ${NO_BUMP} == 1 ]] && echo skip || echo yes)"
    echo "  Push:         $([[ ${NO_BUMP} == 1 ]] && echo skip || echo yes)"
    echo "  Publish:      $([[ ${NO_PUBLISH} == 1 ]] && echo skip || echo yes)"
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

    if [[ "${DRY_RUN}" == "0" && "${BUILD_ONLY}" == "0" && "${NO_BUMP}" == "0" ]]; then
        require_gpg
        ensure_clean_tree
    fi

    if [[ "${BUILD_ONLY}" == "0" && "${NO_BUMP}" == "0" ]]; then
        ensure_on_main
        check_tag_does_not_exist
    fi

    confirm_release_plan

    if [[ "${NO_BUMP}" == "0" && "${BUILD_ONLY}" == "0" ]]; then
        bump_versions
    fi

    run_checks
    build_all

    if [[ "${BUILD_ONLY}" == "1" ]]; then
        log "Build-only mode complete."
        exit 0
    fi

    if [[ "${NO_BUMP}" == "0" ]]; then
        commit_and_tag "${VERSION}"
        push_release
    fi

    if [[ "${NO_PUBLISH}" == "0" ]]; then
        require_gh
        confirm_publish
        publish_docker_image
        publish_github_release
    fi

    log "Publish ${VERSION} complete."
}

main "$@"
