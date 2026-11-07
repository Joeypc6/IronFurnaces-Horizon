fn ironfurnaces_screen_click(game: &crate::game::Game<'_>, block: &str, data: &[i32], at: [f32; 2], right: bool, button: &mut Option<i32>) {
    crate::game::ironfurnaces::screen_click(block, data, at, right, game.has_shift_down(), button);
}
