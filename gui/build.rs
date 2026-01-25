fn main() {
    #[cfg(feature = "gui")]
    {
        let config = slint_build::CompilerConfiguration::new().with_style("qt".into());
        slint_build::compile_with_config("ui/app-window.slint", config).unwrap();
    }
}
