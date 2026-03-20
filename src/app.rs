use ratatui::layout::Rect;

#[derive(Default)]
pub struct App {
    pub status: String,
    pub body: String,
    pub button_rect: Rect,
    pub likes_counter: u32,
}
