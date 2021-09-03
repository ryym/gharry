use reqwest::blocking::Response;

pub fn log_error_response(url: &str, res: Response) {
    let res_text = res.text().unwrap_or_else(|_| "[unavailable]".into());
    log::warn!("request failed: {}, response body: {}", url, res_text);
}
