mod concurrent;
use std::thread;

#[test]
fn test_clone() {
    let v = concurrent::clone::CloneMut::new(96u32);

    for i in 0..64 {
        let mut vc = v.clone();
        thread::spawn(move || {
            *vc = i as u32;
        });
    }

    assert!(*v < 64u32);
}
