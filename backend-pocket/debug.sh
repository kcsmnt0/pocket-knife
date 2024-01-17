#!/bin/bash
export $(cat .env | xargs)
cd litex
source .venv/bin/activate
python litex/tools/litex_term.py --jtag-config=openocd_usb_blaster.cfg jtag
