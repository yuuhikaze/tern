use std::{sync::Arc, thread, time::Duration};

use ignore::{types::TypesBuilder, WalkBuilder, WalkState};
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
        Controller::send_get_profiles_event(tx, Arc::clone(&profiles_arc));
        thread::sleep(Duration::from_millis(100)); // cheap hack to wait for arc mutex data-wise readiness
        Arc::try_unwrap(profiles_arc)
            .unwrap()
            .into_inner()
            .unwrap()
            .iter()
            .for_each(|profile| {
                println!("Running '{}' engine", profile.engine);
                let mut types = TypesBuilder::new();
                types
                    .add(
                        &profile.source_file_extension,
                        &format!("*.{}", &profile.source_file_extension),
                    )
                    .unwrap();
                let types = types
                    .select(&profile.source_file_extension)
                    .build()
                    .unwrap();
                let temp_ignore = temp_file::with_contents(
                    profile
                        .ignore_patterns
                        .clone()
                        .unwrap_or(vec!["".to_string()])
                        .join("\n")
                        .as_bytes(),
                );
                let mut walker = WalkBuilder::new(&profile.source_path);
                let walker = walker
                    .hidden(self.args.hidden)
                    .ignore(false)
                    .git_ignore(false)
                    .git_global(false)
                    .git_exclude(false)
                    .require_git(false)
                    .types(types);
                walker.add_ignore(temp_ignore);
                walker.build_parallel().run(|| {
                    Box::new(|source_path| {
                        if source_path.clone().unwrap().file_type().unwrap().is_file() {
                            let lua = Lua::new();
                            let converter: Function = lua
                                .load(controller::get_converters_dir().join(&profile.engine))
                                .eval()
                                .unwrap();
                            let source_file = String::from(source_path.unwrap().path().to_str().unwrap());
                            let output_file = format!(
                                "{}{}",
                                &source_file
                                    [..source_file.len() - profile.source_file_extension.len()],
                                profile.output_file_extension
                            );
                            println!("Processing: {}", source_file);
                            converter
                                .call::<()>((
                                    source_file.clone(),
                                    output_file,
                                    profile.options.clone().unwrap_or(vec!["".to_string()]),
                                ))
                                .unwrap();
                            println!("Conversion successful for: {}", source_file);
                        }
                        WalkState::Continue
                    })
                });
            });
        let tx = self.tx.clone().unwrap();
        Controller::send_quit_event(tx);
    }
}
