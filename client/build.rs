use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    thread::{self, JoinHandle},
};

use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce, aead::Aead};
use fs_extra::file::CopyOptions;
use serde::Deserialize;
use static_assertions::const_assert_eq;

// --- CONSTANTS ---

const ASSET_LIST: &str = include_str!("assets.json");
const OBFUSCATED_KEY: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/key.bin"));
const MASK: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mask.bin"));

const_assert_eq!(OBFUSCATED_KEY.len(), 32);
const_assert_eq!(MASK.len(), 32);

// --- STRUCTURES ---

#[derive(Deserialize)]
struct Hierarchy {
    #[serde(default)]
    files: Vec<String>,
    #[serde(default)]
    target_files: Vec<String>,
    #[serde(default)]
    directories: HashMap<String, Hierarchy>,
}

// --- MAIN ---

fn main() {
    let hierarchy = serde_json::de::from_str::<Hierarchy>(ASSET_LIST).unwrap();

    let mut src_path = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    src_path.push("assets");
    assert!(
        src_path.is_dir(),
        "Asset directory does not exist! (PATH:{})",
        src_path.display()
    );

    let mut dst_path = Path::new(env!("CARGO_WORKSPACE_DIR")).to_path_buf();
    dst_path.push("target");
    assert!(
        dst_path.is_dir(),
        "Target directory does not exist! (PATH:{})",
        dst_path.display()
    );

    dst_path.push("assets");
    fs_extra::dir::create(&dst_path, true).unwrap();

    let mut handles = Vec::new();
    copy_asset(
        src_path,
        dst_path,
        &hierarchy,
        &CopyOptions::default(),
        &mut handles,
    );

    for handle in handles {
        handle.join().unwrap();
    }
}

fn copy_asset(
    src: PathBuf,
    dst: PathBuf,
    hierarchy: &Hierarchy,
    options: &CopyOptions,
    handles: &mut Vec<JoinHandle<()>>,
) {
    for filename in hierarchy.files.iter() {
        let mut from = src.clone();
        from.push(filename);

        let mut to = dst.clone();
        to.push(filename);

        assert!(
            from.is_file(),
            "Could not find asset file! (PATH:{})",
            from.display()
        );

        fs_extra::file::copy(from, to, options).unwrap();
    }

    for filename in hierarchy.target_files.iter() {
        let mut from = src.clone();
        from.push(filename);

        let mut to = dst.clone();
        to.push(filename);

        assert!(
            from.is_file(),
            "Could not find asset file! (PATH:{})",
            from.display()
        );

        handles.push(thread::spawn(|| {
            let key = reconstruct_key();
            let plaintext = fs::read(from).unwrap();
            let ciphertext = encrypted_bytes(&plaintext, &key);
            fs::write(to, ciphertext).unwrap();
        }));
    }

    // 하위 디렉토리로 이동합니다.
    for (dir, node) in hierarchy.directories.iter() {
        let mut src = src.clone();
        src.push(dir);

        let mut dst = dst.clone();
        dst.push(dir);

        assert!(
            src.is_dir(),
            "Could not find asset directory! (PATH:{})",
            src.display()
        );
        fs_extra::dir::create(&dst, true).unwrap();

        // 에셋을 복사합니다.
        copy_asset(src, dst, node, options, handles);
    }
}

#[inline(never)]
fn reconstruct_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    for i in 0..32 {
        key[i] = OBFUSCATED_KEY[i] ^ MASK[i];
    }
    key
}

fn encrypted_bytes(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = rand::random::<[u8; 12]>();
    let nonce = Nonce::from_slice(nonce.as_slice());
    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref()).unwrap();
    [nonce.as_slice(), ciphertext.as_slice()].concat()
}
