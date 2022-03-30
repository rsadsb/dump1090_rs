use serde::{Deserialize, Serialize};

pub const DEFAULT_CONFIG: &str = include_str!("../config.toml");

#[derive(Debug, Deserialize, Serialize)]
pub struct SdrConfig {
    pub sdrs: Vec<Sdr>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Sdr {
    pub driver: String,
    pub setting: Option<Vec<Arg>>,
    pub gain: Vec<Gain>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Arg {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Gain {
    pub key: String,
    pub value: f64,
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
