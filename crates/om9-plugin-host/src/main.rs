// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_plugin_host::package;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("inspect") if args.len()==2 => {
            // Inspection validates bytes only; it never loads native code.
            let package = package::read(Path::new(&args[1]))?;
            println!("{}",serde_json::to_string_pretty(&package.manifest)?);
        }
        Some("sdk-bundle") if args.len() == 3 => {
            om9_plugin_host::bundle::create(Path::new(&args[1]), Path::new(&args[2]))?;
            println!("SDK bundle: {}", args[2]);
        }
        Some("pack") if args.len()>=4 => {
            let descriptor = serde_json::from_slice(&std::fs::read(&args[1])?)?;
            let mut libraries = BTreeMap::new();
            for arg in &args[3..] {
                let (target,path) = arg.split_once('=').ok_or("library must be target-triple=path")?;
                if libraries.insert(target.into(),PathBuf::from(path)).is_some() { return Err("duplicate target".into()); }
            }
            let manifest = package::pack(descriptor,&libraries,Path::new(&args[2]))?;
            println!("{}",serde_json::to_string_pretty(&manifest)?);
        }
        _ => return Err("usage: om9-plugin inspect package.om9p | pack descriptor.json output.om9p target-triple=library [...] | sdk-bundle repo-root output.zip".into()),
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("OM9 plugin: {error}");
        std::process::exit(1);
    }
}
