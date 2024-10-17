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
                    meta.shift_insert(
                        0,
                        "kernelspec".to_string(),
                        json!(kernelspec_info.kernelspec),
                    );
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
                Some(Value::Null) | None => {
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

#[cfg(test)]
mod test {
    use serde_json::{json, Value};

    use super::maybe_replace_kernelspec;
    use crate::{record::KernelSpecInfo, schema::RawNotebook, strip::write_nb};

    fn make_sample_kernelinfo() -> KernelSpecInfo {
        KernelSpecInfo {
            kernelspec: json!({
                "name": "python3",
                "display_name": "Python 3"
            }),
            python_version: Some("3.12.4".to_string()),
        }
    }
    fn notebook_with_metadata_bytes(meta: serde_json::Value) -> Vec<u8> {
        let nb = RawNotebook {
            metadata: meta,
            ..Default::default()
        };
        let mut nb_bytes = Vec::new();
        write_nb(&mut nb_bytes, &nb).unwrap();
        nb_bytes
    }

    #[test]
    fn test_add_to_blank() {
        let kernelspec_info = make_sample_kernelinfo();
        let nb_bytes = notebook_with_metadata_bytes(Value::Null);

        let out_nb = maybe_replace_kernelspec(&nb_bytes, &kernelspec_info).unwrap();

        assert_eq!(
            out_nb.metadata.get("kernelspec"),
            Some(&kernelspec_info.kernelspec)
        );
        assert_eq!(
            out_nb
                .metadata
                .get("language_info")
                .unwrap()
                .get("version")
                .unwrap()
                .as_str()
                .unwrap(),
            kernelspec_info.python_version.unwrap()
        );
    }
    #[test]
    fn test_add_to_blank_only_version() {
        let kernelspec_info = KernelSpecInfo {
            kernelspec: Value::Null,
            python_version: Some("3.12.4".to_string()),
        };
        let nb_bytes = notebook_with_metadata_bytes(Value::Null);

        let out_nb = maybe_replace_kernelspec(&nb_bytes, &kernelspec_info).unwrap();

        assert_eq!(
            out_nb
                .metadata
                .get("language_info")
                .unwrap()
                .get("version")
                .unwrap()
                .as_str()
                .unwrap(),
            kernelspec_info.python_version.unwrap()
        );
    }

    #[test]
    fn test_add_to_stripped() {
        let kernelspec_info = make_sample_kernelinfo();
        let nb_bytes = notebook_with_metadata_bytes(json!({
                "language_info": {
                        "name": "python",
                    }

        }));

        let out_nb = maybe_replace_kernelspec(&nb_bytes, &kernelspec_info).unwrap();

        assert_eq!(
            out_nb.metadata.get("kernelspec"),
            Some(&kernelspec_info.kernelspec)
        );
        assert_eq!(
            out_nb
                .metadata
                .get("language_info")
                .unwrap()
                .get("version")
                .unwrap()
                .as_str()
                .unwrap(),
            kernelspec_info.python_version.unwrap()
        );
    }

    #[test]
    fn test_dont_add() {
        let kernelspec_info = make_sample_kernelinfo();
        let original_kernelspec = json!({
            "name": "python3-conda",
            "display_name": "Python 3"
        });
        let original_version = "3.8.4".to_string();
        let nb_bytes = notebook_with_metadata_bytes(json!({
            "kernelspec":original_kernelspec,
                "language_info": {
                        "name": "python",
                        "version": original_version.clone()
                    }

        }));

        let out_nb = maybe_replace_kernelspec(&nb_bytes, &kernelspec_info).unwrap();

        assert_eq!(
            out_nb.metadata.get("kernelspec"),
            Some(&original_kernelspec)
        );
        assert_eq!(
            out_nb
                .metadata
                .get("language_info")
                .unwrap()
                .get("version")
                .unwrap()
                .as_str()
                .unwrap(),
            original_version
        );
    }
    #[test]
    fn test_malformed_metadata() {
        let kernelspec_info = make_sample_kernelinfo();
        let nb_bytes_malformed_meta = notebook_with_metadata_bytes(Value::Number(10.into()));
        assert!(maybe_replace_kernelspec(&nb_bytes_malformed_meta, &kernelspec_info).is_err());
        let mut kernelspec_info_no_kernel = kernelspec_info.clone();
        kernelspec_info_no_kernel.kernelspec = Value::Null;
        assert!(
            maybe_replace_kernelspec(&nb_bytes_malformed_meta, &kernelspec_info_no_kernel).is_err()
        );
        let nb_bytes_malformed_language_info =
            notebook_with_metadata_bytes(json!({ "kernelspec": {}, "language_info": 10 }));
        assert!(
            maybe_replace_kernelspec(&nb_bytes_malformed_language_info, &kernelspec_info).is_err()
        );
    }
}
