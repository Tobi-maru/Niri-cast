use crate::profiles::TvProfile;

pub fn choose_profile_for_outputs<'a>(
    profiles: &'a [TvProfile],
    outputs: &[String],
) -> Option<&'a TvProfile> {
    profiles.iter().find(|profile| {
        profile
            .hdmi_output
            .as_ref()
            .map(|needle| outputs.iter().any(|output| output.contains(needle)))
            .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::choose_profile_for_outputs;
    use crate::profiles::TvProfile;

    #[test]
    fn matches_hdmi_output_name() {
        let profiles = vec![TvProfile {
            name: "tv".to_string(),
            hdmi_output: Some("HDMI-A-1".to_string()),
            audio_sink: None,
        }];

        let outputs = vec!["DP-1".to_string(), "HDMI-A-1 connected".to_string()];
        assert!(choose_profile_for_outputs(&profiles, &outputs).is_some());
    }
}
