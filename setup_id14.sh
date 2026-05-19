#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UCM_TARGET="/usr/share/alsa/ucm2/USB-Audio/Audient/Audient-iD14-HiFi-0008.conf"
UCM_SOURCE="$SCRIPT_DIR/Audient-iD14-HiFi-0008.conf"

echo "=== Audient iD14 mk2 Setup ==="
echo ""

# Gerät prüfen
if ! aplay -l 2>/dev/null | grep -q "iD14"; then
    echo "FEHLER: Audient iD14 nicht gefunden. USB-Verbindung prüfen."
    exit 1
fi
echo "[1/5] Gerät erkannt."

# UCM-Datei prüfen
if [ ! -f "$UCM_SOURCE" ]; then
    echo "FEHLER: $UCM_SOURCE nicht gefunden. Script muss im Projektordner liegen."
    exit 1
fi

# UCM-Datei einspielen
sudo cp "$UCM_SOURCE" "$UCM_TARGET"
echo "[2/5] UCM-Datei eingespielt."

# WirePlumber neu starten
systemctl --user restart wireplumber
sleep 2
echo "[3/5] WirePlumber neugestartet."

# ALSA Speaker-Pegel auf 0 dB
amixer -c iD14 sset Speaker 127 > /dev/null
sudo alsactl store
echo "[4/5] ALSA Speaker-Pegel auf 0 dB gesetzt und gespeichert."

# PipeWire-Volume
wpctl set-volume @DEFAULT_AUDIO_SINK@ 1.0
echo "[5/5] PipeWire-Volume auf 100% gesetzt."

echo ""
echo "=== Setup abgeschlossen ==="
echo ""
echo "Test starten? [j/N]"
read -r ANTWORT
if [[ "$ANTWORT" =~ ^[jJ]$ ]]; then
    echo "Playback-Test (FL + FR, 5 Sekunden Sinuston)..."
    speaker-test -c 2 -t sine -l 1 --format=S32_LE
    echo ""
    echo "Aufnahme-Test (5 Sekunden — bitte sprechen)..."
    arecord -f S32_LE -r 48000 -c 2 -d 5 /tmp/id14_test.wav
    echo "Wiedergabe..."
    aplay /tmp/id14_test.wav
    rm /tmp/id14_test.wav
fi
