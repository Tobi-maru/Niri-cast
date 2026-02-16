use crate::adapters::niri::NiriAdapter;

pub fn list_hdmi_outputs(niri: &NiriAdapter) -> anyhow::Result<Vec<String>> {
    niri.list_hdmi_outputs()
}
