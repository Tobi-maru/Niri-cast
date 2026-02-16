use crate::adapters::audio::AudioAdapter;

pub fn switch_to_hdmi(audio: &AudioAdapter) -> anyhow::Result<Option<String>> {
    audio.select_first_hdmi_sink()
}
