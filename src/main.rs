use bevy::DefaultPlugins;
use bevy::prelude::{App, Update, Resource, Res, ResMut};
use bevy_egui::{EguiContexts, EguiPlugin};
use bevy_egui::egui::Window;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EguiPlugin))
        .add_systems(Update, ui_system)
        .init_resource::<MyResource>()
        .run();
}

fn ui_system(mut contexts: EguiContexts,mut resource: ResMut<MyResource>) {
    Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.label(match resource.label {
            true => "hi",
            false => "asdfasdf"
        });
        if ui.button("Click").clicked() {
            resource.label = !resource.label
        };
    });
}

#[derive(Resource, Default)]
struct MyResource {
    label: bool,
}
