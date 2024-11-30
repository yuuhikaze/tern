slint::include_modules!();

use std::rc::Rc;

use slint::{Model, SharedString, VecModel};

use crate::controller::{self, ArgParser, InterfaceEvent, Profile};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub struct InterfaceBuilder;

impl InterfaceBuilder {
    pub fn build(args: ArgParser, tx: Sender<InterfaceEvent>) -> Box<dyn Interface> {
        if args.tui {
            Box::new(CommandLineInterface)
        } else {
            Box::new(GraphicalInterface {
                tx: Some(tx),
                app: None,
            })
        }
    }
}

pub trait Interface {
    fn spawn_and_run(&mut self);
}

struct GraphicalInterface {
    tx: Option<Sender<InterfaceEvent>>,
    app: Option<AppWindow>,
}

impl GraphicalInterface {
    fn manage_backend_callbacks(&self) {
        // save profile
        let app_weak = self.app.as_ref().unwrap().as_weak();
        let app = app_weak.unwrap();
        let tx = self.tx.clone();
        self.app
            .as_ref()
            .unwrap()
            .global::<Backend>()
            .on_save_profile(move || {
                macro_rules! construct_vector_from_getter {
                    ( $x:ident ) => {{
                        let options = app.global::<Backend>().$x().to_string();
                        if !options.is_empty() {
                            Some(options.lines().map(|e| e.into()).collect())
                        } else {
                            None
                        }
                    }};
                }
                let options = construct_vector_from_getter!(get_options);
                let ignore_patterns = construct_vector_from_getter!(get_ignore_patterns);
                let profile_arc = Arc::new(Profile {
                    engine: app.global::<Backend>().get_engine().to_string(),
                    source_path: app.global::<Backend>().get_source_path().to_string(),
                    source_file_extension: app
                        .global::<Backend>()
                        .get_source_file_extension()
                        .to_string(),
                    output_path: app.global::<Backend>().get_output_path().to_string(),
                    output_file_extension: app
                        .global::<Backend>()
                        .get_output_file_extension()
                        .to_string(),
                    options,
                    ignore_patterns,
                    metadata: None,
                });
                let tx = tx.clone();
                let profile = Arc::clone(&profile_arc);
                tokio::spawn(async move {
                    if (tx.unwrap().send(InterfaceEvent::Save(Some(profile))).await).is_err() {
                        println!("Receiver dropped");
                    }
                });
            });
        // show available engines
        let engines: Vec<String> = controller::read_data_dir()
            .filter_map(|entry| entry.ok().and_then(|e| e.file_name().into_string().ok()))
            .collect();
        let engines_model: Rc<VecModel<SharedString>> = Rc::new(VecModel::from(
            engines.into_iter().map(Into::into).collect::<Vec<_>>(),
        ));
        self.app
            .as_ref()
            .unwrap()
            .global::<Backend>()
            .set_engines(engines_model.clone().into());
        // set focus candidate
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
                    .unwrap().iter().position(|e| e == focus_candidate).unwrap();
                app.global::<Backend>().set_focus_candidate_index(matched_index as i32);
            });
    }

    fn clean(&mut self) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            if (tx.unwrap().send(InterfaceEvent::Quit).await).is_err() {
                println!("Receiver dropped");
            }
        });
        drop(self.tx.take());
    }
}

impl Interface for GraphicalInterface {
    fn spawn_and_run(&mut self) {
        self.app = Some(AppWindow::new().unwrap());
        self.manage_backend_callbacks();
        self.app.as_ref().unwrap().run().unwrap();
        self.clean();
    }
}

struct CommandLineInterface;

impl Interface for CommandLineInterface {
    fn spawn_and_run(&mut self) {
        todo!()
    }
}
