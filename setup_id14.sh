#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UCM_TARGET="/usr/share/alsa/ucm2/USB-Audio/Audient/Audient-iD14-HiFi-0008.conf"
UCM_SOURCE="$SCRIPT_DIR/Audient-iD14-HiFi-0008.conf"

# Beim Ausführen als root den echten Desktop-User ermitteln
if [ "$(id -u)" = "0" ]; then
    REAL_USER="${SUDO_USER:-$(logname 2>/dev/null)}"
    if [ -z "$REAL_USER" ] || [ "$REAL_USER" = "root" ]; then
        # Fallback: User mit aktiver /run/user/<uid>-Session suchen
        REAL_USER=$(ls /run/user/ | grep -v '^0$' | head -1 | xargs -I{} id -un {} 2>/dev/null)
    fi
    if [ -z "$REAL_USER" ] || [ "$REAL_USER" = "root" ]; then
        echo "FEHLER: Echter Benutzer nicht ermittelbar. Script als normaler User ausführen oder via sudo."
        exit 1
    fi
else
    REAL_USER="$USER"
fi
REAL_UID=$(id -u "$REAL_USER")
USER_SYSTEMCTL="sudo -u $REAL_USER XDG_RUNTIME_DIR=/run/user/$REAL_UID DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/$REAL_UID/bus systemctl --user"
USER_RUN="sudo -u $REAL_USER XDG_RUNTIME_DIR=/run/user/$REAL_UID DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/$REAL_UID/bus"

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

# UCM-Datei einspielen (braucht root)
cp "$UCM_SOURCE" "$UCM_TARGET"
echo "[2/5] UCM-Datei eingespielt."

# PipeWire + WirePlumber in der User-Session neu starten
$USER_SYSTEMCTL restart pipewire.service pipewire-pulse.service
sleep 2
$USER_SYSTEMCTL restart wireplumber
sleep 2
echo "[3/5] PipeWire + WirePlumber neugestartet (User: $REAL_USER)."

# ALSA Speaker-Pegel auf 0 dB
amixer -c iD14 sset Speaker 127 > /dev/null
alsactl store
echo "[4/5] ALSA Speaker-Pegel auf 0 dB gesetzt und gespeichert."

# PipeWire-Volume
$USER_RUN wpctl set-volume @DEFAULT_AUDIO_SINK@ 1.0
echo "[5/5] PipeWire-Volume auf 100% gesetzt."

echo ""
echo "=== Setup abgeschlossen ==="
echo ""
echo "Test starten? [j/N]"
read -r ANTWORT
if [[ "$ANTWORT" =~ ^[jJ]$ ]]; then
    echo "Playback-Test (FL + FR, 5 Sekunden Sinuston)..."
    $USER_RUN speaker-test -c 2 -t sine -l 1 --format=S32_LE
    echo ""
    echo "Aufnahme-Test (5 Sekunden — bitte sprechen)..."
    $USER_RUN arecord -f S32_LE -r 48000 -c 2 -d 5 /tmp/id14_test.wav
    echo "Wiedergabe..."
    $USER_RUN aplay /tmp/id14_test.wav
    rm /tmp/id14_test.wav
fi
