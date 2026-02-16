use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TvProfile {
    pub name: String,
    pub hdmi_output: Option<String>,
    pub audio_sink: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileCollection {
    pub profiles: Vec<TvProfile>,
}
