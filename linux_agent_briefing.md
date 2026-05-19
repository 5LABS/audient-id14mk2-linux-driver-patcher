# Briefing für Linux-Agent: Audient ID14 Treiber

## Gerät
- **Hersteller:** Audient
- **Modell:** ID14
- **VID:** 0x2708
- **PID:** 0x0008
- **Firmware:** v1.12 (bcdDevice 0x0112)
- **USB:** 2.0 High-Speed

## USB-Klasse
**UAC2 (USB Audio Class 2.0)** — bFunctionProtocol 0x20  
Das Gerät ist vollständig UAC2-konform. Kein proprietäres Protokoll auf Streaming-Ebene.

## Konfiguration (Config 1, aktiv unter Windows)
5 Interfaces:

| Interface | Klasse | Funktion |
|---|---|---|
| 0 | Audio Control | Topologie, Clock, Routing |
| 1 | Audio Streaming | Playback (Host → Gerät) |
| 2 | Audio Streaming | Record (Gerät → Host) |
| 3 | HID | Knöpfe, Regler, Meter |
| 4 | DFU | Firmware-Update (Runtime) |

## Streaming-Endpoints

### Playback (Interface 1, AlternateSetting 1)
| Feld | Wert |
|---|---|
| Endpoint | 0x01 OUT |
| Kanäle | 6 |
| Format | PCM, 24-bit in 32-bit Container |
| wMaxPacketSize | 312 Bytes |
| bInterval | 1 (125µs, High-Speed) |
| Synchronisation | Asynchronous |
| Max. Samplerate | 96kHz |

### Record (Interface 2, AlternateSetting 1)
| Feld | Wert |
|---|---|
| Endpoint | 0x81 IN |
| Kanäle | 12 |
| Format | PCM, 24-bit in 32-bit Container |
| wMaxPacketSize | 624 Bytes |
| bInterval | 1 (125µs, High-Speed) |
| Synchronisation | Asynchronous + Implicit Feedback (bmAttributes 0x25) |
| Max. Samplerate | 96kHz |

### HID (Interface 3)
| Endpoint | 0x83 IN, wMaxPacketSize 64, bInterval 8 |
|---|---|

## Clock-Topologie
| Entity | Typ |
|---|---|
| 41 | Interner programmierbarer Takt (Standard) |
| 44 | Externer Takt (S/PDIF / ADAT Wordclock) |
| 40 | Clock Selector (Host wählbar) |

Der Treiber muss Clock Entity 41 aktivieren und die Samplerate setzen, bevor Audio-Streaming startet.

## Bekannte Problempunkte für snd-usb-audio

1. **Extension Units** — mehrere proprietäre Extension Units in der Audio Control Topologie; snd-usb-audio könnte beim Parsen abbrechen
2. **Clock-Setup** — Clock Entity 41 muss explizit gewählt und Samplerate geschrieben werden
3. **AlternateSetting** — beide Streaming-Interfaces starten auf Alt 0 (zero-bandwidth); Treiber muss auf Alt 1 umschalten
4. **Implicit Feedback** — Record-Endpoint hat bmAttributes 0x25; erfordert korrekte Feedback-Behandlung in snd-usb-audio

## Erste Diagnoseschritte unter Linux

```bash
# Gerät einstecken, dann sofort:
dmesg | grep -i "2708\|audient\|usb-audio\|uac"

# Zeigt ob ALSA das Gerät erkennt:
aplay -l
arecord -l
cat /proc/asound/cards

# Detaillierte USB-Info:
lsusb -v -d 2708:0008

# snd-usb-audio manuell laden mit Debug:
modprobe snd-usb-audio
dmesg | tail -50
```

## Dateien auf Windows-Rechner
- `C:\Users\thisi\Desktop\get desc resp conf.pcapng` — Wireshark-Capture der Initialisierung
- `G:\Projekte\2026_Projekte\audient_treiber\device_analysis.md` — vollständige Descriptor-Analyse

## Ziel
Herausfinden, ob `snd-usb-audio` mit einem kleinen Quirk-Patch ausreicht, oder ob ein eigener Kernel-Treiber nötig ist. Die dmesg-Ausgabe nach dem Einstecken ist der wichtigste erste Datenpunkt.
