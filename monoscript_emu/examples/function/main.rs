use monoscript_emu::run_script;

fn main() {
    let script = include_str!("function.ms");
    run_script(script).expect("failed to run script");
}
