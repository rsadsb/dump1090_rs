use serde::Deserialize;

pub const DEFAULT_CONFIG: &str = include_str!("../config.toml");
#[derive(Debug, Deserialize)]
pub struct SdrConfig {
    pub sdrs: Vec<Sdr>,
}

#[derive(Debug, Deserialize)]
pub struct Sdr {
    #[serde(default = "Sdr::default_channel")]
    pub channel: usize,
    pub driver: String,
    pub setting: Option<Vec<Arg>>,
    pub gain: Vec<Gain>,
    pub antenna: Option<Antenna>,
}

impl Sdr {
    pub fn default_channel() -> usize {
        0
    }
}

#[derive(Debug, Deserialize)]
pub struct Arg {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct Gain {
    pub key: String,
    pub value: f64,
}

#[derive(Debug, Deserialize)]
pub struct Antenna {
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_include_str() {
        // ensure that the include_str config compiles to an SdrConfig
        let _: SdrConfig = toml::from_str(DEFAULT_CONFIG).unwrap();
    }
}
