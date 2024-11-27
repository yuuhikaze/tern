slint::include_modules!();

use std::rc::Rc;

use slint::{SharedString, VecModel};

use crate::config::{self};

pub struct InterfaceBuilder {
    interface: Box<dyn Interface>,
}

impl InterfaceBuilder {
    pub fn new(tui: bool) -> Self {
        Self {
            interface: if tui {
                Box::new(CommandLineInterface)
            } else {
                Box::new(GraphicalInterface)
            },
        }
    }

    pub fn spawn(&self) {
        self.interface.spawn();
    }
}

trait Interface {
    fn spawn(&self);
}

struct GraphicalInterface;

impl GraphicalInterface {
    fn manage_buttons(&self, app: &AppWindow) {
        // let app_weak = app.as_weak();
        app.global::<Backend>().on_save_profile(|| {});

        let engines: Vec<String> = config::read_data_dir()
            .filter_map(|entry| entry.ok().and_then(|e| e.file_name().into_string().ok()))
            .collect();
        let engines_model: Rc<VecModel<SharedString>> = Rc::new(VecModel::from(engines.into_iter().map(Into::into).collect::<Vec<_>>()));
        app.global::<Backend>().set_engines(engines_model.clone().into());
    }
}

impl Interface for GraphicalInterface {
    fn spawn(&self) {
        let app = AppWindow::new().unwrap();
        self.manage_buttons(&app);
        app.run().unwrap();
    }
}

struct CommandLineInterface;

impl Interface for CommandLineInterface {
    fn spawn(&self) {
        todo!()
    }
}
