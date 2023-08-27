use bevy::prelude::{ResMut, Resource};
use bevy_egui::egui::Window;
use bevy_egui::EguiContexts;

#[derive(Resource, Default)]
pub struct MyResource {
    label: bool,
}

pub fn ui_system(mut contexts: EguiContexts, mut resource: ResMut<MyResource>) {
    Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.label(match resource.label {
            true => "hi",
            false => "asdfasdf",
        });
        if ui.button("Click").clicked() {
            resource.label = !resource.label
        };
    });
}
