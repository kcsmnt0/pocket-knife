#!/bin/bash
sudo mount /dev/sdf1 /mnt
sudo cp -v rust.bin /mnt/Assets/riscv/common/boot.bin
sudo cp -v data.json /mnt/Cores/agg23.RISCV/data.json
sudo cp -v interact.json /mnt/Cores/agg23.RISCV/interact.json
sudo umount /mnt
sudo sync
