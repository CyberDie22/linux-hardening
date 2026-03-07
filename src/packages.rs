use std::collections::HashMap;
use std::{fs, io};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use http_range_client::HttpReader;
use owo_colors::OwoColorize;

#[derive(Debug)]
pub struct Package {
    pub(crate) name: String,
    pub(crate) files: HashMap<String, String>,
    pub(crate) download_url: Option<String>,
}

fn get_base_url(name: &str) -> String {
    // format: us.archive.ubuntu.com_ubuntu_dists_noble_main_binary-amd64_Packages
    //         host                 _path  _dists_suite_component_binary-arch_Packages

    let parts: Vec<&str> = name.split('_').collect();

    let host = parts[0];

    let dists_idx = parts.iter().position(|&x| x == "dists").unwrap();

    let path = &parts[1..dists_idx].join("/");

    format!("https://{}/{}", host, path)
}

fn set_download_urls(packages: &mut Vec<Package>) {
    for entry in fs::read_dir("/var/lib/apt/lists").expect("Failed to read apt list directory") {
        let entry = entry.expect("Failed to read apt list file");
        let file_path = entry.path();
        let filename = file_path.file_name().unwrap().to_str().unwrap();

        if filename.ends_with("_Packages") {
            let url = get_base_url(filename);
            let package_list = File::open(&file_path).expect("Failed to open apt list file");
            let reader = BufReader::new(package_list);

            let mut package_name = String::new();
            for line in reader.lines() {
                let line = line.expect("Failed to read line from apt list file");
                if line.starts_with("Package: ") {
                    package_name = line[9..].to_string();
                } else if line.starts_with("Filename: ") {
                    let url = format!("{}/{}", url, &line[10..]);
                    if let Some(package) = packages.iter_mut().find(|p| p.name == package_name) {
                        package.download_url = Some(url);
                    }
                }
            }
        }
    }
}

pub fn get_packages() -> Vec<Package> {
    let mut packages = Vec::new();

    for file in fs::read_dir("/var/lib/dpkg/info").expect("Failed to read dpkg info directory") {
        let file = file.expect("Failed to read dpkg info file");
        let file_path = file.path();

        if file_path.is_file() && file_path.extension().map_or(false, |e| e == "list") {
            let filename = file_path.file_stem().unwrap().to_str().unwrap();
            let package_name = filename.split(":").next().unwrap_or(&filename).to_string();

            let file = File::open(&file_path).expect("Failed to open dpkg info file");
            let reader = BufReader::new(file);

            let mut package = Package {
                name: package_name,
                files: HashMap::new(),
                download_url: None,
            };

            for line in reader.lines() {
                let line = line.expect("Failed to read line from dpkg info file");
                let path = Path::new(line.trim());
                if path.is_file() {
                    let mut package_file = File::open(path).expect("Failed to open package file");

                    let mut hasher = md5::Context::new();
                    let mut buffer: [u8; 1024] = [0; 1024];
                    while let Ok(read) = package_file.read(&mut buffer) {
                        if read == 0 {
                            break;
                        }

                        hasher.consume(&buffer[..read]);
                    }

                    let hash = hasher.finalize()[..].iter().map(|b| format!("{:02x}", b)).collect::<String>();
                    package.files.insert(path.to_str().unwrap().to_string(), hash);
                }
            }

            packages.push(package);
        }
    }

    set_download_urls(&mut packages);

    packages
}

pub fn verify_package(package: &Package) -> Option<(Vec<String>, Vec<String>)> {
    let package_deb = HttpReader::new(package.download_url.clone().unwrap().replace("https", "http").as_str());

    let mut deb_archive = ar::Archive::new(package_deb);

    println!("Verifying package: {}", package.name);
    while let Some(entry_result) = deb_archive.next_entry() {
        // println!("Reading entry");
        let Ok(mut entry) = entry_result else {
            continue;
        };
        let name = str::from_utf8(entry.header().identifier()).expect("Failed to read entry name from package file");
        if !name.starts_with("control.tar") {
            io::copy(&mut entry, &mut io::sink()).expect("Failed to skip entry");
            continue;
        }

        let mut tar_archive: tar::Archive<Box<dyn io::Read>> = if name.ends_with(".zst") {
            let decoder = ruzstd::decoding::StreamingDecoder::new(entry).expect("Failed to create zstd decoder");
            tar::Archive::new(Box::new(decoder))
        } else if name.ends_with("xz") {
            let decoder = lzma_rust2::XzReader::new(entry, false);
            tar::Archive::new(Box::new(decoder))
        } else if name.ends_with("gz") {
            let decoder = flate2::read::GzDecoder::new(entry);
            tar::Archive::new(Box::new(decoder))
        } else {
            tar::Archive::new(Box::new(entry) as Box<dyn io::Read>)
        };

        for tar_entry_result in tar_archive.entries().expect("Failed to read tar archive entries") {
            let mut entry = tar_entry_result.expect("Failed to read tar archive entry");
            let path = entry.header().path().expect("Failed to read entry path from tar archive entry");

            if let Some(file_name) = path.file_name() {
                if file_name != "md5sums" {
                    continue;
                }
            } else {
                continue;
            }

            let mut buffer = Vec::with_capacity(entry.header().size().unwrap() as usize);
            io::copy(&mut entry, &mut buffer).expect("Failed to read entry data from tar archive entry");
            let md5sums = String::from_utf8(buffer).expect("Failed to read md5sums from tar archive entry");
            let md5sums = md5sums.lines().map(|line| {
                let mut split = line.split_whitespace();
                let hash = split.next().unwrap();
                let name = format!("/{}", split.next().unwrap());
                (name, hash)
            }).collect::<HashMap<_, _>>();

            let mut failed_files = Vec::new();
            let mut missed_files = Vec::new();
            for (file_name, hash) in &package.files {
                if let Some(package_hash) = md5sums.get(file_name.as_str()) {
                    if package_hash != hash {
                        failed_files.push(file_name.to_string());
                    }
                } else {
                    missed_files.push(file_name.to_string());
                }
            }
            return Some((failed_files, missed_files));
        }
    }

    println!("Failed to verify package for unknown reason: {:?}", package.name);
    None
}