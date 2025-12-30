
use rust_gui::app::{FileExplorerApp};


fn main() {
    let _ = iced::application(FileExplorerApp::default, FileExplorerApp::update, FileExplorerApp::view)
    .font(iced_fonts::FONTAWESOME_FONT_BYTES)
    .run();
}