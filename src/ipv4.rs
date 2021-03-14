pub use std::net::Ipv4Addr;
use std::{convert::TryFrom, str::FromStr};

use anyhow::bail;
use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mask(pub u8);

impl FromStr for Mask {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u8>()
            .map_err(|_| "Invalid subnet mask")
            .map_or_else(
                |err| Err(err),
                |mask| if mask >= 1 && mask <= 32 {Ok(Mask(mask))} else {Err("Mask should be a number between 1 and 32")})
    }
}

impl ToString for Mask {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl TryFrom<Ipv4Addr> for Mask {
    type Error = anyhow::Error;

    fn try_from(ip: Ipv4Addr) -> Result<Self, Self::Error> {
        let octets = ip.octets();
        let addr: u32 = ((octets[0] as u32 & 0xff) << 24) | ((octets[1] as u32 & 0xff) << 16) | ((octets[2] as u32 & 0xff) << 8) | (octets[3] as u32 & 0xff);

        if addr.leading_ones() + addr.trailing_zeros() == 32 {
            Ok(Mask(addr.leading_ones() as u8))
        } else {
            bail!("Not a valid mask")
        }
    }
}

impl From<Mask> for Ipv4Addr {
    fn from(mask: Mask) -> Self {
        let addr: u32 = 1 << mask.0;

        let (a, b, c, d) = (
            ((addr >> 24) & 0xff) as u8,
            ((addr >> 16) & 0xff) as u8,
            ((addr >> 8) & 0xff) as u8,
            (addr & 0xff) as u8);

        Ipv4Addr::new(a, b, c, d)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Subnet {
    pub gateway: Ipv4Addr,
    pub mask: Mask,
}

impl ToString for Subnet {
    fn to_string(&self) -> String {
        let mut s = self.gateway.to_string();
        s.push('/');
        s.push_str(self.mask.0.to_string().as_str());

        s
    }
}

impl FromStr for Subnet {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();
        if let [gateway_str, mask_str] = parts[..] {
            if let Ok(gateway) = gateway_str.parse::<Ipv4Addr>() {
                mask_str.parse::<Mask>().map(|mask| Self {gateway, mask})
            } else {
                Err("Invalid ip address format, expected XXX.XXX.XXX.XXX")
            }
        } else {
            Err("Expected <gateway-ip-address>/<mask>")
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClientSettings {
    pub ip: Ipv4Addr,
    pub subnet: Subnet,
    pub dns: Option<Ipv4Addr>,
    pub secondary_dns: Option<Ipv4Addr>,
}

impl Default for ClientSettings {
    fn default() -> ClientSettings {
        ClientSettings {
            ip: Ipv4Addr::new(192, 168, 71, 200),
            subnet: Subnet {
                gateway: Ipv4Addr::new(192, 168, 71, 1),
                mask: Mask(24),
            },
            dns: Some(Ipv4Addr::new(8, 8, 8, 8)),
            secondary_dns: Some(Ipv4Addr::new(8, 8, 4, 4)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ClientConfiguration {
    DHCP,
    Fixed(ClientSettings),
}

impl ClientConfiguration {
    pub fn as_fixed_settings_ref(&self) -> Option<&ClientSettings> {
        match self {
            Self::Fixed(client_settings) => Some(client_settings),
            _ => None
        }
    }

    pub fn as_fixed_settings_mut(&mut self) -> &mut ClientSettings {
        match self {
            Self::Fixed(client_settings) => client_settings,
            _ => {
                *self = ClientConfiguration::Fixed(Default::default());
                self.as_fixed_settings_mut()
            }
        }
    }
}

impl Default for ClientConfiguration {
    fn default() -> ClientConfiguration {
        ClientConfiguration::DHCP
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RouterConfiguration {
    pub subnet: Subnet,
    pub dhcp_enabled: bool,
    pub dns: Option<Ipv4Addr>,
    pub secondary_dns: Option<Ipv4Addr>,
}

impl Default for RouterConfiguration {
    fn default() -> RouterConfiguration {
        RouterConfiguration {
            subnet: Subnet {
                gateway: Ipv4Addr::new(192, 168, 71, 1),
                mask: Mask(24),
            },
            dhcp_enabled: true,
            dns: Some(Ipv4Addr::new(8, 8, 8, 8)),
            secondary_dns: Some(Ipv4Addr::new(8, 8, 4, 4)),
        }
    }
}
