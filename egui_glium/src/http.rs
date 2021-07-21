use std::collections::BTreeMap;

pub use epi::http::{Request, Response};

/// NOTE: Ok(..) is returned on network error.
/// Err is only for failure to use the fetch api.
pub fn fetch_blocking(request: &Request) -> Result<Response, String> {
    let mut req = ureq::request(&request.method, &request.url);

    for header in &request.headers {
        req = req.set(header.0, header.1);
    }

    let resp = if request.body.is_empty() {
        req.call()
    } else {
        req.send_bytes(&request.body)
    };

    let (ok, resp) = match resp {
        Ok(resp) => (true, resp),
        Err(ureq::Error::Status(_, resp)) => (false, resp), // Still read the body on e.g. 404
        Err(ureq::Error::Transport(error)) => return Err(error.to_string()),
    };

    let url = resp.get_url().to_owned();
    let status = resp.status();
    let status_text = resp.status_text().to_owned();
    let mut headers = BTreeMap::new();
    for key in &resp.headers_names() {
        if let Some(value) = resp.header(key) {
            // lowercase for easy lookup
            headers.insert(key.to_ascii_lowercase(), value.to_owned());
        }
    }

    let mut reader = resp.into_reader();
    let mut bytes = vec![];
    use std::io::Read;
    reader
        .read_to_end(&mut bytes)
        .map_err(|err| err.to_string())?;

    let response = Response {
        url,
        ok,
        status,
        status_text,
        bytes,
        headers,
    };
    Ok(response)
}

// ----------------------------------------------------------------------------

pub(crate) struct GliumHttp {}

impl epi::backend::Http for GliumHttp {
    fn fetch_dyn(
        &self,
        request: Request,
        on_done: Box<dyn FnOnce(Result<Response, String>) + Send>,
    ) {
        std::thread::spawn(move || {
            let result = crate::http::fetch_blocking(&request);
            on_done(result)
        });
    }
}
