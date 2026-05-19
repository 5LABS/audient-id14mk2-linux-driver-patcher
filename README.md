# Audient iD14 mk2 — Setup-Anleitung (Ubuntu 26.04 LTS)

## Voraussetzungen

- Ubuntu 26.04 LTS, Linux Kernel 7+
- Secure Boot kann aktiv bleiben
- PipeWire + WirePlumber (in Ubuntu 26.04 Standard)
- iD14 per USB angeschlossen

---

## Schritt 1 — Gerät prüfen

Nach dem Einstecken des iD14 prüfen ob es erkannt wird:

```bash
cat /proc/asound/cards
```

Erwartete Ausgabe (Kartennummer kann abweichen):

```
 0 [PCH            ]: HDA-Intel - ...
 1 [iD14           ]: USB-Audio - Audient iD14
```

Falls das iD14 fehlt: USB-Kabel und USB-Port wechseln, dann `dmesg | grep -i "2708\|audient"` prüfen.

---

## Schritt 2 — UCM-Datei patchen

Die mitgelieferte UCM-Konfiguration von Ubuntu hat die Kanal-Zuordnungen vertauscht. Die korrigierte Datei aus diesem Repository einspielen:

```bash
sudo cp Audient-iD14-HiFi-0008.conf \
     /usr/share/alsa/ucm2/USB-Audio/Audient/Audient-iD14-HiFi-0008.conf
```

**Was wurde geändert:** In der Original-Datei zeigt `Line1` ("Monitor Output 1-2") auf USB-Kanäle 2+3 (stumm ohne proprietäre Treiber-Init), `Headphones` auf Kanäle 0+1 (physisch aktiv). Die korrigierte Datei tauscht diese Zuordnung und setzt `Line1` als Default (Priority 200).

---

## Schritt 3 — WirePlumber neu starten

```bash
systemctl --user restart wireplumber
```

Danach prüfen ob `Line__sink` als Default-Filter aktiv ist (Sternchen vor der Zeile):

```bash
wpctl status | grep -A5 "Filters"
```

Erwartung: `*  ... HiFi__Line__sink`

---

## Schritt 4 — ALSA Speaker-Pegel auf 0 dB setzen

```bash
amixer -c iD14 sset Speaker 127
```

Ohne diesen Schritt ist der Ausgang auf -20 dB gedrosselt (~10% Lautstärke).

Einstellung dauerhaft speichern (einmalig):

```bash
sudo alsactl store
```

Ab sofort wird der Wert bei jedem Boot automatisch wiederhergestellt.

---

## Schritt 5 — PipeWire-Volume setzen

```bash
wpctl set-volume @DEFAULT_AUDIO_SINK@ 1.0
```

Dieser Wert wird von WirePlumber im Benutzerprofil gespeichert und bleibt erhalten.

---

## Schritt 6 — Test Playback

```bash
speaker-test -c 2 -t sine -l 1 --format=S32_LE
```

Erwartung: FL und FR am Monitor-Out gleichlaut, kein Fehler.

---

## Schritt 7 — Test Aufnahme

```bash
arecord -f S32_LE -r 48000 -c 2 -d 5 /tmp/test.wav && aplay /tmp/test.wav
```

5 Sekunden aufnehmen, direkt abspielen. Erwartung: Stimme klar im Monitor-Out.

---

## Zusammenfassung aller Befehle

```bash
# 1. Datei einspielen
sudo cp Audient-iD14-HiFi-0008.conf \
     /usr/share/alsa/ucm2/USB-Audio/Audient/Audient-iD14-HiFi-0008.conf

# 2. WirePlumber neu starten
systemctl --user restart wireplumber

# 3. Pegel setzen
amixer -c iD14 sset Speaker 127
sudo alsactl store
wpctl set-volume @DEFAULT_AUDIO_SINK@ 1.0

# 4. Testen
speaker-test -c 2 -t sine -l 1 --format=S32_LE
```

---

## Bekannte Einschränkungen

- **USB-Kanäle 2–5 sind stumm** — das iD14 benötigt proprietäre USB Control Requests (Extension Units) um die internen Routing-Kanäle für Headphone-Out, Cue usw. zu aktivieren. Unter Linux passiert das nicht automatisch. Betrifft: zweiter Kopfhörer-Ausgang, DAW-Sends.
- **Direkte ALSA/JACK-Nutzung (hw:CARD=iD14)** — umgeht UCM und PipeWire. Kanal-Asymmetrie (FR leiser) tritt auf, da Extension Units nicht initialisiert sind.
- **Package-Update** — `sudo apt upgrade` kann die UCM-Datei unter `/usr/share/alsa/ucm2/` überschreiben. Nach einem Update `dpkg -l alsa-ucm-conf` prüfen, ob eine neue Version installiert wurde, und Schritt 2 wiederholen.

---

## Datei-Referenz

| Datei | Beschreibung |
|---|---|
| `Audient-iD14-HiFi-0008.conf` | Gepatchte UCM-Konfiguration (dieses Repository) |
| `/usr/share/alsa/ucm2/USB-Audio/Audient/Audient-iD14-HiFi-0008.conf` | Ziel-Pfad im System |
| `/var/lib/alsa/asound.state` | ALSA Mixer-Zustand (gespeichert durch alsactl store) |
