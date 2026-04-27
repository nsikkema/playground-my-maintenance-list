//! Integration tests verifying that embedded web distribution assets match the files on disk.

use camino::{Utf8Path, Utf8PathBuf};
use flate2::Compression;
use flate2::write::GzEncoder;
use lib_web::{get_file_data, get_file_list, get_index_data};
use std::env;
use std::fs::File;
use std::io::{Read, Write};

fn get_uncompressed_data(path: &Utf8PathBuf) -> Vec<u8> {
    let metadata = path.metadata().expect("Unable to read metadata.");
    let mut file_data = vec![0; metadata.len() as usize];
    let mut file = File::open(path).expect("Unable to open file.");
    let _ = file.read(&mut file_data).expect("File buffer overflow.");

    file_data
}

#[test]
fn test_data_uncompressed() {
    let dist_path = Utf8Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("dist");

    for file_item in get_file_list().iter() {
        let local_path = dist_path.join(file_item);

        let file_data_uncompressed = get_uncompressed_data(&local_path);

        let stored_file_data_uncompressed = get_file_data(file_item).unwrap().data_uncompressed;
        assert_eq!(stored_file_data_uncompressed, file_data_uncompressed);
    }
}

#[test]
fn test_data_gzip() {
    let dist_path = Utf8Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("dist");

    for file_item in get_file_list().iter() {
        let local_path = dist_path.join(file_item);

        let file_data_uncompressed = get_uncompressed_data(&local_path);

        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(&file_data_uncompressed).unwrap();
        let file_data_compressed = encoder.finish().unwrap();

        let stored_file_data_gzip = get_file_data(file_item).unwrap().data_gzip;
        assert_eq!(stored_file_data_gzip, file_data_compressed);
    }
}

#[test]
fn test_mime_type() {
    let dist_path = Utf8Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("dist");

    for file_item in get_file_list().iter() {
        let local_path = dist_path.join(file_item);

        let mime_type = mime_guess::from_path(local_path).first_or_octet_stream();

        let stored_mime_type = get_file_data(file_item).unwrap().mime_type;
        assert_eq!(stored_mime_type, mime_type)
    }
}

#[test]
fn test_index_data() {
    let local_path = Utf8Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("dist")
        .join("index.html");

    let file_data_uncompressed = get_uncompressed_data(&local_path);

    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(&file_data_uncompressed).unwrap();
    let file_data_compressed = encoder.finish().unwrap();

    let mime_guess = mime_guess::from_path(local_path).first_or_octet_stream();

    let index_data = get_index_data();
    assert_eq!(index_data.data_uncompressed, file_data_uncompressed);
    assert_eq!(index_data.data_gzip, file_data_compressed);
    assert_eq!(index_data.mime_type, mime_guess);
}
