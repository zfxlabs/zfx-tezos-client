use std::fs;
use std::process::Command;

fn main() -> std::io::Result<()> {
    let st = Command::new("npm").args(&["install"]).status()?;
    assert!(st.success());
    let st = Command::new("npx").args(&["webpack"]).status()?;
    assert!(st.success());
    // Delete generated LICENSE
    let lic = "./scripts/tezos_js_bridge.bundle.js.LICENSE.txt";
    fs::remove_file(lic)?;
    return Ok(());
}
