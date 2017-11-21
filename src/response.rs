//! Parses I2C responses from the EC EZO Chip.
//!
//! Code modified from "Federico Mena Quintero <federico@gnome.org>"'s original.
use std::fmt;
use std::str::FromStr;

use errors::*;

/// Response for commands that may or may not expect ACK.
#[derive(Clone, Debug, PartialEq)]
pub enum ResponseStatus {
    Ack,
    None,
}

impl fmt::Display for ResponseStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Current firmware settings of the EZO chip.
#[derive(Clone, PartialEq)]
pub struct DeviceInfo {
    pub device: String,
    pub firmware: String,
}

impl DeviceInfo {
    pub fn parse(response: &str) -> Result<DeviceInfo> {
        if response.starts_with("?I,") {
            let rest = response.get(3..).unwrap();
            let mut split = rest.split(',');

            let device = if let Some(device_str) = split.next() {
                device_str.to_string()
            } else {
                return Err(ErrorKind::ResponseParse.into());
            };

            let firmware = if let Some(firmware_str) = split.next() {
                firmware_str.to_string()
            } else {
                return Err(ErrorKind::ResponseParse.into());
            };

            if let Some(_) = split.next() {
                return Err(ErrorKind::ResponseParse.into());
            }

            if firmware.len() == 0 || device.len() == 0 {
                return Err(ErrorKind::ResponseParse.into());
            }

            Ok (DeviceInfo { device, firmware } )

        } else {
            Err(ErrorKind::ResponseParse.into())
        }
    }
}

impl fmt::Debug for DeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "?I,{},{}", self.device, self.firmware)
    }
}

impl fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{}", self.device, self.firmware)
    }
}

/// Reason for which the device restarted, data sheet pp. 58
#[derive(Copy, Clone, PartialEq)]
pub enum RestartReason {
    PoweredOff,
    SoftwareReset,
    BrownOut,
    Watchdog,
    Unknown,
}

impl fmt::Debug for RestartReason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RestartReason::PoweredOff => write!(f, "P"),
            RestartReason::SoftwareReset => write!(f, "S"),
            RestartReason::BrownOut => write!(f, "B"),
            RestartReason::Watchdog => write!(f, "W"),
            RestartReason::Unknown => write!(f, "U"),
        }
    }
}

impl fmt::Display for RestartReason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RestartReason::PoweredOff => write!(f, "powered-off"),
            RestartReason::SoftwareReset => write!(f, "software-reset"),
            RestartReason::BrownOut => write!(f, "brown-out"),
            RestartReason::Watchdog => write!(f, "watchdog"),
            RestartReason::Unknown => write!(f, "unknown"),
        }
    }
}

/// Response from the "Status" command to get the device status
#[derive(Copy, Clone, PartialEq)]
pub struct DeviceStatus {
    pub restart_reason: RestartReason,
    pub vcc_voltage: f64,
}

impl DeviceStatus {
    /// Parses the result of the "Status" command to get the device's status.
    pub fn parse(response: &str) -> Result<DeviceStatus> {
        if response.starts_with("?STATUS,") {
            let rest = response.get(8..).unwrap();
            let mut split = rest.split(',');

            let restart_reason = match split.next() {
                Some("P") => RestartReason::PoweredOff,
                Some("S") => RestartReason::SoftwareReset,
                Some("B") => RestartReason::BrownOut,
                Some("W") => RestartReason::Watchdog,
                Some("U") => RestartReason::Unknown,
                _ => return Err(ErrorKind::ResponseParse.into()),
            };

            let voltage = if let Some(voltage_str) = split.next() {
                f64::from_str(voltage_str)
                    .chain_err(|| ErrorKind::ResponseParse)?
            } else {
                return Err(ErrorKind::ResponseParse.into());
            };

            if let Some(_) = split.next() {
                return Err(ErrorKind::ResponseParse.into());
            }

            Ok(DeviceStatus {
                   restart_reason: restart_reason,
                   vcc_voltage: voltage,
               })
        } else {
            Err(ErrorKind::ResponseParse.into())
        }
    }
}

impl fmt::Debug for DeviceStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "?STATUS,{:?},{:.*}", self.restart_reason, 3, self.vcc_voltage)
    }
}

impl fmt::Display for DeviceStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{:.*}", self.restart_reason, 3, self.vcc_voltage)
    }
}

/// Exported calibration string of the EC EZO chip.
#[derive(Clone, PartialEq)]
pub enum Exported {
    ExportString(String),
    Done,
}

impl Exported {
    pub fn parse(response: &str) -> Result<Exported> {
        if response.starts_with("*") {
            match response {
                "*DONE" => Ok(Exported::Done),
                _ => Err(ErrorKind::ResponseParse.into()),
            }
        } else {
            match response.len() {
                1..13 => Ok(Exported::ExportString(response.to_string())),
                _ => Err(ErrorKind::ResponseParse.into()),
            }
        }
    }
}

impl fmt::Debug for Exported {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Exported::ExportString(ref s) => write!(f, "{}", s),
            &Exported::Done => write!(f, "*DONE"),
        }
    }
}

impl fmt::Display for Exported {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Exported::ExportString(ref s) => write!(f, "{}", s),
            &Exported::Done => write!(f, "DONE"),
        }
    }
}

/// Export the current calibration settings of the EC EZO chip.
#[derive(Copy, Clone, PartialEq)]
pub struct ExportedInfo {
    pub lines: u16,
    pub total_bytes: u16,
}

impl ExportedInfo {
    pub fn parse(response: &str) -> Result<ExportedInfo> {
        if response.starts_with("?EXPORT,") {
            let num_str = response.get(8..).unwrap();

            let mut split = num_str.split(",");

            let lines = if let Some(lines_str) = split.next() {
                u16::from_str(lines_str)
                    .chain_err(|| ErrorKind::ResponseParse)?
            } else {
                return Err(ErrorKind::ResponseParse.into());
            };

            let total_bytes = if let Some(totalbytes_str) = split.next() {
                u16::from_str(totalbytes_str)
                    .chain_err(|| ErrorKind::ResponseParse)?
            } else {
                return Err(ErrorKind::ResponseParse.into());
            };

            if let Some(_) = split.next() {
                return Err(ErrorKind::ResponseParse.into());
            }

            Ok (ExportedInfo { lines, total_bytes } )
        } else {
            Err(ErrorKind::ResponseParse.into())
        }
    }
}

impl fmt::Debug for ExportedInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "?EXPORT,{},{}", self.lines, self.total_bytes)
    }
}

impl fmt::Display for ExportedInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{}", self.lines, self.total_bytes)
    }
}

/// Status of I2C protocol lock.
#[derive(Copy, Clone, PartialEq)]
pub enum ProtocolLockStatus {
    Off,
    On,
}

impl ProtocolLockStatus {
    pub fn parse(response: &str) -> Result<ProtocolLockStatus> {
        if response.starts_with("?PLOCK,") {
            let rest = response.get(7..).unwrap();
            let mut split = rest.split(',');

            let _plock_status = match split.next() {
                Some("1") => Ok(ProtocolLockStatus::On),
                Some("0") => Ok(ProtocolLockStatus::Off),
                _ => return Err(ErrorKind::ResponseParse.into()),
            };

            match split.next() {
                None => _plock_status,
                _ => Err(ErrorKind::ResponseParse.into()),
            }
        } else {
            Err(ErrorKind::ResponseParse.into())
        }
    }
}

impl fmt::Debug for ProtocolLockStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ProtocolLockStatus::On => write!(f, "?PLOCK,1"),
            ProtocolLockStatus::Off => write!(f, "?PLOCK,0"),
        }
    }
}

impl fmt::Display for ProtocolLockStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ProtocolLockStatus::On => write!(f, "on"),
            ProtocolLockStatus::Off => write!(f, "off"),
        }
    }
}

/// Status of EZO's LED.
#[derive(Copy, Clone, PartialEq)]
pub enum LedStatus {
    Off,
    On,
}

impl LedStatus {
    pub fn parse(response: &str) -> Result<LedStatus> {
        if response.starts_with("?L,") {
            let rest = response.get(3..).unwrap();

            match rest {
                "1" => Ok(LedStatus::On),
                "0" => Ok(LedStatus::Off),
                _ => return Err(ErrorKind::ResponseParse.into()),
            }
        } else {
            Err(ErrorKind::ResponseParse.into())
        }
    }
}

impl fmt::Debug for LedStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LedStatus::On => write!(f, "?L,1"),
            LedStatus::Off => write!(f, "?L,0"),
        }
    }
}

impl fmt::Display for LedStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LedStatus::On => write!(f, "on"),
            LedStatus::Off => write!(f, "off"),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_response_to_device_status() {
        let response = "?STATUS,P,1.5";
        assert_eq!(DeviceStatus::parse(response).unwrap(),
                   DeviceStatus {
                       restart_reason: RestartReason::PoweredOff,
                       vcc_voltage: 1.5,
                   });

        let response = "?STATUS,S,1.5";
        assert_eq!(DeviceStatus::parse(response).unwrap(),
                   DeviceStatus {
                       restart_reason: RestartReason::SoftwareReset,
                       vcc_voltage: 1.5,
                   });

        let response = "?STATUS,B,1.5";
        assert_eq!(DeviceStatus::parse(response).unwrap(),
                   DeviceStatus {
                       restart_reason: RestartReason::BrownOut,
                       vcc_voltage: 1.5,
                   });

        let response = "?STATUS,W,1.5";
        assert_eq!(DeviceStatus::parse(response).unwrap(),
                   DeviceStatus {
                       restart_reason: RestartReason::Watchdog,
                       vcc_voltage: 1.5,
                   });

        let response = "?STATUS,U,1.5";
        let device_status = DeviceStatus {
            restart_reason: RestartReason::Unknown,
            vcc_voltage: 1.5,
        };
        assert_eq!(DeviceStatus::parse(response).unwrap(), device_status);
    }

    #[test]
    fn parses_device_status_to_response() {
        let device_status = DeviceStatus {
            restart_reason: RestartReason::Unknown,
            vcc_voltage: 3.15,
        };
        assert_eq!(format!("{}", device_status), "unknown,3.150");
    }

    #[test]
    fn parsing_invalid_device_status_yields_error() {
        let response = "";
        assert!(DeviceStatus::parse(response).is_err());

        let response = "?STATUS,X,";
        assert!(DeviceStatus::parse(response).is_err());

        let response = "?Status,P,1.5,";
        assert!(DeviceStatus::parse(response).is_err());
    }

    #[test]
    fn parses_response_to_device_information() {
        let response = "?I,RTD,2.01";
        assert_eq!(DeviceInfo::parse(response).unwrap(),
                   DeviceInfo {
                       device: "RTD".to_string(),
                       firmware: "2.01".to_string(),
                   } );

        let response = "?I,RTD,1.98";
        assert_eq!(DeviceInfo::parse(response).unwrap(),
                   DeviceInfo {
                       device: "RTD".to_string(),
                       firmware: "1.98".to_string(),
                   } );
    }

    #[test]
    fn parses_device_information_to_response() {
        let device_info = DeviceInfo {
            device: "RTD".to_string(),
            firmware: "2.01".to_string(),
        };
        assert_eq!(format!("{}", device_info), "RTD,2.01");

        let device_info = DeviceInfo {
            device: "RTD".to_string(),
            firmware: "1.98".to_string(),
        };
        assert_eq!(format!("{}", device_info), "RTD,1.98");
    }

    #[test]
    fn parsing_invalid_device_info_yields_error() {
        let response = "";
        assert!(DeviceInfo::parse(response).is_err());

        let response = "?I";
        assert!(DeviceInfo::parse(response).is_err());

        let response = "?I,";
        assert!(DeviceInfo::parse(response).is_err());

        let response = "?I,,";
        assert!(DeviceInfo::parse(response).is_err());

        let response = "?I,a,b,c";
        assert!(DeviceInfo::parse(response).is_err());
    }

    #[test]
    fn parses_response_to_export_info() {
        let response = "?EXPORT,0,0";
        assert_eq!(ExportedInfo::parse(response).unwrap(),
                   ExportedInfo { lines: 0, total_bytes: 0 } );

        let response = "?EXPORT,10,120";
        assert_eq!(ExportedInfo::parse(response).unwrap(),
                   ExportedInfo { lines: 10, total_bytes: 120 } );
    }

    #[test]
    fn parses_export_info_to_response() {
        let export_info = ExportedInfo { lines: 0, total_bytes: 0 };
        assert_eq!(format!("{}", export_info), "0,0");

        let export_info = ExportedInfo { lines: 10, total_bytes: 120 };
        assert_eq!(format!("{}", export_info), "10,120");
    }

    #[test]
    fn parsing_invalid_export_info_yields_error() {
        let response = "?EXPORT,11,120,10";
        assert!(ExportedInfo::parse(response).is_err());

        let response = "?EXPORT,1012";
        assert!(ExportedInfo::parse(response).is_err());

        let response = "10,*DON";
        assert!(ExportedInfo::parse(response).is_err());

        let response = "12,";
        assert!(ExportedInfo::parse(response).is_err());

        let response = "";
        assert!(ExportedInfo::parse(response).is_err());
    }

    #[test]
    fn parses_response_to_data_export_string() {
        let response = "0";
        assert_eq!(Exported::parse(response).unwrap(),
                   Exported::ExportString("0".to_string()));

        let response = "012abc";
        assert_eq!(Exported::parse(response).unwrap(),
                   Exported::ExportString("012abc".to_string()));

        let response = "123456abcdef";
        assert_eq!(Exported::parse(response).unwrap(),
                   Exported::ExportString("123456abcdef".to_string()));

        let response = "*DONE";
        assert_eq!(Exported::parse(response).unwrap(),
                   Exported::Done);
    }

    #[test]
    fn parses_data_export_string_to_response() {
        let exported = Exported::ExportString("0".to_string());
        assert_eq!(format!("{}", exported), "0");

        let exported = Exported::ExportString("012abc".to_string());
        assert_eq!(format!("{}", exported), "012abc");

        let exported = Exported::ExportString("123456abcdef".to_string());
        assert_eq!(format!("{}", exported), "123456abcdef");

        let exported = Exported::ExportString("*DONE".to_string());
        assert_eq!(format!("{}", exported), "*DONE");
    }

    #[test]
    fn parsing_invalid_export_string_yields_error() {
        let response = "*";
        assert!(Exported::parse(response).is_err());

        let response = "*DONE*";
        assert!(Exported::parse(response).is_err());

        let response = "**DONE";
        assert!(Exported::parse(response).is_err());

        let response = "12345678901234567890";
        assert!(Exported::parse(response).is_err());
    }

    #[test]
    fn parses_response_to_led_status() {
        let response = "?L,1";
        assert_eq!(LedStatus::parse(&response).unwrap(),
                   LedStatus::On);

        let response = "?L,0";
        assert_eq!(LedStatus::parse(&response).unwrap(),
                   LedStatus::Off);
    }

    #[test]
    fn parses_led_status_to_response() {
        let led = LedStatus::On;
        assert_eq!(format!("{}", led), "on");

        let led = LedStatus::Off;
        assert_eq!(format!("{}", led), "off");
    }

    #[test]
    fn parsing_invalid_led_status_yields_error() {
        let response = "";
        assert!(LedStatus::parse(&response).is_err());

        let response = "?L,";
        assert!(LedStatus::parse(&response).is_err());

        let response = "?L,b";
        assert!(LedStatus::parse(&response).is_err());

        let response = "?L,17";
        assert!(LedStatus::parse(&response).is_err());
    }

    #[test]
    fn parses_response_to_protocol_lock_status() {
        let response = "?PLOCK,1";
        assert_eq!(ProtocolLockStatus::parse(&response).unwrap(),
                   ProtocolLockStatus::On);

        let response = "?PLOCK,0";
        assert_eq!(ProtocolLockStatus::parse(&response).unwrap(),
                   ProtocolLockStatus::Off);
    }

    #[test]
    fn parses_protocol_lock_status_to_response() {
        let plock = ProtocolLockStatus::On;
        assert_eq!(format!("{}", plock), "on");

        let plock = ProtocolLockStatus::Off;
        assert_eq!(format!("{}", plock), "off");
    }

    #[test]
    fn parsing_invalid_protocol_lock_status_yields_error() {
        let response = "";
        assert!(ProtocolLockStatus::parse(&response).is_err());

        let response = "?PLOCK,57";
        assert!(ProtocolLockStatus::parse(&response).is_err());

        let response = "?PLOCK,b";
        assert!(ProtocolLockStatus::parse(&response).is_err());

        let response = "?PLOCK,b,1";
        assert!(ProtocolLockStatus::parse(&response).is_err());
    }
}
