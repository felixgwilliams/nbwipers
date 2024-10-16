use std::io::{stdin, Read};
use std::io::{stdout, Write};

use anyhow::bail;
use serde_json::{json, Value};

use crate::files::get_cwd;
use crate::record::{get_kernelspec_file, read_kernelspec_file, KernelSpecInfo};
use crate::schema::RawNotebook;
use crate::strip::write_nb;

pub fn smudge(path: String) -> Result<(), anyhow::Error> {
    let mut in_nb_bytes = Vec::new();
    stdin().lock().read_to_end(&mut in_nb_bytes)?;
    // let lock = std_in.lock();
    // serde_json::from_reader(lock)?
    let kernelspec_info =
        read_kernelspec_file(get_kernelspec_file(get_cwd())?)?.unwrap_or_default();
    match kernelspec_info.get(&path) {
        Some(kernel_spec) => {
            let out_nb = maybe_replace_kernelspec(&in_nb_bytes, kernel_spec)?;
            write_nb(stdout(), &out_nb)?;
        }
        None => {
            stdout().write_all(&in_nb_bytes)?;
        }
    }

    Ok(())
}

fn maybe_replace_kernelspec(
    nb_in: &[u8],
    kernelspec_info: &KernelSpecInfo,
) -> Result<RawNotebook, anyhow::Error> {
    let mut nb = serde_json::from_slice::<RawNotebook>(nb_in)?;

    if !kernelspec_info.kernelspec.is_null() {
        match nb.metadata {
            Value::Null => {
                nb.metadata = json!(
                    {
                        "kernelspec": kernelspec_info.kernelspec,
                    }
                )
            }
            Value::Object(ref mut meta) => {
                if !meta.contains_key("kernelspec") {
                    meta.insert("kernelspec".to_string(), json!(kernelspec_info.kernelspec));
                }
            }
            _ => bail!("Unexpected metadata type"),
        }
    }
    if let Some(ref version) = kernelspec_info.python_version {
        match nb.metadata {
            Value::Null => {
                nb.metadata = json!(
                    {"language_info":{"version":version}}
                )
            }
            Value::Object(ref mut meta) => match meta.get_mut("language_info") {
                Some(Value::Null) => {
                    meta.insert("language_info".to_string(), json!({"version":version}));
                }
                Some(Value::Object(lang_info)) => {
                    if !lang_info.contains_key("version") {
                        lang_info.insert("version".to_string(), json!(version));
                    }
                }
                _ => bail!("Unexpected language info type"),
            },
            _ => bail!("Unexpected metadata type"),
        }
    };

    Ok(nb)
}
