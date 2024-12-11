use crate::controller::{self, AgentEvent, AgentMessageBroker, Controller, ConverterArgs, Profile};
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use indicatif::{ProgressBar, ProgressStyle};
use mlua::{Function, Lua};
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::{
    fs,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Condvar, Mutex,
    },
    time::Duration,
};
use tokio::sync::mpsc::Sender;

pub struct ConverterFactory {
    tx: Option<Sender<AgentEvent>>,
    args: ConverterArgs,
    interrupt: Arc<AtomicBool>,
}

impl ConverterFactory {
    pub fn build(tx: Sender<AgentEvent>, args: ConverterArgs) -> Self {
        let interrupt = Arc::new(AtomicBool::new(false));
        let interrupt_clone = interrupt.clone();
        ctrlc::set_handler(move || {
            eprintln!(
                "\n\x1b[1;33mInterruption detected. Cancelling subsequent conversions...\x1b[0m"
            );
            interrupt_clone.store(true, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl+C handler");
        Self {
            tx: Some(tx),
            args,
            interrupt,
        }
    }

    pub async fn run(&self) {
        // spinner
        let spinner = ProgressBar::new(1);
        spinner.enable_steady_tick(Duration::from_millis(100));
        set_spinner_label(&spinner, "Loading resources");
        // stored data retrieval
        let profiles_arc = Default::default();
        let tx = self.tx.clone().unwrap();
        Controller::send_get_profiles_event(tx, Arc::clone(&profiles_arc)).await;
        let (lock, cvar) = &*profiles_arc;
        let profiles = lock.lock().unwrap();
        drop(cvar.wait(profiles).unwrap());
        spinner.finish();

        self.process_files(profiles_arc);

        let tx = self.tx.clone().unwrap();
        Controller::send_quit_event(tx).await;
    }

    fn process_files(&self, profiles: Arc<(Mutex<Vec<Profile>>, Condvar)>) {
        let lua = Lua::new();
        Arc::try_unwrap(profiles)
            .unwrap()
            .0
            .into_inner()
            .unwrap()
            .into_iter()
            .for_each(|profile| {
                if self.interrupt.load(Ordering::SeqCst) {
                    return;
                }
                println!("\x1b[1mRunning '{}' engine\x1b[0m", profile.engine);
                // ignore patterns
                let mut override_builder = OverrideBuilder::new(&profile.source_root);
                override_builder.inverted_matching(false);
                if let Some(ignore_pattern) = &profile.ignore_patterns {
                    ignore_pattern.iter().for_each(|glob| {
                        override_builder.add(glob).unwrap();
                    });
                };
                let override_construct = override_builder.build().unwrap();
                // walker configuration
                let mut walk_builder = WalkBuilder::new(&profile.source_root);
                walk_builder
                    .hidden(self.args.hidden)
                    .overrides(override_construct);
                // load lua converter
                let converter: Function = lua
                    .load(controller::get_converters_dir().join(&profile.engine))
                    .eval()
                    .unwrap();
                // iterate over files
                walk_builder
                    .build()
                    .filter_map(|entry| {
                        entry
                            .map_err(|err| eprintln!("Error processing directory entry: {}", err))
                            .ok()
                    })
                    .filter(|entry| {
                        entry
                            .path()
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .is_some_and(|ext_str| ext_str == profile.source_file_extension)
                    })
                    .par_bridge()
                    .for_each(|entry| {
                        if self.interrupt.load(Ordering::SeqCst) {
                            return;
                        }
                        // create output path
                        let output_path = Path::new(&profile.output_root).join(
                            entry
                                .path()
                                .strip_prefix(&profile.source_root)
                                .unwrap()
                                .parent()
                                .unwrap(),
                        );
                        fs::create_dir_all(&output_path).unwrap();
                        // define source_file, output_file
                        let source_file = entry.path();
                        let output_file = output_path
                            .join(entry.file_name())
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
                        eprintln!(
                            "\x1b[2mSuccess [{}]: {}\x1b[0m",
                            source_file.to_str().unwrap(),
                            result
                        );
                    });
            });
    }
}

fn set_spinner_label(pb: &ProgressBar, label: &str) {
    pb.set_style(
        ProgressStyle::default_spinner()
            .template(&format!("{} {{spinner}}", label))
            .unwrap()
            .tick_chars("◇◈◆✓"),
    );
}
