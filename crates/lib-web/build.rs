//! Build script for generating embedded web distribution assets.

use camino::{Utf8Path, Utf8PathBuf};
use flate2::Compression;
use flate2::write::GzEncoder;
use glob::glob;
use phf_codegen::Map;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::process::Command;
use std::{env, fs};

#[cfg(windows)]
static NPM_CMD: &str = "npm.cmd";
#[cfg(not(windows))]
static NPM_CMD: &str = "npm";

static RUSTFMT_CMD: &str = "rustfmt";

fn install_npm_dependencies(path: &Utf8Path) {
    println!("Installing npm dependencies for libraries/web");
    let npm_exit_status = Command::new(NPM_CMD)
        .args(["install"])
        .current_dir(path)
        .status()
        .expect("npm command failed to start");

    if !npm_exit_status.success() {
        panic!("Failed to install npm dependencies for libraries/web")
    }
}

fn build_npm(path: &Utf8Path) {
    println!("Building npm project for libraries/lib-web.");
    let npm_exit_status = Command::new(NPM_CMD)
        .args(["run", "build"])
        .current_dir(path)
        .status()
        .expect("npm command failed to start");

    if !npm_exit_status.success() {
        panic!("Failed to build npm portion of libraries/lib-web")
    }
}

fn get_uncompressed_data(path: &Utf8PathBuf) -> Vec<u8> {
    let metadata = path.metadata().expect("Unable to read metadata.");
    let mut file_data = vec![0; metadata.len() as usize];
    let mut file = File::open(path).expect("Unable to open file.");
    let _ = file.read(&mut file_data).expect("File buffer overflow.");

    file_data
}

fn compress_data(data: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(data).expect("Unable to write data.");
    encoder.finish().expect("Unable to finish compression.")
}

fn generate_dist_map(path: &Utf8Path) {
    let input_path = path.join("dist");
    let input_path_str = (input_path.as_str().to_owned() + "/").replace('\\', "/");

    let output_directory_string = env::var("OUT_DIR").expect("Unable to get OUT_DIR.");
    let output_directory = Utf8Path::new(&output_directory_string);

    let data_path = output_directory.join("data");
    let data_compressed_path = output_directory.join("data-compressed");
    let output_path = output_directory.join("web_codegen.rs");
    {
        let mut output_file =
            BufWriter::new(File::create(output_path).expect("Failed to open output file."));

        let mut map = Map::new();

        let mut index_string = String::new();

        for path in glob(&format!("{input_path_str}/**/*"))
            .expect("Failed to read glob pattern.")
            .flatten()
        {
            let local_path = Utf8PathBuf::from_path_buf(path).expect("Invalid UTF-8 path.");
            if local_path.is_dir() {
                continue;
            }

            let reduced_path = local_path
                .to_string()
                .replace('\\', "/")
                .replace(input_path_str.as_str(), "");

            let new_path = data_path.join(&reduced_path);
            if let Some(new_parent_path) = new_path.parent() {
                fs::create_dir_all(new_parent_path).expect("Failed to create parent path.");
            }
            fs::copy(&local_path, data_path.join(&reduced_path))
                .expect("Failed to copy to new path.");

            let file_data_uncompressed = get_uncompressed_data(&local_path);
            let file_data_compressed = compress_data(&file_data_uncompressed);

            let new_compressed_path = data_compressed_path.join(&reduced_path);
            if let Some(new_parent_compressed_path) = new_compressed_path.parent() {
                fs::create_dir_all(new_parent_compressed_path)
                    .expect("Failed to create parent path.");
            }
            fs::write(new_compressed_path, &file_data_compressed)
                .expect("Failed to write compressed data.");

            let mime_type = mime_guess::from_path(local_path.clone()).first_or_octet_stream();

            let s = format!(
                "&Resource {{data_uncompressed: include_bytes!({:?}), data_gzip: include_bytes!({:?}), mime_type: {:?}}}",
                "data/".to_owned() + reduced_path.as_str(),
                "data-compressed/".to_owned() + reduced_path.as_str(),
                mime_type
            );

            if reduced_path == "index.html" {
                let mut chars = s.chars();
                chars.next();
                index_string = chars.as_str().to_string();
                map.entry(reduced_path, "&INDEX_DATA");
            } else {
                map.entry(reduced_path, s);
            };
        }

        if !index_string.is_empty() {
            writeln!(&mut output_file).expect("Failed to write to output file.");
            write!(
                &mut output_file,
                "const INDEX_DATA: Resource = {index_string};"
            )
            .expect("Failed to write to output file.");
        }

        writeln!(&mut output_file).expect("Failed to write to output file.");
        write!(
            &mut output_file,
            "const FILES: phf::Map<&'static str, &'static Resource> = {}",
            map.build()
        )
        .expect("Failed to write to output file.");
        writeln!(&mut output_file, ";").expect("Failed to write to output file.");
    }

    if env::var("SKIP_RUSTFMT").unwrap_or_default() != "true" {
        println!("Formatting output files");
        let rustfmt_exit_status = Command::new(RUSTFMT_CMD)
            .args(["web_codegen.rs"])
            .current_dir(output_directory)
            .status()
            .expect("Failed to format web_codegen.rs.");

        if !rustfmt_exit_status.success() {
            panic!("Failed to format output of libraries/lib-web")
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=public");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=index.html");
    println!("cargo:rerun-if-changed=package.json");
    println!("cargo:rerun-if-changed=tsconfig.app.json");
    println!("cargo:rerun-if-changed=tsconfig.json");
    println!("cargo:rerun-if-changed=vite.config.ts");

    let current_path_string =
        env::var("CARGO_MANIFEST_DIR").expect("Unable to get current Cargo path.");
    let current_path = Utf8Path::new(&current_path_string);

    if env::var("SKIP_NPM_BUILD").unwrap_or_default() != "true" {
        install_npm_dependencies(current_path);
        build_npm(current_path);
    }

    generate_dist_map(current_path);
}
