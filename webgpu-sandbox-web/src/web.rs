
use crate::global::window;

pub struct CurrentQueryParameters {

}

impl CurrentQueryParameters {
    pub fn value() -> Option<String> {
        let window = window();
        let location = window.location();
        let search = location.search().ok()?;
        let search = search.trim_start_matches('?');
        Some(search.into())
    }
}
