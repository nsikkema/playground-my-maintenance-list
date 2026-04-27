//! Embedded web distribution assets for the maintenance list application.

/// A static web resource containing both uncompressed and gzip-compressed data.
#[derive(Debug)]
pub struct Resource {
    /// The raw, uncompressed bytes of the resource.
    pub data_uncompressed: &'static [u8],
    /// The gzip-compressed bytes of the resource.
    pub data_gzip: &'static [u8],
    /// The MIME type of the resource (e.g. `"text/html"`, `"application/javascript"`).
    pub mime_type: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/web_codegen.rs"));

#[cfg(feature = "list")]
use itertools::Itertools;

/// Returns the static [`Resource`] for the given file path, or `None` if not found.
pub fn get_file_data(file_path: &str) -> Option<&'static Resource> {
    if let Some(file_data) = FILES.get(file_path) {
        Some(file_data)
    } else {
        None
    }
}

/// Returns the static [`Resource`] for the root `index.html` file.
pub fn get_index_data() -> &'static Resource {
    &INDEX_DATA
}

/// Returns a sorted list of all embedded file paths.
#[cfg(feature = "list")]
pub fn get_file_list() -> Vec<&'static &'static str> {
    FILES.keys().sorted().collect_vec()
}
