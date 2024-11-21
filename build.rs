fn main() {
    // let config = slint_build::CompilerConfiguration::new().with_style("native".into());
    slint_build::compile("ui/app-window.slint").unwrap();
}
