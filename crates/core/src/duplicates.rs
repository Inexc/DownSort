use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use rayon::prelude::*;
use sha2::{Digest, Sha256};

use crate::model::FileItem;

/// Compute SHA-256 checksum of a file streaming in 64 KB chunks.
pub fn compute_file_sha256(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];

    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    let result = hasher.finalize();
    Ok(hex::encode(result))
}

/// Compute a quick 16 KB prefix SHA-256 hash to rapidly eliminate non-identical files with the same size
/// without reading multi-gigabyte files into memory.
pub fn compute_file_prefix_hash(path: &Path) -> io::Result<[u8; 32]> {
    let mut file = File::open(path)?;
    let mut buffer = [0u8; 16384];
    let count = file.read(&mut buffer)?;
    let mut hasher = Sha256::new();
    hasher.update(&buffer[..count]);
    Ok(hasher.finalize().into())
}

/// Information about a detected group of duplicate files.
#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    pub group_id: usize,
    pub hash: String,
    pub original_file_id: usize,
    pub duplicate_file_ids: Vec<usize>,
    pub file_size: u64,
    pub wasted_size: u64, // (duplicate_count - 1) * file_size
}

/// Find duplicates among the given files.
/// 
/// Algorithmic steps:
/// 1. Group files by size. Filter out any size groups with less than 2 files or 0 bytes.
/// 2. For files with matching sizes, compute SHA-256 checksum in parallel (rayon).
/// 3. Group files with matching SHA-256 checksums.
/// 4. For each group with >= 2 files:
///    - Find the oldest file (by created timestamp, fallback to last_accessed or id).
///    - Mark oldest as `is_original = true`, `selected_for_deletion = false`.
///    - Mark remaining files as `is_original = false`, `selected_for_deletion = true`.
///    - Set `duplicate_group_id` on all items in the group.
/// 
/// Returns a list of `DuplicateGroup`.
pub fn find_and_mark_duplicates(files: &mut [FileItem]) -> Vec<DuplicateGroup> {
    // 1. Group indices by file size
    let mut size_to_indices: HashMap<u64, Vec<usize>> = HashMap::new();
    for (idx, item) in files.iter().enumerate() {
        if item.size > 0 {
            size_to_indices.entry(item.size).or_default().push(idx);
        }
    }

    // Filter candidate indices that share size with at least one other file
    let candidate_indices: Vec<usize> = size_to_indices
        .into_values()
        .filter(|indices| indices.len() >= 2)
        .flatten()
        .collect();

    if candidate_indices.is_empty() {
        return Vec::new();
    }

    // 2. Multi-stage filtering for candidate indices:
    // First, compute quick 16 KB prefix hash for candidates in parallel to discard false positives with 0 overhead
    let prefix_hashes: Vec<(usize, Result<[u8; 32], io::Error>)> = candidate_indices
        .par_iter()
        .map(|&idx| {
            let res = compute_file_prefix_hash(&files[idx].path);
            (idx, res)
        })
        .collect();

    // Group candidates by (size, prefix_hash)
    let mut prefix_map: HashMap<(u64, [u8; 32]), Vec<usize>> = HashMap::new();
    for (idx, hash_res) in prefix_hashes {
        if let Ok(p_hash) = hash_res {
            prefix_map.entry((files[idx].size, p_hash)).or_default().push(idx);
        }
    }

    // Only candidates whose prefix hash AND size match at least one other file need full hashing
    let full_hash_candidates: Vec<usize> = prefix_map
        .into_values()
        .filter(|indices| indices.len() >= 2)
        .flatten()
        .collect();

    if full_hash_candidates.is_empty() {
        return Vec::new();
    }

    // 3. Parallel compute full SHA-256 ONLY for the surviving candidates
    let hashes: Vec<(usize, Result<String, io::Error>)> = full_hash_candidates
        .par_iter()
        .map(|&idx| {
            let res = compute_file_sha256(&files[idx].path);
            (idx, res)
        })
        .collect();

    // Store hashes in file items
    for (idx, hash_res) in hashes {
        if let Ok(hash) = hash_res {
            files[idx].hash = Some(hash);
        }
    }

    // 4. Group candidates by full hash
    let mut hash_to_indices: HashMap<String, Vec<usize>> = HashMap::new();
    for &idx in &full_hash_candidates {
        if let Some(hash) = &files[idx].hash {
            hash_to_indices.entry(hash.clone()).or_default().push(idx);
        }
    }

    // 4. Mark duplicates and create DuplicateGroups
    let mut duplicate_groups = Vec::new();
    let mut next_group_id = 1;

    for (hash, mut indices) in hash_to_indices {
        if indices.len() < 2 {
            continue;
        }

        // Sort indices by created date ascending (oldest first)
        // If created is None or equal, fallback to last_accessed, then to id
        indices.sort_by(|&a, &b| {
            let file_a = &files[a];
            let file_b = &files[b];

            match (file_a.created, file_b.created) {
                (Some(ca), Some(cb)) => ca.cmp(&cb),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => match (file_a.last_accessed, file_b.last_accessed) {
                    (Some(la), Some(lb)) => la.cmp(&lb),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => file_a.id.cmp(&file_b.id),
                },
            }
        });

        let group_id = next_group_id;
        next_group_id += 1;

        let original_idx = indices[0];
        let original_id = files[original_idx].id;
        let file_size = files[original_idx].size;
        let mut duplicate_ids = Vec::new();

        // Mark original
        files[original_idx].duplicate_group_id = Some(group_id);
        files[original_idx].is_original = true;
        files[original_idx].selected_for_deletion = false;

        // Mark duplicates
        for &dup_idx in &indices[1..] {
            let dup_id = files[dup_idx].id;
            duplicate_ids.push(dup_id);
            files[dup_idx].duplicate_group_id = Some(group_id);
            files[dup_idx].is_original = false;
            files[dup_idx].selected_for_deletion = true;
        }

        let wasted_size = (indices.len() as u64 - 1) * file_size;

        duplicate_groups.push(DuplicateGroup {
            group_id,
            hash,
            original_file_id: original_id,
            duplicate_file_ids: duplicate_ids,
            file_size,
            wasted_size,
        });
    }

    duplicate_groups
}

/// Pure logic duplicate finder for testing without reading actual disk files.
/// Groups files using pre-existing file sizes and hashes.
pub fn group_duplicate_items(files: &mut [FileItem]) -> Vec<DuplicateGroup> {
    let mut hash_to_indices: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, item) in files.iter().enumerate() {
        if item.size > 0 {
            if let Some(hash) = &item.hash {
                hash_to_indices.entry(hash.clone()).or_default().push(idx);
            }
        }
    }

    let mut duplicate_groups = Vec::new();
    let mut next_group_id = 1;

    for (hash, mut indices) in hash_to_indices {
        if indices.len() < 2 {
            continue;
        }

        indices.sort_by(|&a, &b| {
            let file_a = &files[a];
            let file_b = &files[b];

            match (file_a.created, file_b.created) {
                (Some(ca), Some(cb)) => ca.cmp(&cb),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => file_a.id.cmp(&file_b.id),
            }
        });

        let group_id = next_group_id;
        next_group_id += 1;

        let original_idx = indices[0];
        let original_id = files[original_idx].id;
        let file_size = files[original_idx].size;
        let mut duplicate_ids = Vec::new();

        files[original_idx].duplicate_group_id = Some(group_id);
        files[original_idx].is_original = true;
        files[original_idx].selected_for_deletion = false;

        for &dup_idx in &indices[1..] {
            let dup_id = files[dup_idx].id;
            duplicate_ids.push(dup_id);
            files[dup_idx].duplicate_group_id = Some(group_id);
            files[dup_idx].is_original = false;
            files[dup_idx].selected_for_deletion = true;
        }

        let wasted_size = (indices.len() as u64 - 1) * file_size;

        duplicate_groups.push(DuplicateGroup {
            group_id,
            hash,
            original_file_id: original_id,
            duplicate_file_ids: duplicate_ids,
            file_size,
            wasted_size,
        });
    }

    duplicate_groups
}
