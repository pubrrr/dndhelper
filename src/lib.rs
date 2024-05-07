pub mod common;
#[cfg(feature = "bevy")]
pub mod game;
pub mod scan_assets;

#[cfg(test)]
mod tests {
    use bevy::ecs::event::ManualEventReader;
    use bevy::log::debug;
    use bevy::prelude::{App, Event, Events, State, States};

    pub trait AppWrapper {
        fn app(&self) -> &App;

        fn app_mut(&mut self) -> &mut App;

        fn update(&mut self) {
            debug!("---------------- Updating app ----------------");
            self.app_mut().update();
        }

        fn send_event<E: Event>(&mut self, event: E) {
            self.app_mut().world.send_event(event);
        }

        fn get_event_reader<E: Event>(&self) -> ManualEventReader<E> {
            self.app().world.resource::<Events<E>>().get_reader()
        }

        fn get_events<E: Event + Clone>(&self, event_reader: &mut ManualEventReader<E>) -> Vec<E> {
            event_reader
                .read(self.app().world.resource::<Events<E>>())
                .cloned()
                .collect()
        }

        fn get<S: States>(&self) -> &S {
            self.app().world.resource::<State<S>>()
        }
    }

    #[macro_export]
    macro_rules! generate_test_app {
        ($($field_name:ident: $field_type:ty),*) => {
            struct TestApp {
                app: App,
                $($field_name: $field_type),*
            }

            impl AppWrapper for TestApp {
                fn app(&self) -> &App {
                    &self.app
                }

                fn app_mut(&mut self) -> &mut App {
                    &mut self.app
                }
            }
        };
    }
}
