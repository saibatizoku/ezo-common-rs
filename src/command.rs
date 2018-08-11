//! Commands common to EZO chips
use std::str::FromStr;
use std::thread;
use std::time::Duration;

use super::errors::{ErrorKind, EzoError};
use super::response::*;
use super::{
    response_code, string_from_response_data, write_to_ezo, BpsRate, Command, ResponseCode,
};

use failure::ResultExt;
use i2cdev::core::I2CDevice;
use i2cdev::linux::LinuxI2CDevice;

/// Maximum ascii-character response size + 2
pub const MAX_DATA: usize = 401;

define_command! {
    doc: "`Baud,n` command, where `n` is a variant belonging to `BpsRate`. Switch chip to UART mode.",
    cmd: Baud(BpsRate), { format!("BAUD,{}", cmd.parse()) }, 0
}

impl FromStr for Baud {
    type Err = EzoError;

    fn from_str(s: &str) -> Result<Self, EzoError> {
        let supper = s.to_uppercase();
        if supper.starts_with("BAUD,") {
            let rest = supper.get(5..).ok_or(ErrorKind::CommandParse)?;
            let mut split = rest.split(',');
            let rate = match split.next() {
                Some("300") => BpsRate::Bps300,
                Some("1200") => BpsRate::Bps1200,
                Some("2400") => BpsRate::Bps2400,
                Some("9600") => BpsRate::Bps9600,
                Some("19200") => BpsRate::Bps19200,
                Some("38400") => BpsRate::Bps38400,
                Some("57600") => BpsRate::Bps57600,
                Some("115200") => BpsRate::Bps115200,
                _ => return Err(ErrorKind::BpsRateParse)?,
            };
            match split.next() {
                None => return Ok(Baud(rate)),
                _ => return Err(ErrorKind::BaudParse)?,
            }
        }
        Err(ErrorKind::BaudParse)?
    }
}

define_command! {
    doc: "`CAL,CLEAR` command. Clears current calibration.",
    CalibrationClear, { "CAL,CLEAR".to_string() }, 300, Ack
}

impl FromStr for CalibrationClear {
    type Err = EzoError;

    fn from_str(s: &str) -> Result<Self, EzoError> {
        let supper = s.to_uppercase();
        match supper.as_ref() {
            "CAL,CLEAR" => Ok(CalibrationClear),
            _ => Err(ErrorKind::CommandParse)?,
        }
    }
}

define_command! {
    doc: "`I2C,n` command, where `n` is of type `u16`. Chance I2C address.",
    cmd: DeviceAddress(u16), { format!("I2C,{}", cmd) }, 300
}

impl FromStr for DeviceAddress {
    type Err = EzoError;

    fn from_str(s: &str) -> Result<Self, EzoError> {
        let supper = s.to_uppercase();
        if supper.starts_with("I2C,") {
            let rest = supper.get(4..).ok_or(ErrorKind::CommandParse)?;
            let mut split = rest.split(',');
            let value = match split.next() {
                Some(n) => n.parse::<u16>().context(ErrorKind::CommandParse)?,
                _ => return Err(ErrorKind::CommandParse)?,
            };
            match split.next() {
                None => return Ok(DeviceAddress(value)),
                _ => return Err(ErrorKind::CommandParse)?,
            }
        } else {
            Err(ErrorKind::CommandParse)?
        }
    }
}

define_command! {
    doc: "`I` command. Returns a `DeviceInfo` response. Device information.",
    DeviceInformation, { "I".to_string() }, 300,
    resp: DeviceInfo, { DeviceInfo::parse(&resp) }
}

impl FromStr for DeviceInformation {
    type Err = EzoError;

    fn from_str(s: &str) -> Result<Self, EzoError> {
        let supper = s.to_uppercase();
        match supper.as_ref() {
            "I" => Ok(DeviceInformation),
            _ => Err(ErrorKind::CommandParse)?,
        }
    }
}

define_command! {
    doc: "`EXPORT` command. Returns an `Exported` response. Exports current calibration.",
    Export, { "EXPORT".to_string() }, 300,
    resp: Exported, { Exported::parse(&resp) }
}

impl FromStr for Export {
    type Err = EzoError;

    fn from_str(s: &str) -> Result<Self, EzoError> {
        let supper = s.to_uppercase();
        match supper.as_ref() {
            "EXPORT" => Ok(Export),
            _ => Err(ErrorKind::CommandParse)?,
        }
    }
}

define_command! {
    doc: "`EXPORT,?` command. Returns an `ExportedInfo` response. Calibration string info.",
    ExportInfo, { "EXPORT,?".to_string() }, 300,
    resp: ExportedInfo, { ExportedInfo::parse(&resp) }
}

impl FromStr for ExportInfo {
    type Err = EzoError;

    fn from_str(s: &str) -> Result<Self, EzoError> {
        let supper = s.to_uppercase();
        match supper.as_ref() {
            "EXPORT,?" => Ok(ExportInfo),
            _ => Err(ErrorKind::CommandParse)?,
        }
    }
}

define_command! {
    doc: "`FACTORY` command. Enable factory reset.",
    Factory, { "FACTORY".to_string() }, 0
}

impl FromStr for Factory {
    type Err = EzoError;

    fn from_str(s: &str) -> Result<Self, EzoError> {
        let supper = s.to_uppercase();
        match supper.as_ref() {
            "FACTORY" => Ok(Factory),
            _ => Err(ErrorKind::CommandParse)?,
        }
    }
}

define_command! {
    doc: "`FIND` command. Find device with blinking white LED.",
    Find, { "F".to_string() }, 300
}

impl FromStr for Find {
    type Err = EzoError;

    fn from_str(s: &str) -> Result<Self, EzoError> {
        let supper = s.to_uppercase();
        match supper.as_ref() {
            "F" => Ok(Find),
            _ => Err(ErrorKind::CommandParse)?,
        }
    }
}

define_command! {
    doc: "`IMPORT,n` command, where `n` is of type `String`.",
    cmd: Import(String), { format!("IMPORT,{}", cmd) }, 300, Ack
}

impl FromStr for Import {
    type Err = EzoError;

    fn from_str(s: &str) -> Result<Self, EzoError> {
        let supper = s.to_uppercase();
        if supper.starts_with("IMPORT,") {
            let rest = supper.get(7..).ok_or(ErrorKind::CommandParse)?;
            let mut split = rest.split(',');
            let value = match split.next() {
                Some(n) if n.len() > 0 && n.len() < 13 => n.to_string(),
                _ => Err(ErrorKind::CommandParse)?,
            };
            match split.next() {
                None => return Ok(Import(value)),
                _ => Err(ErrorKind::CommandParse)?,
            }
        } else {
            Err(ErrorKind::CommandParse)?
        }
    }
}

define_command! {
    doc: "`L,0` command. Disable LED.",
    LedOff, { "L,0".to_string() }, 300, Ack
}

impl FromStr for LedOff {
    type Err = EzoError;

    fn from_str(s: &str) -> Result<Self, EzoError> {
        let supper = s.to_uppercase();
        match supper.as_ref() {
            "L,0" => Ok(LedOff),
            _ => Err(ErrorKind::CommandParse)?,
        }
    }
}

define_command! {
    doc: "`L,1` command. Enable LED.",
    LedOn, { "L,1".to_string() }, 300, Ack
}

impl FromStr for LedOn {
    type Err = EzoError;

    fn from_str(s: &str) -> Result<Self, EzoError> {
        let supper = s.to_uppercase();
        match supper.as_ref() {
            "L,1" => Ok(LedOn),
            _ => Err(ErrorKind::CommandParse)?,
        }
    }
}

define_command! {
    doc: "`L,?` command. Returns a `LedStatus` response. Get current LED status.",
    LedState, { "L,?".to_string() }, 300,
    resp: LedStatus, { LedStatus::parse(&resp) }
}

impl FromStr for LedState {
    type Err = EzoError;

    fn from_str(s: &str) -> Result<Self, EzoError> {
        let supper = s.to_uppercase();
        match supper.as_ref() {
            "L,?" => Ok(LedState),
            _ => Err(ErrorKind::CommandParse)?,
        }
    }
}

define_command! {
    doc: "`PLOCK,0` command. Disable I2C protocol lock.",
    ProtocolLockDisable, { "PLOCK,0".to_string() }, 300, Ack
}

impl FromStr for ProtocolLockDisable {
    type Err = EzoError;

    fn from_str(s: &str) -> Result<Self, EzoError> {
        let supper = s.to_uppercase();
        match supper.as_ref() {
            "PLOCK,0" => Ok(ProtocolLockDisable),
            _ => Err(ErrorKind::CommandParse)?,
        }
    }
}

define_command! {
    doc: "`PLOCK,1` command. Enable I2C protocol lock.",
    ProtocolLockEnable, { "PLOCK,1".to_string() }, 300, Ack
}

impl FromStr for ProtocolLockEnable {
    type Err = EzoError;

    fn from_str(s: &str) -> Result<Self, EzoError> {
        let supper = s.to_uppercase();
        match supper.as_ref() {
            "PLOCK,1" => Ok(ProtocolLockEnable),
            _ => Err(ErrorKind::CommandParse)?,
        }
    }
}

define_command! {
    doc: "`PLOCK,?` command. Returns a `ProtocolLockStatus` response. Get the Protocol Lock status.",
    ProtocolLockState, { "PLOCK,?".to_string() }, 300,
    resp: ProtocolLockStatus, { ProtocolLockStatus::parse(&resp) }
}

impl FromStr for ProtocolLockState {
    type Err = EzoError;

    fn from_str(s: &str) -> Result<Self, EzoError> {
        let supper = s.to_uppercase();
        match supper.as_ref() {
            "PLOCK,?" => Ok(ProtocolLockState),
            _ => Err(ErrorKind::CommandParse)?,
        }
    }
}

define_command! {
    doc: "`SLEEP` command. Enter sleep mode/low power.",
    Sleep, { "SLEEP".to_string() }, 0
}

impl FromStr for Sleep {
    type Err = EzoError;

    fn from_str(s: &str) -> Result<Self, EzoError> {
        let supper = s.to_uppercase();
        match supper.as_ref() {
            "SLEEP" => Ok(Sleep),
            _ => Err(ErrorKind::CommandParse)?,
        }
    }
}

define_command! {
    doc: "`STATUS` command. Returns a `DeviceStatus` response. Retrieve status information.",
    Status, { "STATUS".to_string() }, 300,
    resp: DeviceStatus, { DeviceStatus::parse(&resp) }
}

impl FromStr for Status {
    type Err = EzoError;

    fn from_str(s: &str) -> Result<Self, EzoError> {
        let supper = s.to_uppercase();
        match supper.as_ref() {
            "STATUS" => Ok(Status),
            _ => Err(ErrorKind::CommandParse)?,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_command_baud_300() {
        let cmd = Baud(BpsRate::Bps300);
        assert_eq!(cmd.get_command_string(), "BAUD,300");
        assert_eq!(cmd.get_delay(), 0);
    }

    #[test]
    fn parse_case_insensitive_command_baud_300() {
        let cmd = "baud,300".parse::<Baud>().unwrap();
        assert_eq!(cmd, Baud(BpsRate::Bps300));

        let cmd = "BAUD,300".parse::<Baud>().unwrap();
        assert_eq!(cmd, Baud(BpsRate::Bps300));
    }

    #[test]
    fn build_command_baud_1200() {
        let cmd = Baud(BpsRate::Bps1200);
        assert_eq!(cmd.get_command_string(), "BAUD,1200");
        assert_eq!(cmd.get_delay(), 0);
    }

    #[test]
    fn parse_case_insensitive_command_baud_1200() {
        let cmd = "baud,1200".parse::<Baud>().unwrap();
        assert_eq!(cmd, Baud(BpsRate::Bps1200));

        let cmd = "BAUD,1200".parse::<Baud>().unwrap();
        assert_eq!(cmd, Baud(BpsRate::Bps1200));
    }

    #[test]
    fn build_command_baud_2400() {
        let cmd = Baud(BpsRate::Bps2400);
        assert_eq!(cmd.get_command_string(), "BAUD,2400");
        assert_eq!(cmd.get_delay(), 0);
    }

    #[test]
    fn parse_case_insensitive_command_baud_2400() {
        let cmd = "baud,2400".parse::<Baud>().unwrap();
        assert_eq!(cmd, Baud(BpsRate::Bps2400));

        let cmd = "BAUD,2400".parse::<Baud>().unwrap();
        assert_eq!(cmd, Baud(BpsRate::Bps2400));
    }

    #[test]
    fn build_command_baud_9600() {
        let cmd = Baud(BpsRate::Bps9600);
        assert_eq!(cmd.get_command_string(), "BAUD,9600");
        assert_eq!(cmd.get_delay(), 0);
    }

    #[test]
    fn parse_case_insensitive_command_baud_9600() {
        let cmd = "baud,9600".parse::<Baud>().unwrap();
        assert_eq!(cmd, Baud(BpsRate::Bps9600));

        let cmd = "BAUD,9600".parse::<Baud>().unwrap();
        assert_eq!(cmd, Baud(BpsRate::Bps9600));
    }

    #[test]
    fn build_command_baud_19200() {
        let cmd = Baud(BpsRate::Bps19200);
        assert_eq!(cmd.get_command_string(), "BAUD,19200");
        assert_eq!(cmd.get_delay(), 0);
    }

    #[test]
    fn parse_case_insensitive_command_baud_19200() {
        let cmd = "baud,19200".parse::<Baud>().unwrap();
        assert_eq!(cmd, Baud(BpsRate::Bps19200));

        let cmd = "BAUD,19200".parse::<Baud>().unwrap();
        assert_eq!(cmd, Baud(BpsRate::Bps19200));
    }

    #[test]
    fn build_command_baud_38400() {
        let cmd = Baud(BpsRate::Bps38400);
        assert_eq!(cmd.get_command_string(), "BAUD,38400");
        assert_eq!(cmd.get_delay(), 0);
    }

    #[test]
    fn parse_case_insensitive_command_baud_38400() {
        let cmd = "baud,38400".parse::<Baud>().unwrap();
        assert_eq!(cmd, Baud(BpsRate::Bps38400));

        let cmd = "BAUD,38400".parse::<Baud>().unwrap();
        assert_eq!(cmd, Baud(BpsRate::Bps38400));
    }

    #[test]
    fn build_command_baud_57600() {
        let cmd = Baud(BpsRate::Bps57600);
        assert_eq!(cmd.get_command_string(), "BAUD,57600");
        assert_eq!(cmd.get_delay(), 0);
    }

    #[test]
    fn parse_case_insensitive_command_baud_57600() {
        let cmd = "baud,57600".parse::<Baud>().unwrap();
        assert_eq!(cmd, Baud(BpsRate::Bps57600));

        let cmd = "BAUD,57600".parse::<Baud>().unwrap();
        assert_eq!(cmd, Baud(BpsRate::Bps57600));
    }

    #[test]
    fn build_command_baud_115200() {
        let cmd = Baud(BpsRate::Bps115200);
        assert_eq!(cmd.get_command_string(), "BAUD,115200");
        assert_eq!(cmd.get_delay(), 0);
    }

    #[test]
    fn parse_case_insensitive_command_baud_115200() {
        let cmd = "baud,115200".parse::<Baud>().unwrap();
        assert_eq!(cmd, Baud(BpsRate::Bps115200));

        let cmd = "BAUD,115200".parse::<Baud>().unwrap();
        assert_eq!(cmd, Baud(BpsRate::Bps115200));
    }

    #[test]
    fn build_command_calibration_clear() {
        let cmd = CalibrationClear;
        assert_eq!(cmd.get_command_string(), "CAL,CLEAR");
        assert_eq!(cmd.get_delay(), 300);
    }

    #[test]
    fn parse_case_insensitive_command_calibration_clear() {
        let cmd = "cal,clear".parse::<CalibrationClear>().unwrap();
        assert_eq!(cmd, CalibrationClear);

        let cmd = "Cal,CLEAR".parse::<CalibrationClear>().unwrap();
        assert_eq!(cmd, CalibrationClear);
    }

    #[test]
    fn build_command_change_device_address() {
        let cmd = DeviceAddress(88);
        assert_eq!(cmd.get_command_string(), "I2C,88");
        assert_eq!(cmd.get_delay(), 300);
    }

    #[test]
    fn parse_case_insensitive_device_address() {
        let cmd = "i2c,1".parse::<DeviceAddress>().unwrap();
        assert_eq!(cmd, DeviceAddress(1));

        let cmd = "I2C,123".parse::<DeviceAddress>().unwrap();
        assert_eq!(cmd, DeviceAddress(123));
    }

    #[test]
    fn parse_invalid_command_device_address_yields_err() {
        let cmd = "I2C,".parse::<DeviceAddress>();
        assert!(cmd.is_err());

        let cmd = "I2C,1a21.43".parse::<DeviceAddress>();
        assert!(cmd.is_err());
    }

    #[test]
    fn build_command_device_information() {
        let cmd = DeviceInformation;
        assert_eq!(cmd.get_command_string(), "I");
        assert_eq!(cmd.get_delay(), 300);
    }

    #[test]
    fn parse_case_insensitive_command_device_information() {
        let cmd = "i".parse::<DeviceInformation>().unwrap();
        assert_eq!(cmd, DeviceInformation);

        let cmd = "I".parse::<DeviceInformation>().unwrap();
        assert_eq!(cmd, DeviceInformation);
    }

    #[test]
    fn build_command_export() {
        let cmd = Export;
        assert_eq!(cmd.get_command_string(), "EXPORT");
        assert_eq!(cmd.get_delay(), 300);
    }

    #[test]
    fn parse_case_insensitive_command_export() {
        let cmd = "export".parse::<Export>().unwrap();
        assert_eq!(cmd, Export);

        let cmd = "EXPORT".parse::<Export>().unwrap();
        assert_eq!(cmd, Export);
    }

    #[test]
    fn build_command_export_info() {
        let cmd = ExportInfo;
        assert_eq!(cmd.get_command_string(), "EXPORT,?");
        assert_eq!(cmd.get_delay(), 300);
    }

    #[test]
    fn parse_case_insensitive_command_export_info() {
        let cmd = "export,?".parse::<ExportInfo>().unwrap();
        assert_eq!(cmd, ExportInfo);

        let cmd = "EXPORT,?".parse::<ExportInfo>().unwrap();
        assert_eq!(cmd, ExportInfo);
    }

    #[test]
    fn build_command_factory() {
        let cmd = Factory;
        assert_eq!(cmd.get_command_string(), "FACTORY");
        assert_eq!(cmd.get_delay(), 0);
    }

    #[test]
    fn parse_case_insensitive_command_factory() {
        let cmd = "factory".parse::<Factory>().unwrap();
        assert_eq!(cmd, Factory);

        let cmd = "FACTORY".parse::<Factory>().unwrap();
        assert_eq!(cmd, Factory);
    }

    #[test]
    fn build_command_find() {
        let cmd = Find;
        assert_eq!(cmd.get_command_string(), "F");
        assert_eq!(cmd.get_delay(), 300);
    }

    #[test]
    fn parse_case_insensitive_command_find() {
        let cmd = "f".parse::<Find>().unwrap();
        assert_eq!(cmd, Find);

        let cmd = "F".parse::<Find>().unwrap();
        assert_eq!(cmd, Find);
    }

    #[test]
    fn build_command_import() {
        let calibration_string = "ABCDEFGHIJKLMNO".to_string();
        let cmd = Import(calibration_string);
        assert_eq!(cmd.get_command_string(), "IMPORT,ABCDEFGHIJKLMNO");
        assert_eq!(cmd.get_delay(), 300);
    }

    #[test]
    fn parse_case_insensitive_command_import() {
        let cmd = "import,1".parse::<Import>().unwrap();
        assert_eq!(cmd, Import("1".to_string()));

        let cmd = "IMPORT,abcdef".parse::<Import>().unwrap();
        assert_eq!(cmd, Import("ABCDEF".to_string()));
    }

    #[test]
    fn build_command_led_off() {
        let cmd = LedOff;
        assert_eq!(cmd.get_command_string(), "L,0");
        assert_eq!(cmd.get_delay(), 300);
    }

    #[test]
    fn parse_case_insensitive_command_led_off() {
        let cmd = "l,0".parse::<LedOff>().unwrap();
        assert_eq!(cmd, LedOff);

        let cmd = "L,0".parse::<LedOff>().unwrap();
        assert_eq!(cmd, LedOff);
    }

    #[test]
    fn build_command_led_on() {
        let cmd = LedOn;
        assert_eq!(cmd.get_command_string(), "L,1");
        assert_eq!(cmd.get_delay(), 300);
    }

    #[test]
    fn parse_case_insensitive_command_led_on() {
        let cmd = "l,1".parse::<LedOn>().unwrap();
        assert_eq!(cmd, LedOn);

        let cmd = "L,1".parse::<LedOn>().unwrap();
        assert_eq!(cmd, LedOn);
    }

    #[test]
    fn build_command_led_state() {
        let cmd = LedState;
        assert_eq!(cmd.get_command_string(), "L,?");
        assert_eq!(cmd.get_delay(), 300);
    }

    #[test]
    fn parse_case_insensitive_command_led_state() {
        let cmd = "l,?".parse::<LedState>().unwrap();
        assert_eq!(cmd, LedState);

        let cmd = "L,?".parse::<LedState>().unwrap();
        assert_eq!(cmd, LedState);
    }

    #[test]
    fn build_command_plock_disable() {
        let cmd = ProtocolLockDisable;
        assert_eq!(cmd.get_command_string(), "PLOCK,0");
        assert_eq!(cmd.get_delay(), 300);
    }

    #[test]
    fn parse_case_insensitive_command_plock_disable() {
        let cmd = "plock,0".parse::<ProtocolLockDisable>().unwrap();
        assert_eq!(cmd, ProtocolLockDisable);

        let cmd = "PLOCK,0".parse::<ProtocolLockDisable>().unwrap();
        assert_eq!(cmd, ProtocolLockDisable);
    }

    #[test]
    fn build_command_plock_enable() {
        let cmd = ProtocolLockEnable;
        assert_eq!(cmd.get_command_string(), "PLOCK,1");
        assert_eq!(cmd.get_delay(), 300);
    }

    #[test]
    fn parse_case_insensitive_command_plock_enable() {
        let cmd = "plock,1".parse::<ProtocolLockEnable>().unwrap();
        assert_eq!(cmd, ProtocolLockEnable);

        let cmd = "PLOCK,1".parse::<ProtocolLockEnable>().unwrap();
        assert_eq!(cmd, ProtocolLockEnable);
    }

    #[test]
    fn build_command_plock_status() {
        let cmd = ProtocolLockState;
        assert_eq!(cmd.get_command_string(), "PLOCK,?");
        assert_eq!(cmd.get_delay(), 300);
    }

    #[test]
    fn parse_case_insensitive_command_plock_status() {
        let cmd = "plock,?".parse::<ProtocolLockState>().unwrap();
        assert_eq!(cmd, ProtocolLockState);

        let cmd = "PLOCK,?".parse::<ProtocolLockState>().unwrap();
        assert_eq!(cmd, ProtocolLockState);
    }

    #[test]
    fn build_command_sleep_mode() {
        let cmd = Sleep;
        assert_eq!(cmd.get_command_string(), "SLEEP");
        assert_eq!(cmd.get_delay(), 0);
    }

    #[test]
    fn parse_case_insensitive_command_sleep() {
        let cmd = "Sleep".parse::<Sleep>().unwrap();
        assert_eq!(cmd, Sleep);

        let cmd = "SLEEP".parse::<Sleep>().unwrap();
        assert_eq!(cmd, Sleep);
    }

    #[test]
    fn build_command_device_status() {
        let cmd = Status;
        assert_eq!(cmd.get_command_string(), "STATUS");
        assert_eq!(cmd.get_delay(), 300);
    }

    #[test]
    fn parse_case_insensitive_command_device_status() {
        let cmd = "status".parse::<Status>().unwrap();
        assert_eq!(cmd, Status);

        let cmd = "STATUS".parse::<Status>().unwrap();
        assert_eq!(cmd, Status);
    }
}
