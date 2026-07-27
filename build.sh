#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

IS_TEST=0
[[ "${1:-}" == *"/deps/"* ]] && IS_TEST=1
log() { [[ "$IS_TEST" == 1 ]] || echo "$@"; }
BUILD=build
SECTORS=120
IMG="$BUILD/os.img"

mkdir -p "$BUILD"

log "==> bootloader"
nasm -Ibootloader/ bootloader/boot.asm   -o build/boot.bin
nasm -Ibootloader/ bootloader/stage2.asm -o build/stage2.bin

if [[ -f "${1:-}" ]]; then
    log "==> rust kernel (from cargo: $1)"
    cp "$1" "$BUILD/kernel.bin"
    RUN=1
else
    log "==> rust kernel"
    cargo build --release
    cp target/x86_64-unknown-none/release/kernel "$BUILD/kernel.bin"
    [[ "${1:-}" == "run" ]] && RUN=1 || RUN=0
fi

KSIZE=$(wc -c < "$BUILD/kernel.bin")
if (( KSIZE > SECTORS * 512 )); then
    echo "WARNING: kernel.bin is $KSIZE bytes ($(( (KSIZE + 511) / 512 )) sectors) but the \
bootloader only loads $((SECTORS * 512)) bytes ($SECTORS sectors). Bump 'dw' in stage2.asm kernel_dap \
and SECTORS here." >&2
fi

log "==> disk image"
truncate -s $(( 8 * 512 )) build/stage2.bin

cat build/boot.bin build/stage2.bin build/kernel.bin > build/os.img

truncate -s $(( (1 + 8 + SECTORS) * 512 )) build/os.img

log "==> done: $IMG ($(wc -c < "$IMG") bytes)"

if [[ "${RUN:-0}" == "1" ]]; then
    log "==> qemu"
    status=0

    if [[ "$IS_TEST" == 1 ]]; then
        timeout 300 qemu-system-x86_64 -drive format=raw,file="$IMG" \
            -no-reboot \
            -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
            -serial stdio -display none </dev/null || status=$?
    else
        qemu-system-x86_64 -drive format=raw,file="$IMG" \
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
