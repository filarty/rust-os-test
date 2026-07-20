#!/usr/bin/env bash
set -euo pipefail 

killall -q qemu-system-x86_64 || true

cargo build --target x86_64-unknown-uefi

mkdir -p esp/efi/boot

cp target/x86_64-unknown-uefi/debug/test-os.efi esp/efi/boot/bootx64.efi

sync

if [ ! -f OVMF_VARS_rw.fd ]; then
    cp OVMF_VARS.fd OVMF_VARS_rw.fd
    chmod +w OVMF_VARS_rw.fd
fi

qemu-system-x86_64 -enable-kvm \
    -cpu max \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.fd \
    -drive if=pflash,format=raw,readonly=off,file=OVMF_VARS_rw.fd \
    -drive format=raw,file=fat:rw:esp