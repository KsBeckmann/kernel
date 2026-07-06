#!/usr/bin/env bash
#
# Builds the bootloader + Rust kernel into a bootable disk image.
#
#   ./build.sh           build only -> build/os.img
#   ./build.sh run       build and launch QEMU
#   cargo run --release  cargo builds the kernel, then calls us as its runner
#
# When cargo invokes us as the runner, it passes the freshly-built kernel
# binary as $1 (an existing file). In that case we skip `cargo build` (cargo
# already did it) and go straight to assembling the image + QEMU.
#
set -euo pipefail
cd "$(dirname "$0")"   # always run from the project root

# cargo test builds its binary under .../deps/ ; use that to run quietly
IS_TEST=0
[[ "${1:-}" == *"/deps/"* ]] && IS_TEST=1
log() { [[ "$IS_TEST" == 1 ]] || echo "$@"; }
BUILD=build
SECTORS=120            # how many sectors the bootloader reads (must match boot.asm DAP)
IMG="$BUILD/os.img"

mkdir -p "$BUILD"

log "==> bootloader"
nasm -Ibootloader/ bootloader/boot.asm -o "$BUILD/boot.bin"

if [[ -f "${1:-}" ]]; then
    # called by cargo as a runner: $1 is the kernel binary cargo just built
    log "==> rust kernel (from cargo: $1)"
    cp "$1" "$BUILD/kernel.bin"
    RUN=1
else
    log "==> rust kernel"
    cargo build --release
    cp target/i686-unknown-none/release/kernel "$BUILD/kernel.bin"
    [[ "${1:-}" == "run" ]] && RUN=1 || RUN=0
fi

# safety check: the bootloader only loads $SECTORS sectors. NEVER silence this
# (a truncated kernel jumps into garbage -> triple fault -> endless reboot loop).
KSIZE=$(wc -c < "$BUILD/kernel.bin")
if (( KSIZE > SECTORS * 512 )); then
    echo "WARNING: kernel.bin is $KSIZE bytes ($(( (KSIZE + 511) / 512 )) sectors) but the \
bootloader only loads $((SECTORS * 512)) bytes ($SECTORS sectors). Bump 'dw' in boot.asm DAP \
and SECTORS here." >&2
fi

log "==> disk image"
cat "$BUILD/boot.bin" "$BUILD/kernel.bin" > "$IMG"
# pad to (1 boot sector + $SECTORS kernel sectors) so the disk read never runs past EOF
truncate -s $(( (SECTORS + 1) * 512 )) "$IMG"

log "==> done: $IMG ($(wc -c < "$IMG") bytes)"

if [[ "${RUN:-0}" == "1" ]]; then
    log "==> qemu"
    status=0

    if [[ "$IS_TEST" == 1 ]]; then
        # test mode: headless, serial output only, 5-min timeout. stdin from
        # /dev/null so QEMU does not grab the terminal and hang after exit_qemu.
        timeout 300 qemu-system-i386 -drive format=raw,file="$IMG" \
            -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
            -serial stdio -display none </dev/null || status=$?
    else
        # normal run: graphical window, interactive
        qemu-system-i386 -drive format=raw,file="$IMG" \
            -device isa-debug-exit,iobase=0xf4,iosize=0x04 || status=$?
    fi

    if [[ $status -eq 33 ]]; then
        log "==> tests passed"
        exit 0
    fi
    if [[ $status -eq 124 ]]; then
        echo "==> TIMED OUT (test travou por 300s)" >&2
        exit 1
    fi
    exit $status
fi
