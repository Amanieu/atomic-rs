extern crate autocfg;

fn main() {
    let ac = autocfg::new();

    for root in &["core", "std"] {
        ac.emit_path_cfg(
            &format!("{}::sync::atomic::AtomicUsize", root),
            "has_atomic_usize",
        );
        ac.emit_path_cfg(
            &format!("{}::sync::atomic::AtomicIsize", root),
            "has_atomic_isize",
        );

        for size in &[8, 16, 32, 64] {
            ac.emit_path_cfg(
                &format!("{}::sync::atomic::AtomicU{}", root, size),
                &format!("has_atomic_u{}", size),
            );
            ac.emit_path_cfg(
                &format!("{}::sync::atomic::AtomicI{}", root, size),
                &format!("has_atomic_i{}", size),
            );
        }
    }
}
