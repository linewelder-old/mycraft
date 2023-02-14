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
