use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), _) => app.running = false,
        (KeyCode::Tab, _) | (KeyCode::Right, _) => app.next_tab(),
        (KeyCode::BackTab, _) | (KeyCode::Left, _) => app.previous_tab(),
        (KeyCode::Char('r'), _) => app.refresh_discovery(),
        (KeyCode::Char('d'), _) => app.run_diagnostics(),
        (KeyCode::Char('c'), _) => app.cast_preflight(),
        (KeyCode::Char('e'), _) => app.cast_extend_right(),
        (KeyCode::Char('w'), _) => app.cast_extend_left(),
        (KeyCode::Char('v'), _) => app.cast_mirror(),
        (KeyCode::Char('h'), _) => app.cast_hdmi_only(),
        (KeyCode::Char('u'), _) => app.cast_restore_all(),
        (KeyCode::Char('m'), _) => app.discover_hdmi_outputs(),
        (KeyCode::Char('a'), _) => app.apply_hdmi_audio(),
        (KeyCode::Char('j'), _) => app.select_next_audio_sink(),
        (KeyCode::Char('k'), _) => app.select_prev_audio_sink(),
        (KeyCode::Enter, _) => app.apply_selected_audio_sink(),
        (KeyCode::Char('p'), _) => app.switch_to_laptop_audio(),
        (KeyCode::Char('t'), _) => app.switch_to_tv_audio(),
        (KeyCode::Char('s'), _) => app.save_profile(),
        (KeyCode::Char('l'), _) => app.load_profile(),
        (KeyCode::Char('x'), KeyModifiers::CONTROL) => app.running = false,
        _ => {}
    }
}
