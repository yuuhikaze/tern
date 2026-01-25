#[cfg(feature = "gui")]
slint::include_modules!();

use std::rc::Rc;

#[cfg(feature = "gui")]
use slint::{Model, SharedString, VecModel};

use crate::controller::{self, AgentEvent, AgentMessageBroker, Controller, InterfaceArgs, Profile};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub trait Interface {
    fn spawn_and_run(&mut self);
}

pub struct InterfaceBuilder;

impl InterfaceBuilder {
    pub fn build(tx: Sender<AgentEvent>, args: InterfaceArgs) -> Box<dyn Interface> {
        #[cfg(feature = "gui")]
        if !args.tui {
            return Box::new(GraphicalInterface {
                tx: Some(tx),
                app: None,
            });
        }

        Box::new(CommandLineInterface)
    }
}

#[cfg(feature = "gui")]
struct GraphicalInterface {
    tx: Option<Sender<AgentEvent>>,
    app: Option<AppWindow>,
}

#[cfg(feature = "gui")]
impl Interface for GraphicalInterface {
    fn spawn_and_run(&mut self) {
        self.app = Some(AppWindow::new().unwrap());
        self.manage_interface_related_callbacks();
        self.manage_model_related_callbacks();
        self.app.as_ref().unwrap().run().unwrap();
        self.clean();
    }
}

#[cfg(feature = "gui")]
impl GraphicalInterface {
    fn manage_interface_related_callbacks(&self) {
        let app_weak = self.app.as_ref().unwrap().as_weak();
        // get, set stored and available engines
        let app = app_weak.unwrap();
        let tx = self.tx.clone();
        self.app
            .as_ref()
            .unwrap()
            .global::<Backend>()
            .on_set_stored_and_available_engines(move || {
                let stored_engines_arc = Default::default();
                let _runtime_guard = controller::get_runtime_handle().enter();
                let message_handle = async {
                    Controller::send_get_column_event(
                        tx.clone().unwrap(),
                        Arc::clone(&stored_engines_arc),
                        "engine".to_string(),
                    )
                    .await;
                };
                futures::executor::block_on(message_handle);
                let (lock, cvar) = &*stored_engines_arc;
                let stored_engines = lock.lock().unwrap();
                drop(cvar.wait(stored_engines).unwrap());
                let stored_engines = Arc::try_unwrap(stored_engines_arc)
                    .unwrap()
                    .0
                    .into_inner()
                    .unwrap();
                let stored_engines_model: Rc<VecModel<SharedString>> = Rc::new(VecModel::from(
                    stored_engines
                        .into_iter()
                        .map(Into::into)
                        .collect::<Vec<_>>(),
                ));
                app.global::<Backend>()
                    .set_stored_engines(stored_engines_model.into());
            });
        self.app
            .as_ref()
            .unwrap()
            .global::<Backend>()
            .invoke_set_stored_and_available_engines();
        // set available engines
        let app = app_weak.unwrap();
        let available_engines: Vec<String> = controller::read_data_dir()
            .filter_map(|entry| entry.ok().and_then(|e| e.file_name().into_string().ok()))
            .collect();
        let available_engines_model: Rc<VecModel<SharedString>> = Rc::new(VecModel::from(
            available_engines
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>(),
        ));
        app.global::<Backend>()
            .set_available_engines(available_engines_model.into());
        // set focus candidate on click
        let app = app_weak.unwrap();
        self.app
            .as_ref()
            .unwrap()
            .global::<Backend>()
            .on_set_focus_candidate(move |focus_candidate: FocusCandidate| {
                let focus_candidate_list = app.global::<Backend>().get_focus_candidate_list();
                let matched_index = focus_candidate_list
                    .as_any()
                    .downcast_ref::<VecModel<FocusCandidate>>()
                    .unwrap()
                    .iter()
                    .position(|e| e == focus_candidate)
                    .unwrap();
                app.global::<Backend>()
                    .set_focus_candidate_index(matched_index as i32);
            });
    }

    fn manage_model_related_callbacks(&self) {
        let app_weak = self.app.as_ref().unwrap().as_weak();
        // store profile
        let app = app_weak.unwrap();
        let tx = self.tx.clone();
        self.app
            .as_ref()
            .unwrap()
            .global::<Backend>()
            .on_store_profile(move || {
                macro_rules! construct_vector_from_getter {
                    ( $x:ident ) => {{
                        let column = app.global::<Backend>().$x().to_string();
                        if !column.is_empty() {
                            Some(column.lines().map(|e| e.into()).collect())
                        } else {
                            None
                        }
                    }};
                }
                let options = construct_vector_from_getter!(get_options);
                let ignore_patterns = construct_vector_from_getter!(get_ignore_patterns);
                let profile_arc = Arc::new(Profile {
                    id: 0,
                    engine: app.global::<Backend>().get_engine().to_string(),
                    source_root: app.global::<Backend>().get_source_root().to_string(),
                    source_file_extension: app
                        .global::<Backend>()
                        .get_source_file_extension()
                        .to_string(),
                    output_root: app.global::<Backend>().get_output_root().to_string(),
                    output_file_extension: app
                        .global::<Backend>()
                        .get_output_file_extension()
                        .to_string(),
                    options,
                    ignore_patterns,
                    metadata: None,
                });
                let profile = Arc::clone(&profile_arc);
                let _runtime_guard = controller::get_runtime_handle().enter();
                let message_handle = async {
                    Controller::send_store_profile_event(tx.clone().unwrap(), profile).await;
                };
                futures::executor::block_on(message_handle);
            });
    }

    fn clean(&mut self) {
        let _runtime_guard = controller::get_runtime_handle().enter();
        let message_handle = async {
            Controller::send_quit_event(self.tx.clone().unwrap()).await;
        };
        futures::executor::block_on(message_handle);
        drop(self.tx.take());
    }
}

struct CommandLineInterface;

impl Interface for CommandLineInterface {
    fn spawn_and_run(&mut self) {
        println!("Tern Core: Batch conversion complete (TUI not yet implemented).");
    }
}
