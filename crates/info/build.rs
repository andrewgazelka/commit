// Copyright (c) 2025 Andrew Gazelka
// SPDX-License-Identifier: MIT

use sha2::{Digest, Sha256};
use std::io::Write;

fn main() {
    // Discover the repository starting from the current directory
    let repo = match git2::Repository::discover(".") {
        Ok(repo) => repo,
        Err(_) => {
            // Not in a git repository, use default values
            println!(
                "cargo:rustc-env=GIT_COMMIT=0000000000000000000000000000000000000000000000000000000000000000"
            );
            println!("cargo:rustc-env=GIT_DIRTY=false");
            println!("cargo:rustc-env=GIT_HISTORY_LEN=0");

            // Write empty history file
            let out_dir = std::env::var("OUT_DIR").unwrap();
            let history_path = std::path::Path::new(&out_dir).join("history.bin");
            std::fs::write(history_path, &[]).unwrap();
            return;
        }
    };

    // Get the HEAD commit hash
    let head = repo.head().ok();

    // Collect commit history (SHA-256 hashes of git commit IDs)
    let history_hashes: Vec<[u8; 32]> = if let Some(head_ref) = &head {
        if let Ok(commit) = head_ref.peel_to_commit() {
            let mut revwalk = repo.revwalk().unwrap();
            revwalk.push(commit.id()).unwrap();

            revwalk
                .filter_map(|oid| oid.ok())
                .map(|oid| {
                    // Hash the git commit ID with SHA-256
                    let mut hasher = Sha256::new();
                    hasher.update(oid.as_bytes());
                    let result = hasher.finalize();
                    let mut hash = [0u8; 32];
                    hash.copy_from_slice(&result);
                    hash
                })
                .collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Write history as raw bytes to a file
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let history_path = std::path::Path::new(&out_dir).join("history.bin");
    let mut file = std::fs::File::create(&history_path).unwrap();
    for hash in &history_hashes {
        file.write_all(hash).unwrap();
    }

    // Check if the working directory is dirty
    let mut status_options = git2::StatusOptions::new();
    status_options.include_untracked(true);

    let dirty = repo
        .statuses(Some(&mut status_options))
        .map(|statuses| !statuses.is_empty())
        .unwrap_or(false);

    // Get current commit hash (first in history)
    let current_hash = history_hashes
        .first()
        .map(|h| hex::encode(h))
        .unwrap_or_else(|| {
            "0000000000000000000000000000000000000000000000000000000000000000".to_string()
        });

    println!("cargo:rustc-env=GIT_COMMIT={}", current_hash);
    println!("cargo:rustc-env=GIT_DIRTY={}", dirty);
    println!("cargo:rustc-env=GIT_HISTORY_LEN={}", history_hashes.len());

    // Rerun if git state changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
}
