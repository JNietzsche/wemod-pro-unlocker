use std::{fs, path::PathBuf};

pub fn get_all_app_bundles(extracted_resource_dir: PathBuf) -> Vec<PathBuf> {
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

pub fn get_all_vendor_bundles(extracted_resource_dir: PathBuf) -> Vec<PathBuf> {
    let mut vendor_bundles = vec![];
    let ls_result = fs::read_dir(extracted_resource_dir);

    if ls_result.is_err() {
        crate::err(format!(
            "failed to find vendor bundle files: {}",
            ls_result.unwrap_err()
        ));
        return vendor_bundles;
    }

    for file_result in ls_result.unwrap() {
        if file_result.is_err() {
            println!(
                "error while finding vendor bundle files: {}",
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

        vendor_bundles.push(file.path());
    }
    vendor_bundles
}
