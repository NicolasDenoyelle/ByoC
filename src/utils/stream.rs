use crate::utils::io::{IOVec, Resize};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::BTreeMap;
use std::io::{Read, Seek, SeekFrom, Write};

struct Stream<T, S>
where
    T: DeserializeOwned + Serialize,
    S: Read + Write + Seek + Resize + Clone,
{
    make_vec: fn(chunk_size: usize) -> S,
    vecs: BTreeMap<usize, IOVec<T, S>>,
}

/// Compute the next power of two or current
/// number if number is a power of two.
fn bucket_id(mut n: usize) -> usize {
    let mut i = 0usize;

    loop {
        let s = n << 1usize;
        if (s >> 1usize) == n {
            n = s;
            i += 1;
        } else {
            break;
        }
    }

    1usize + (!0usize >> i)
}
