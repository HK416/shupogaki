use bevy::{prelude::*, window::WindowResized};

use super::*;

// --- UPDATE SYSTEMS ---

pub fn initialize_font_size(
    windows: Query<&Window>,
    mut query: Query<(&mut TextFont, &ResizableFont), Added<ResizableFont>>,
) {
    let window = windows.single().unwrap();
    for (mut font, &resizable) in query.iter_mut() {
        match resizable {
            ResizableFont::Vertical { base, size } => {
                let font_size = window.height() / base * size;
                font.font_size = font_size;
            }
        }
    }
}

pub fn update_font_size(
    mut reader: EventReader<WindowResized>,
    mut query: Query<(&mut TextFont, &ResizableFont)>,
) {
    for event in reader.read() {
        for (mut font, &resizable) in query.iter_mut() {
            match resizable {
                ResizableFont::Vertical { base, size } => {
                    let font_size = event.height / base * size;
                    font.font_size = font_size;
                }
            }
        }
    }
}
