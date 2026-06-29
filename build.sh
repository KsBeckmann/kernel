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

BUILD=build
SECTORS=16             # how many sectors the bootloader reads (must match boot.asm)
IMG="$BUILD/os.img"

mkdir -p "$BUILD"

echo "==> bootloader"
nasm -Ibootloader/ bootloader/boot.asm -o "$BUILD/boot.bin"

if [[ -f "${1:-}" ]]; then
    # called by cargo as a runner: $1 is the kernel binary cargo just built
    echo "==> rust kernel (from cargo: $1)"
    cp "$1" "$BUILD/kernel.bin"
    RUN=1
else
    echo "==> rust kernel"
    cargo build --release
    cp target/i686-unknown-none/release/kernel "$BUILD/kernel.bin"
    [[ "${1:-}" == "run" ]] && RUN=1 || RUN=0
fi

# safety check: the bootloader only loads $SECTORS sectors
KSIZE=$(wc -c < "$BUILD/kernel.bin")
if (( KSIZE > SECTORS * 512 )); then
    echo "WARNING: kernel.bin is $KSIZE bytes but the bootloader only loads \
$((SECTORS * 512)) bytes ($SECTORS sectors). Increase AL in boot.asm and SECTORS here." >&2
fi

echo "==> disk image"
cat "$BUILD/boot.bin" "$BUILD/kernel.bin" > "$IMG"
# pad to (1 boot sector + $SECTORS kernel sectors) so the disk read never runs past EOF
truncate -s $(( (SECTORS + 1) * 512 )) "$IMG"

echo "==> done: $IMG ($(wc -c < "$IMG") bytes)"

if [[ "${RUN:-0}" == "1" ]]; then
    echo "==> qemu"
    exec qemu-system-i386 -drive format=raw,file="$IMG"
fi
