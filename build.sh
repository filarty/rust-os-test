#!/usr/bin/env bash
set -euo pipefail

killall -q qemu-system-x86_64 || true

cargo build -p kernel --target x86_64-unknown-none

cargo build -p bootloader --target x86_64-unknown-uefi

mkdir -p esp/efi/boot
mkdir -p esp/boot

cp target/x86_64-unknown-uefi/debug/bootloader.efi esp/efi/boot/bootx64.efi
cp target/x86_64-unknown-none/debug/kernel esp/boot/kernel.bin

sync

echo
qemu-system-x86_64 -enable-kvm \
    -cpu max \
    -vga std \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.fd \
    -drive if=pflash,format=raw,readonly=off,file=OVMF_VARS_rw.fd \
    -drive format=raw,file=fat:rw:esp \
    -serial file:serial.log