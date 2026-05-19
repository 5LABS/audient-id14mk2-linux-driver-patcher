# Audient ID14 — USB Analyse

## Device Descriptor
| Feld | Wert |
|---|---|
| idVendor | 0x2708 (Audient) |
| idProduct | 0x0008 (ID14) |
| bcdDevice | 0x0112 (Firmware v1.12) |
| bcdUSB | 0x0200 (USB 2.0) |
| bDeviceClass | 0xEF (Composite / IAD) |
| bNumConfigurations | 2 |

## Configuration 1 (aktiv unter Windows)
- bConfigurationValue: 1
- bNumInterfaces: 5
- Stromaufnahme: 500mA (Bus-powered)

## USB Audio Class Version
**UAC2 (USB Audio Class 2.0)** — bFunctionProtocol: 0x20
→ snd-usb-audio unter Linux unterstützt UAC2 grundsätzlich

## Interface Association: Audio (Interfaces 0–2)
bFunctionClass: Audio, bFunctionProtocol: 0x20 (UAC2)

## Interface 0: Audio Control
Kategorie: I/O box (0x08)

### Clock-Topologie
| Entity | Typ | Beschreibung |
|---|---|---|
| 41 | Internal programmable clock | Interner Takt (Standard) |
| 44 | External clock | Extern (S/PDIF / ADAT Word Clock) |
| 40 | Clock Selector | Wählt zwischen 41 und 44 |

→ Host kann Samplerate und Clockquelle umschalten

### Input Terminal (ID 2)
- Typ: USB Streaming (0x0101)
- 6 Kanäle (Playback vom Host)
- Verbunden mit Clock Selector 40

### Feature Unit (ID 10)
- Source: Entity 51 (noch nicht analysiert)
- Alle Controls: Not present
- → Mixing/Routing vermutlich über proprietäre Vendor Controls

## Audio Streaming — Interface 1 (Playback → Host → Gerät)
| Feld | Wert |
|---|---|
| Endpoint | 0x01 OUT |
| Kanäle | **6** |
| Format | PCM, 24-bit in 32-bit Container (Subslot 4, BitRes 24) |
| wMaxPacketSize | 312 Bytes |
| bInterval | 1 (125µs Microframes → USB High-Speed) |
| Synchronisation | **Asynchronous** |
| Max. Samplerate | **96kHz** (13 Samples × 6ch × 4B = 312 Bytes) |

## Audio Streaming — Interface 2 (Record → Gerät → Host)
| Feld | Wert |
|---|---|
| Endpoint | 0x81 IN |
| Kanäle | **12** |
| Format | PCM, 24-bit in 32-bit Container |
| wMaxPacketSize | 624 Bytes |
| bInterval | 1 (125µs) |
| Synchronisation | Asynchronous + Implicit Feedback |
| Max. Samplerate | **96kHz** (13 Samples × 12ch × 4B = 624 Bytes) |

## Interface 3: HID (Knöpfe / Regler)
- Endpoint 0x83 IN, wMaxPacketSize 64, bInterval 8
- Standard HID → funktioniert unter Linux out-of-the-box

## Interface 4: DFU (Device Firmware Upgrade)
- Runtime-Modus, bcdDFUVersion 0x0110
- Kein Endpoint im Runtime-Modus

## Audio-Topologie (Audio Control, Interface 0)
```
[USB IN Terminal 2, 6ch] → [Feature Unit 10] → [Output Terminal 20: Speaker]
[Input Terminal 1: Mic, 12ch] → [Feature Unit 11] → [Extension Units / Routing]
                                                   → [Output Terminal 22: USB OUT, 12ch]
Clock: Entity 41 (intern) ┐
       Entity 44 (extern) ┤→ [Selector 40] → alle Terminals
Extension Units: proprietäre Routing-Matrix (Mixer, Monitor-Sends, etc.)
```

## Bewertung: Warum funktioniert snd-usb-audio nicht?

Das Gerät ist **vollständig UAC2-konform**. Die Mathematik stimmt exakt.
Das Problem liegt wahrscheinlich an einem oder mehreren dieser Punkte:

1. **Extension Units** — `snd-usb-audio` kennt die proprietären Extension-Unit-Typen nicht
   und bricht ggf. beim Parsen der Audio-Control-Topologie ab
2. **Clock-Setup** — der Treiber muss Clock Entity 41 auswählen und die Samplerate
   setzen, bevor Audio-Streaming startet; fehlt das, bleibt das Gerät stumm
3. **Alternate Setting** — Linux muss explizit auf AlternateSetting 1 umschalten
   (beide Streaming-Interfaces starten auf Alt 0 = zero-bandwidth)
4. **Asynchronous Feedback** auf Record-Endpoint — `bmAttributes: 0x25` (Implicit Feedback)
   erfordert korrekte Feedback-Endpoint-Behandlung

## Nächste Schritte
- [ ] Unter Linux testen: `dmesg` nach dem Einstecken → was erkennt snd-usb-audio?
- [ ] `aplay -l` und `arecord -l` zeigen, ob das Gerät erkannt wird
- [ ] `snd-usb-audio` quirk-flag für VID:PID 2708:0008 recherchieren
- [ ] Konfiguration 2 analysieren (zweite pcapng aufnehmen mit GET DESCRIPTOR CONFIGURATION index 1)
- [ ] Vendor-spezifische Control Requests während Betrieb aufzeichnen (Gain, Phantom Power)

## TODO: Noch zu analysieren
- [ ] Restliche Entities in Interface 0 (Output Terminal, Mixer/Selector Units)
- [ ] Interface 1: Audio Streaming (Playback)
- [ ] Interface 2: Audio Streaming (Record)
- [ ] Interface 3+4: HID / Vendor-spezifisch
- [ ] SET CONFIGURATION — welche Config wird gewählt?
- [ ] Endpoint-Descriptors (Adressen, Paketgrößen, Intervalle)
- [ ] Isochronous Transfer-Größen → Samplerate berechnen
- [ ] Vendor-spezifische Control Requests während Betrieb
