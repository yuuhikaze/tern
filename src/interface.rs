slint::include_modules!();

pub struct InterfaceBuilder {
    interface: Box<dyn Interface>,
}

impl InterfaceBuilder {
    pub fn new(tui: bool) -> Self {
        Self {
            interface: if tui { Box::new(CommandLineInterface) } else { Box::new(GraphicalInterface) },
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
