//! Gemeinsamer libusb-Layer fuer die Audient iD14 MkII Vendor-Controls.
//!
//! Das Geraet ist UAC2-class-compliant, versteckt Routing/Format aber hinter
//! proprietaeren Extension Units. Diese werden ueber UAC2-Control-Requests
//! (SET_CUR / GET_CUR) an die jeweilige Entity adressiert.

use std::time::Duration;

use rusb::{Context, DeviceHandle, UsbContext};

pub const VID: u16 = 0x2708;
pub const PID: u16 = 0x0008;

/// AudioControl-Interface (Interface 0) — Recipient aller Entity-Control-Requests.
pub const AC_INTERFACE: u16 = 0;

/// Clock-Source-Entity "Audient Optical1 Clock" (externer Takt).
pub const ENTITY_OPTICAL_CLOCK: u8 = 44;
/// Clock-Selector-Entity.
pub const ENTITY_CLOCK_SELECTOR: u8 = 40;

/// UAC2 Control Selectors.
pub const CS_SAM_FREQ: u8 = 0x01; // Clock Source: Sampling Frequency
pub const CS_CLOCK_VALID: u8 = 0x02; // Clock Source: Clock Validity
pub const CX_CLOCK_SELECTOR: u8 = 0x01; // Clock Selector: Selector Control

/// UAC2 bRequest.
pub const REQ_CUR: u8 = 0x01; // SET_CUR / GET_CUR (Wert via Richtung im bmRequestType)

/// bmRequestType: Host->Device, Class, Interface.
pub const RT_SET: u8 = 0x21;
/// bmRequestType: Device->Host, Class, Interface.
pub const RT_GET: u8 = 0xA1;

const TIMEOUT: Duration = Duration::from_millis(500);

pub struct Id14 {
    handle: DeviceHandle<Context>,
    claimed: bool,
}

impl Id14 {
    /// Oeffnet das erste angeschlossene iD14 MkII und beansprucht das
    /// AudioControl-Interface. snd-usb-audio wird dafuer kurz vom Interface 0
    /// geloest (auto-detach) und beim Schliessen wieder angehaengt. Waehrend
    /// dieser Zeit ist die Audiowiedergabe ueber das Geraet unterbrochen.
    pub fn open() -> rusb::Result<Self> {
        let ctx = Context::new()?;
        for dev in ctx.devices()?.iter() {
            let desc = dev.device_descriptor()?;
            if desc.vendor_id() == VID && desc.product_id() == PID {
                let mut handle = dev.open()?;
                // Kernel-Treiber bei claim automatisch loesen und bei release
                // wieder anhaengen. Nicht alle Plattformen unterstuetzen das.
                let _ = handle.set_auto_detach_kernel_driver(true);
                let claimed = handle.claim_interface(AC_INTERFACE as u8).is_ok();
                return Ok(Self { handle, claimed });
            }
        }
        Err(rusb::Error::NoDevice)
    }

    fn windex(entity: u8) -> u16 {
        ((entity as u16) << 8) | AC_INTERFACE
    }

    fn wvalue(cs: u8, cn: u8) -> u16 {
        ((cs as u16) << 8) | (cn as u16)
    }

    /// GET_CUR auf (entity, cs, cn). Liest `len` Bytes.
    pub fn get_cur(&self, entity: u8, cs: u8, cn: u8, len: usize) -> rusb::Result<Vec<u8>> {
        let mut buf = vec![0u8; len];
        let n = self.handle.read_control(
            RT_GET,
            REQ_CUR,
            Self::wvalue(cs, cn),
            Self::windex(entity),
            &mut buf,
            TIMEOUT,
        )?;
        buf.truncate(n);
        Ok(buf)
    }

    /// SET_CUR auf (entity, cs, cn) mit `data`.
    pub fn set_cur(&self, entity: u8, cs: u8, cn: u8, data: &[u8]) -> rusb::Result<()> {
        let n = self.handle.write_control(
            RT_SET,
            REQ_CUR,
            Self::wvalue(cs, cn),
            Self::windex(entity),
            data,
            TIMEOUT,
        )?;
        if n != data.len() {
            return Err(rusb::Error::Io);
        }
        Ok(())
    }

    /// Liest die Clock-Validity der optischen Quelle (Entity 44). true = Lock.
    pub fn optical_clock_valid(&self) -> rusb::Result<bool> {
        let v = self.get_cur(ENTITY_OPTICAL_CLOCK, CS_CLOCK_VALID, 0, 1)?;
        Ok(v.first().copied().unwrap_or(0) != 0)
    }
}

impl Drop for Id14 {
    fn drop(&mut self) {
        if self.claimed {
            // Gibt das Interface frei; durch auto-detach wird snd-usb-audio
            // anschliessend wieder angehaengt.
            let _ = self.handle.release_interface(AC_INTERFACE as u8);
        }
    }
}
