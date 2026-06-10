//! Vendor-Probe v5 fuer den optischen Eingang des iD14 MkII.
//!
//! Voraussetzung: optisches Kabel angeschlossen, S/PDIF-Quelle spielt aktiv.
//!
//! Gegenueber v4 behoben:
//!   - laengere Settle-Zeit + Bestaetigungsfenster (langsamer PLL-Lock wird erkannt),
//!   - Restore erst NACH dem Fenster (richtiger Wert wird nicht vorzeitig verworfen),
//!   - RESET-Erkennung: bei USB-Fehler wird das Geraet neu geoeffnet und der Lauf
//!     fortgesetzt; die Position wird laut geloggt (damit klar ist, ob/wann ein
//!     Reset mitten im Lauf passierte).
//!
//! Clock-Sources (41/44) und Feature Units (10/11) werden NIE beschrieben.

use std::thread::sleep;
use std::time::Duration;

use common::{Id14, CX_CLOCK_SELECTOR, ENTITY_CLOCK_SELECTOR};

const ENTITIES: &[u8] = &[62, 50, 51, 52, 54, 55, 60];
const CONTROL_SELECTORS: std::ops::RangeInclusive<u8> = 1..=20;
/// Realistische Format-Enum-Werte (ADAT/S/PDIF ist ein kleiner Schalter).
const VALUES: &[u8] = &[0x00, 0x01, 0x02, 0x03];

const SELECT_OPTICAL: u8 = 2;
const SELECT_INTERNAL: u8 = 1;

/// Bestaetigungsfenster: nach einem Write so lange auf Lock pruefen.
const CONFIRM_CHECKS: &[u64] = &[400, 800, 1300]; // ms-Marken (kumulativ gewartet)

struct Dev {
    inner: Id14,
}

impl Dev {
    fn open() -> Dev {
        loop {
            match Id14::open() {
                Ok(inner) => {
                    let d = Dev { inner };
                    d.select(SELECT_OPTICAL);
                    return d;
                }
                Err(_) => {
                    eprintln!("[reopen] warte auf Geraet...");
                    sleep(Duration::from_millis(800));
                }
            }
        }
    }

    fn select(&self, v: u8) {
        let _ = self
            .inner
            .set_cur(ENTITY_CLOCK_SELECTOR, CX_CLOCK_SELECTOR, 0, &[v]);
    }

    /// Validity lesen; Err signalisiert USB-Fehler (moeglicher Reset).
    fn valid(&self) -> rusb::Result<bool> {
        self.inner.optical_clock_valid()
    }
}

fn main() {
    let mut dev = Dev::open();
    println!("[start] Geraet offen, optical gewaehlt.");

    // Baseline + Reset-Erkennung scharf stellen.
    match dev.valid() {
        Ok(v) => println!("[start] baseline validity = {v}\n"),
        Err(e) => {
            println!("[start] baseline-Lesefehler {e} -> reopen");
            dev = Dev::open();
        }
    }

    let mut writes = 0u32;
    let mut resets = 0u32;

    for &entity in ENTITIES {
        for cs in CONTROL_SELECTORS {
            // Snapshot lesen; bei Fehler -> evtl. Reset.
            let snapshot = match dev.inner.get_cur(entity, cs, 0, 1) {
                Ok(v) if !v.is_empty() => v,
                Ok(_) => continue,
                Err(_) => {
                    // Kann "CS existiert nicht" ODER Reset sein. Re-check via valid().
                    if dev.valid().is_err() {
                        resets += 1;
                        println!("[RESET] bei E={entity} CS={cs} (#{resets}) -> reopen");
                        dev = Dev::open();
                    }
                    continue;
                }
            };
            println!("[probe] E={entity} CS={cs} wert=0x{:02x}", snapshot[0]);

            for &val in VALUES {
                if val == snapshot[0] {
                    continue;
                }
                if dev.inner.set_cur(entity, cs, 0, &[val]).is_err() {
                    continue;
                }
                writes += 1;
                dev.select(SELECT_OPTICAL);

                // Bestaetigungsfenster: bei jeder Marke pruefen.
                let mut waited = 0u64;
                let mut locked = false;
                let mut reset_here = false;
                for &mark in CONFIRM_CHECKS {
                    sleep(Duration::from_millis(mark - waited));
                    waited = mark;
                    match dev.valid() {
                        Ok(true) => {
                            locked = true;
                            break;
                        }
                        Ok(false) => {}
                        Err(_) => {
                            reset_here = true;
                            break;
                        }
                    }
                }

                if locked {
                    dev.select(SELECT_OPTICAL);
                    println!("\n=== TREFFER ===");
                    println!("SET_CUR Entity={entity} CS={cs} CN=0 Payload=0x{val:02x} -> S/PDIF-Lock");
                    println!("Vorheriger Wert: 0x{:02x}. {writes} Schreibvorgaenge, {resets} Resets.", snapshot[0]);
                    return;
                }

                if reset_here {
                    resets += 1;
                    println!("[RESET] beim Test E={entity} CS={cs} val=0x{val:02x} (#{resets}) -> reopen");
                    dev = Dev::open();
                    // snapshot nach Reset evtl. ungueltig -> nicht restaurieren.
                    continue;
                }

                // kein Lock -> Original zuruecksetzen.
                let _ = dev.inner.set_cur(entity, cs, 0, &snapshot);
            }
        }
    }

    dev.select(SELECT_INTERNAL);
    println!("\nKein Treffer. {writes} Schreibvorgaenge, {resets} Resets erkannt.");
    if resets > 0 {
        println!("ACHTUNG: {resets} Resets — Teile des Laufs liefen evtl. auf neuem Handle, aber lueckenlos fortgesetzt.");
    }
}
