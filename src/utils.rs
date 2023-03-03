pub mod raycasting;

#[inline]
pub(crate) fn as_bytes<T>(value: &T) -> &[u8] {
    unsafe {
        let pointer = value as *const T as *const u8;
        std::slice::from_raw_parts(pointer, std::mem::size_of::<T>())
    }
}

#[inline]
pub(crate) fn as_bytes_slice<T>(slice: &[T]) -> &[u8] {
    unsafe {
        let pointer = slice.as_ptr() as *const u8;
        let length = slice.len() * std::mem::size_of::<T>();
        std::slice::from_raw_parts(pointer, length)
    }
}

#[macro_export]
macro_rules! timeit {
    ($name:expr => $block:expr) => {
        if cfg!(debug_assertions) {
            let start_instant = std::time::Instant::now();

            let result = $block;

            let duration = std::time::Instant::now() - start_instant;
            println!("{} done in {}us", $name, duration.as_micros());

            result
        } else {
            $block
        }
    };
}
