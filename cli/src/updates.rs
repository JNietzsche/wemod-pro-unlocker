pub fn get_latest_release() -> Option<serde_json::Value> {
    let request_url = "https://api.github.com/repos/bennett-sh/wemod-pro-unlocker/releases/latest";
    let request = minreq::Request::new(minreq::Method::Get, request_url)
        .with_header("User-Agent", "wmpu-cli");

    match request.send() {
        Ok(response) => {
            let text_response = match response.as_str() {
                Ok(text) => text,
                Err(err) => {
                    println!("failed to check for updates: {}", err);
                    return None;
                }
            };
            let json_response: serde_json::Value = match serde_json::from_str(text_response) {
                Ok(json) => json,
                Err(err) => {
                    println!("failed to check for updates: {}", err);
                    return None;
                }
            };

            return Some(json_response);
        }
        Err(err) => println!("failed to check for updates: {}", err),
    }

    None
}
