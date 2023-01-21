use std::{collections::HashMap, fs, path::PathBuf};

pub fn patch_pro_mode(extracted_resource_dir: PathBuf, opts: &HashMap<String, String>) {
    for app_bundle in get_all_app_bundles(extracted_resource_dir) {
        let contents_result = fs::read_to_string(&app_bundle);

        if contents_result.is_err() {
            println!(
                "error while reading possible app bundle file: {}",
                contents_result.unwrap_err()
            );
            continue;
        }

        let contents = contents_result.unwrap();

        if contents.contains(r#""application/json"===e.headers.get("Content-Type")"#) {
            let app_bundle_patch = include_str!("fetchIntercept.js").to_string().replace(
                "/*{%account%}*/",
                if opts.contains_key("account") {
                    opts.get("account").unwrap()
                } else {
                    ""
                },
            );
            let app_bundle_original_code =
              "return\"application/json\"===e.headers.get(\"Content-Type\")?await e.json():await e.text()";

            if !contents.contains(app_bundle_original_code) {
                crate::err(
                    "failed to enable pro mode. WeMod may have updated their program".to_string(),
                );
            }

            let app_bundle_contents_patched =
                contents.replace(app_bundle_original_code, app_bundle_patch.as_str());

            match fs::write(&app_bundle, app_bundle_contents_patched) {
                Ok(_) => break,
                Err(err) => println!("failed to enable pro mode (write): {}", err),
            };
        }
    }
}

fn get_all_app_bundles(extracted_resource_dir: PathBuf) -> Vec<PathBuf> {
    let mut app_bundles = vec![];
    let ls_result = fs::read_dir(extracted_resource_dir);

    if ls_result.is_err() {
        crate::err(format!(
            "failed to find app bundle file: {}",
            ls_result.unwrap_err()
        ));
        return app_bundles;
    }

    for file_result in ls_result.unwrap() {
        if file_result.is_err() {
            println!(
                "error while finding app bundle file: {}",
                file_result.unwrap_err()
            );
            continue;
        }
        let file = file_result.unwrap();
        let file_name_os = &file.file_name();
        let file_name = file_name_os.to_str().unwrap();

        if !(file_name.starts_with("app-") && file_name.ends_with(".js")) {
            continue;
        }

        app_bundles.push(file.path());
    }

    app_bundles
}

pub fn patch_creator_mode(extracted_resource_dir: PathBuf) {
    for app_bundle in get_all_app_bundles(extracted_resource_dir) {
        let contents_result = fs::read_to_string(&app_bundle);

        if contents_result.is_err() {
            println!(
                "error while reading possible app bundle file: {}",
                contents_result.unwrap_err()
            );
            continue;
        }

        let contents = contents_result.unwrap();

        if contents.contains("get isCreator(){") {
            println!("creator");
            match fs::write(
                &app_bundle,
                contents.replace("get isCreator(){", "get isCreator(){return true;"),
            ) {
                Ok(_) => {}
                Err(err) => println!("failed to patch creator mode: {}", err),
            };
        }
    }
}

pub fn patch_vendor_bundle(extracted_resource_dir: PathBuf) {
    let ls_result = fs::read_dir(extracted_resource_dir);

    if ls_result.is_err() {
        crate::err(format!(
            "failed to find vendor bundle file: {}",
            ls_result.unwrap_err()
        ));
        return;
    }

    for file_result in ls_result.unwrap() {
        if file_result.is_err() {
            println!(
                "error while finding vendor bundle file: {}",
                file_result.unwrap_err()
            );
            continue;
        }
        let file = file_result.unwrap();
        let file_name_os = &file.file_name();
        let file_name = file_name_os.to_str().unwrap();

        if !(file_name.starts_with("vendors-") && file_name.ends_with(".js")) {
            continue;
        }

        let contents_result = fs::read_to_string(&file.path());

        if contents_result.is_err() {
            println!(
                "error while reading possible vendor bundle file: {}",
                contents_result.unwrap_err()
            );
            continue;
        }

        let mut contents = contents_result.unwrap();

        let vendor_bundle_patch = include_str!("vendorPatch.js")
            .to_string()
            .replace("/*{%version%}*/", crate::VERSION);

        contents.insert_str(0, &vendor_bundle_patch);

        let write_result = fs::write(file.path(), contents);

        if write_result.is_err() {
            println!(
                "error while writing vendor bundle file: {}",
                write_result.unwrap_err()
            );
            continue;
        }

        break;
    }
}
