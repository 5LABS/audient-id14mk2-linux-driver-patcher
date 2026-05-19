## # Audient iD14 mk 2 – Treiber
Das USB Audio Interface wird in Ubuntu angezeigt. Aber es wird über den Kanal "Monitor" nichts abgespielt.

## Runtime
- Ubuntu 26.04 LTS mit Linux Kernel 7
- Secure Boot aktiv

## Hardware

- **Gerät:** Audient iD14 MkII
- **USB:** `2708:0008`, High Speed (480 Mbps), UAC 2.0
- **Audio:** 6 Kanäle OUT / 12 Kanäle IN, 24-bit, `S32_LE`-Format
- **Abtastraten:** 44100, 48000, 88200, 96000 Hz
- **USB-Interfaces:**
  - Interface 0: AudioControl (UAC2) – `snd-usb-audio`
  - Interface 1: Streaming OUT (6ch, 24-bit) – `snd-usb-audio`
  - Interface 2: Streaming IN (12ch, 24-bit) – `snd-usb-audio`
  - Interface 3: HID (Encoder/Buttons) – `usbhid` → `/dev/input/eventN`
  - Interface 4: DFU (Firmware-Update)
- **Monitor-Knob:** Im Speaker/HP-Modus → direkter analoger Pegel, keine USB-Events. Im iD-Modus → REL_WHEEL-Events für DAW.
