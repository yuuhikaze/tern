use std::{fs, path::Path, sync::Arc};

use ignore::{overrides::OverrideBuilder, types::TypesBuilder, WalkBuilder, WalkState};
use mlua::{Function, Lua};
use tokio::sync::mpsc::Sender;

use crate::controller::{self, AgentEvent, AgentMessageBroker, Controller, ConverterArgs};

pub struct ConverterFactory {
    tx: Option<Sender<AgentEvent>>,
    args: ConverterArgs,
}

impl ConverterFactory {
    pub fn build(tx: Sender<AgentEvent>, args: ConverterArgs) -> Self {
        Self { tx: Some(tx), args }
    }

    pub fn run(&self) {
        let profiles_arc = Default::default();
        let tx = self.tx.clone().unwrap();
        let _runtime_guard = controller::get_runtime_handle().enter();
        let message_handle = async {
            Controller::send_get_profiles_event(tx, Arc::clone(&profiles_arc)).await;
        };
        futures::executor::block_on(message_handle);
        let (lock, cvar) = &*profiles_arc;
        let profiles = lock.lock().unwrap();
        drop(cvar.wait(profiles).unwrap());
        let lua = Lua::new();
        Arc::try_unwrap(profiles_arc)
            .unwrap()
            .0
            .into_inner()
            .unwrap()
            .iter()
            .for_each(|profile| {
                println!("\x1b[1mRunning '{}' engine\x1b[0m", profile.engine);
                // ignore patterns
                let mut override_builder = OverrideBuilder::new(&profile.source_root);
                if let Some(ignore_pattern) = &profile.ignore_patterns {
                    ignore_pattern.iter().for_each(|inverted_glob| {
                        override_builder.add(&format!("!{}", inverted_glob)).unwrap();
                    });
                };
                let override_construct = override_builder.build().unwrap();
                // file extension matching
                let mut types_builder = TypesBuilder::new();
                types_builder
                    .add(
                        &profile.source_file_extension,
                        &format!("*.{}", &profile.source_file_extension),
                    )
                    .unwrap();
                let types = types_builder
                    .select(&profile.source_file_extension)
                    .build()
                    .unwrap();
                // walker configuration
                let mut walk_builder = WalkBuilder::new(&profile.source_root);
                walk_builder
                    .hidden(self.args.hidden)
                    .overrides(override_construct)
                    .types(types);
                // load lua converter
                let converter: Function = lua
                    .load(controller::get_converters_dir().join(&profile.engine))
                    .eval()
                    .unwrap();
                // matched files iteration
                walk_builder.build_parallel().run(|| {
                    Box::new(|source_path| {
                        let source_path = source_path.unwrap();
                        if source_path.file_type().unwrap().is_file() {
                            // create output path
                            let output_path = Path::new(&profile.output_root).join(
                                source_path
                                    .path()
                                    .strip_prefix(&profile.source_root)
                                    .unwrap()
                                    .parent()
                                    .unwrap(),
                            );
                            fs::create_dir_all(&output_path).unwrap();
                            // define source_file, output_file
                            let source_file = source_path.path();
                            let output_file = output_path
                                .join(source_path.file_name())
                                .with_extension(&profile.output_file_extension);
                            // notify conversion has started
                            println!("Processing: {}", source_file.to_str().unwrap());
                            // run converter
                            let result = converter
                                .call::<bool>((
                                    source_file,
                                    output_file,
                                    profile.options.clone().unwrap_or(vec!["".to_string()]),
                                ))
                                .unwrap();
                            // notify conversion status
                            println!(
                                "\x1b[2mSuccess [{}]: {}\x1b[0m",
                                source_file.to_str().unwrap(),
                                result
                            );
                        }
                        WalkState::Continue
                    })
                });
            });
        let tx = self.tx.clone().unwrap();
        let _runtime_guard = controller::get_runtime_handle().enter();
        let message_handle = async {
            Controller::send_quit_event(tx).await;
        };
        futures::executor::block_on(message_handle);
    }
}
