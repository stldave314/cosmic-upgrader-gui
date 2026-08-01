#!/usr/bin/env bash
#
# Build, install and package cosmic-upgrader-gui.
#
# One script so that CI and a local build take the same path — in particular so
# that every packaging target passes `--features release-build`, which forces
# diagnostic logging off at compile time. A release cannot ship with logging on
# by forgetting a step here, because there is only one step.
#
#   ./install.sh build      Release build
#   ./install.sh install    Build and install into the system (needs root)
#   ./install.sh uninstall  Remove an installed copy (needs root)
#   ./install.sh deb        Build a .deb
#   ./install.sh rpm        Build an .rpm
#   ./install.sh tarball    Build a portable tarball
#   ./install.sh packages   deb + rpm + tarball
#   ./install.sh hooks      Install the repository's git hooks
#   ./install.sh check      cargo check, clippy and the test suite
#
# Set BUNDLE_TOPGRADE=1 to build a copy of topgrade into the package for
# systems that have none of their own.

set -euo pipefail

APP_ID="com.github.cosmic_upgrader_gui"
BIN="cosmic-upgrader-gui"
PREFIX="${PREFIX:-/usr}"
DESTDIR="${DESTDIR:-}"
DIST="dist"

# Forced on for everything that produces an artefact someone else will run.
FEATURES="release-build"
if [ "${BUNDLE_TOPGRADE:-0}" = "1" ]; then
    FEATURES="$FEATURES,bundled-topgrade"
fi

root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$root"

version() { grep -m1 '^version' Cargo.toml | cut -d'"' -f2; }

info() { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
die()  { printf '\033[1;31mError:\033[0m %s\n' "$*" >&2; exit 1; }

need_root() {
    [ "$(id -u)" -eq 0 ] || die "this needs root; re-run with sudo"
}

require() {
    command -v "$1" >/dev/null 2>&1 || die "$1 is not installed${2:+ ($2)}"
}

build() {
    info "Building $BIN $(version) with features: $FEATURES"
    cargo build --release --features "$FEATURES"

    if [ "${BUNDLE_TOPGRADE:-0}" = "1" ]; then
        build_bundled_topgrade
    fi
}

# Build topgrade into the tree so a package can carry it.
#
# Installed to a private libexec directory rather than onto PATH: this copy is a
# fallback for systems with no topgrade, and putting a second one on PATH would
# shadow whatever the user installs later.
build_bundled_topgrade() {
    require cargo
    local out="target/bundled"
    info "Building bundled topgrade into $out"
    cargo install topgrade --root "$out" --locked --force
    [ -x "$out/bin/topgrade" ] || die "bundled topgrade was not produced"
}

check() {
    info "cargo check"
    cargo check --all-targets
    info "cargo clippy"
    cargo clippy --all-targets -- -D warnings
    info "cargo test"
    cargo test
    info "locale check"
    check_locales
}

# Every locale must carry the same keys as the fallback, with the same
# placeholders. Locale files fall back silently, so a missing key shows up as
# stray English at runtime rather than as a build error — which is exactly the
# kind of thing that needs checking mechanically.
check_locales() {
    python3 - <<'PY'
import re, sys, pathlib

root = pathlib.Path("i18n")
fallback = root / "en" / "cosmic_upgrader_gui.ftl"
if not fallback.exists():
    sys.exit("i18n/en/cosmic_upgrader_gui.ftl is missing")

key_re = re.compile(r"^([a-zA-Z][a-zA-Z0-9_-]*)\s*=", re.M)
placeholder_re = re.compile(r"\{\s*\$([a-zA-Z0-9_]+)\s*\}")

def entries_of(path):
    """Key -> set of placeholder names, ignoring comments."""
    text = path.read_text(encoding="utf-8")
    text = "\n".join(l for l in text.splitlines() if not l.lstrip().startswith("#"))
    found = list(key_re.finditer(text))
    result = {}
    for index, match in enumerate(found):
        end = found[index + 1].start() if index + 1 < len(found) else len(text)
        result[match.group(1)] = set(placeholder_re.findall(text[match.start():end]))
    return result, [m.group(1) for m in found]

reference, _ = entries_of(fallback)
failed = False

for locale in sorted(p for p in root.iterdir() if p.is_dir()):
    path = locale / "cosmic_upgrader_gui.ftl"
    if not path.exists():
        print(f"  {locale.name}: MISSING {path}")
        failed = True
        continue

    entries, order = entries_of(path)
    problems = []

    if missing := sorted(set(reference) - set(entries)):
        problems.append(f"missing {missing}")
    if orphaned := sorted(set(entries) - set(reference)):
        problems.append(f"orphaned {orphaned}")
    if duplicates := sorted({k for k in order if order.count(k) > 1}):
        problems.append(f"duplicate {duplicates}")

    # Placeholders only surface at runtime, in that language, so a dropped
    # `{ $version }` is invisible until someone runs in that locale.
    for key in sorted(set(reference) & set(entries)):
        if reference[key] != entries[key]:
            problems.append(
                f"placeholders differ for {key!r}: "
                f"en={sorted(reference[key])} {locale.name}={sorted(entries[key])}"
            )

    if problems:
        failed = True
        print(f"  {locale.name}: " + "; ".join(problems))
    else:
        print(f"  {locale.name}: {len(entries)} keys OK")

sys.exit(1 if failed else 0)
PY
}

install_files() {
    need_root
    build

    info "Installing into ${DESTDIR}${PREFIX}"
    install -Dm755 "target/release/$BIN"        "${DESTDIR}${PREFIX}/bin/$BIN"
    install -Dm644 resources/app.desktop        "${DESTDIR}${PREFIX}/share/applications/${APP_ID}.desktop"
    install -Dm644 resources/app.metainfo.xml   "${DESTDIR}${PREFIX}/share/metainfo/${APP_ID}.metainfo.xml"
    install -Dm644 resources/icon.svg           "${DESTDIR}${PREFIX}/share/icons/hicolor/scalable/apps/${APP_ID}.svg"

    if [ "${BUNDLE_TOPGRADE:-0}" = "1" ]; then
        install -Dm755 target/bundled/bin/topgrade \
            "${DESTDIR}${PREFIX}/libexec/$BIN/topgrade"
    fi

    if [ -z "$DESTDIR" ]; then
        update-desktop-database "${PREFIX}/share/applications" 2>/dev/null || true
        gtk-update-icon-cache -f -t "${PREFIX}/share/icons/hicolor" 2>/dev/null || true
    fi
    info "Installed. Run '$BIN' or find \"Upgrader\" in the applications menu."
}

uninstall_files() {
    need_root
    info "Removing $BIN"

    # The scheduled units belong to the user who created them, not to root, so
    # they are left alone here — removing them would need to guess at every home
    # directory on the machine. The application disables its own timer when the
    # schedule is switched off.
    rm -f "${DESTDIR}${PREFIX}/bin/$BIN" \
          "${DESTDIR}${PREFIX}/share/applications/${APP_ID}.desktop" \
          "${DESTDIR}${PREFIX}/share/metainfo/${APP_ID}.metainfo.xml" \
          "${DESTDIR}${PREFIX}/share/icons/hicolor/scalable/apps/${APP_ID}.svg"
    rm -rf "${DESTDIR}${PREFIX}/libexec/$BIN"

    if [ -z "$DESTDIR" ]; then
        update-desktop-database "${PREFIX}/share/applications" 2>/dev/null || true
        gtk-update-icon-cache -f -t "${PREFIX}/share/icons/hicolor" 2>/dev/null || true
    fi
    info "Removed. A scheduled timer, if you set one up, is still in ~/.config/systemd/user."
}

deb() {
    require cargo-deb "cargo install cargo-deb"
    mkdir -p "$DIST"
    info "Building .deb"
    # cargo-deb runs its own build, so the feature has to be handed through
    # explicitly rather than relying on the earlier one.
    cargo deb --output "$DIST" -- --features "$FEATURES"
    info "Wrote $(ls -1 "$DIST"/*.deb | tail -1)"
}

rpm() {
    require cargo-generate-rpm "cargo install cargo-generate-rpm"
    mkdir -p "$DIST"
    info "Building .rpm"
    # cargo-generate-rpm packages an existing build rather than making one, so
    # the release build has to happen first and with the right feature.
    build
    cargo generate-rpm --output "$DIST"
    info "Wrote $(ls -1 "$DIST"/*.rpm | tail -1)"
}

tarball() {
    build
    local name="${BIN}-$(version)-x86_64-linux"
    local stage="$DIST/$name"
    info "Building tarball"

    rm -rf "$stage"
    mkdir -p "$stage/bin" "$stage/share/applications" "$stage/share/metainfo" \
             "$stage/share/icons/hicolor/scalable/apps"

    install -Dm755 "target/release/$BIN"      "$stage/bin/$BIN"
    install -Dm644 resources/app.desktop      "$stage/share/applications/${APP_ID}.desktop"
    install -Dm644 resources/app.metainfo.xml "$stage/share/metainfo/${APP_ID}.metainfo.xml"
    install -Dm644 resources/icon.svg         "$stage/share/icons/hicolor/scalable/apps/${APP_ID}.svg"
    install -Dm644 README.md LICENSE          "$stage/"

    if [ "${BUNDLE_TOPGRADE:-0}" = "1" ]; then
        install -Dm755 target/bundled/bin/topgrade "$stage/libexec/$BIN/topgrade"
    fi

    tar -C "$DIST" -czf "$DIST/$name.tar.gz" "$name"
    rm -rf "$stage"
    info "Wrote $DIST/$name.tar.gz"
}

hooks() {
    [ -d .git ] || die "not a git repository"
    info "Installing git hooks"
    git config core.hooksPath .githooks
    chmod +x .githooks/* 2>/dev/null || true
    info "Hooks installed from .githooks"
}

case "${1:-build}" in
    build)     build ;;
    install)   install_files ;;
    uninstall) uninstall_files ;;
    deb)       deb ;;
    rpm)       rpm ;;
    tarball)   tarball ;;
    packages)  deb; rpm; tarball ;;
    hooks)     hooks ;;
    check)     check ;;
    locales)   check_locales ;;
    *)         die "unknown target '${1}'. Try: build install uninstall deb rpm tarball packages hooks check" ;;
esac
