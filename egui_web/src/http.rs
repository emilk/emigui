use wasm_bindgen::prelude::*;

pub use epi::http::{Request, Response};

/// NOTE: Ok(..) is returned on network error.
/// Err is only for failure to use the fetch api.
pub async fn fetch_async(request: &Request) -> Result<Response, String> {
    fetch_jsvalue(request)
        .await
        .map_err(|err| err.as_string().unwrap_or(format!("{:#?}", err)))
}

/// NOTE: Ok(..) is returned on network error.
/// Err is only for failure to use the fetch api.
async fn fetch_jsvalue(request: &Request) -> Result<Response, JsValue> {
    // https://rustwasm.github.io/wasm-bindgen/examples/fetch.html
    // https://github.com/seanmonstar/reqwest/blob/master/src/wasm/client.rs

    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let mut opts = web_sys::RequestInit::new();
    opts.method(&request.method);
    opts.mode(web_sys::RequestMode::Cors);

    if !request.body.is_empty() {
        let body_bytes: &[u8] = &request.body;
        let body_array: js_sys::Uint8Array = body_bytes.into();
        let js_value: &JsValue = body_array.as_ref();
        opts.body(Some(js_value));
    }

    let js_request = web_sys::Request::new_with_str_and_init(&request.url, &opts)?;

    for h in &request.headers {
        js_request.headers().set(h.0, h.1)?;
    }

    let window = web_sys::window().unwrap();
    let response = JsFuture::from(window.fetch_with_request(&js_request)).await?;
    let response: web_sys::Response = response.dyn_into()?;

    let array_buffer = JsFuture::from(response.array_buffer()?).await?;
    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let bytes = uint8_array.to_vec();

    // https://developer.mozilla.org/en-US/docs/Web/API/Headers
    // "Note: When Header values are iterated over, [...] values from duplicate header names are combined."
    let mut headers = std::collections::BTreeMap::<String, String>::new();
    let js_headers: web_sys::Headers = response.headers();
    let js_iter = js_sys::try_iter(&js_headers)
        .expect("headers try_iter")
        .expect("headers have an iterator");

    for item in js_iter {
        let item = item.expect("headers iterator");
        let array: js_sys::Array = item.into();
        let v: Vec<JsValue> = array.to_vec();

        let mut key = v[0]
            .as_string()
            .ok_or_else(|| JsValue::from_str("headers name"))?;
        let value = v[1]
            .as_string()
            .ok_or_else(|| JsValue::from_str("headers value"))?;

        // for easy lookup
        key.make_ascii_lowercase();
        headers.insert(key, value);
    }

    Ok(Response {
        url: response.url(),
        ok: response.ok(),
        status: response.status(),
        status_text: response.status_text(),
        bytes,
        headers,
    })
}

// ----------------------------------------------------------------------------

pub(crate) struct WebHttp {}

impl epi::backend::Http for WebHttp {
    fn fetch_dyn(
        &self,
        request: Request,
        on_done: Box<dyn FnOnce(Result<Response, String>) + Send>,
    ) {
        crate::spawn_future(async move {
            let result = crate::http::fetch_async(&request).await;
            on_done(result)
        });
    }
}
