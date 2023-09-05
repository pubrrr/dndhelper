use bevy::prelude::{debug, DetectChanges, Event, EventReader, Local, Res, ResMut, Resource};
use bevy_egui::egui::{Align, ScrollArea, Window};
use bevy_egui::EguiContexts;

#[derive(Event, Debug, Clone)]
pub struct LogEvent {
    pub message: String,
}

#[derive(Resource, Debug, Default)]
pub struct LogRecord {
    storage: Vec<LogEvent>,
}

pub fn handle_log_events(mut log_events: EventReader<LogEvent>, mut log_record: ResMut<LogRecord>) {
    for log_event in log_events.iter() {
        log_record.storage.push(log_event.clone());
    }
}

pub fn display_log_events(mut contexts: EguiContexts, log_record: Res<LogRecord>) {
    Window::new("Logs")
        .default_size((250., 150.))
        .show(contexts.ctx_mut(), |ui| {
            ScrollArea::both().show(ui, |ui| {
                for log_event in log_record.storage.iter() {
                    ui.label(&log_event.message);
                }
                if log_record.is_changed() {
                    debug!("Scrolling to bottom of logs");
                    ui.scroll_to_cursor(Some(Align::BOTTOM));
                }
            });
        });
}
