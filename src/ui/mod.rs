mod home;
mod typing;
mod results;
mod theme_picker;
mod widgets;

use ratatui::Frame;
use crate::app::{App, Screen};

pub fn draw(f: &mut Frame, app: &App) {
    match app.screen {
        Screen::Home => home::draw(f, app),
        Screen::Typing => typing::draw(f, app),
        Screen::Results => results::draw(f, app),
        Screen::ThemePicker => theme_picker::draw(f, app),
        Screen::Settings => home::draw(f, app), // placeholder
    }
}
