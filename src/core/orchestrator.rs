use crate::adapters::{audio::AudioAdapter, niri::NiriAdapter};

pub fn switch_to_tv_mode(niri: &NiriAdapter, audio: &AudioAdapter) -> anyhow::Result<String> {
    let outputs = niri.list_hdmi_outputs()?;
    let sink = audio.select_first_hdmi_sink()?;

    Ok(format!(
        "tv mode updated: hdmi_outputs={}, selected_sink={}",
        outputs.len(),
        sink.unwrap_or_else(|| "none".to_string())
    ))
}
