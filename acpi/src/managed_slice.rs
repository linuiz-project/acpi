use core::{
    alloc::{self, AllocError, Layout},
    mem::MaybeUninit,
    ptr::NonNull,
};

/// Thin wrapper around a regular slice, taking a reference to an allocator for automatic
/// deallocation when the slice is dropped out of scope.
#[derive(Debug)]
pub struct ManagedSlice<T, A>
where
    A: alloc::Allocator,
{
    memory: NonNull<[T]>,
    allocator: A,
}

impl<T, A> ManagedSlice<T, A>
where
    A: alloc::Allocator,
{
    pub fn new_uninit_in(len: usize, allocator: A) -> Result<ManagedSlice<MaybeUninit<T>, A>, AllocError> {
        let layout = Layout::array::<T>(len).map_err(|_| AllocError)?;
        allocator.allocate(layout).map(|ptr| ManagedSlice::<MaybeUninit<T>, A> {
            memory: NonNull::slice_from_raw_parts(ptr.as_non_null_ptr().cast(), len),
            allocator,
        })
    }
}

impl<T: Copy, A> ManagedSlice<T, A>
where
    A: alloc::Allocator,
{
    /// Attempts to allocate a new `&mut [T]` in the given allocator.
    pub fn new_in(len: usize, value: T, allocator: A) -> Result<Self, AllocError> {
        let layout = Layout::array::<T>(len).map_err(|_| AllocError)?;

        allocator
            .allocate(layout)
            .map(|ptr| NonNull::slice_from_raw_parts(ptr.as_non_null_ptr().cast::<T>(), len))
            .map(|memory| {
                // We must properly initialize the contents of the slice to avoid returning `ManagedSlice<MaybeUninit<T>>`.
                unsafe { memory.as_uninit_slice_mut().fill(core::mem::MaybeUninit::new(value)) };
                Self { memory, allocator }
            })
    }
}

impl<T, A> Drop for ManagedSlice<T, A>
where
    A: alloc::Allocator,
{
    fn drop(&mut self) {
        let ptr = self.memory.as_non_null_ptr().cast();
        let layout = Layout::array::<T>(self.len()).unwrap();

        // Safety: Caller is required to provide a slice allocated with the provided allocator.
        unsafe { self.allocator.deallocate(ptr, layout) };
    }
}

impl<T, A> core::ops::Deref for ManagedSlice<T, A>
where
    A: alloc::Allocator,
{
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        // Safety: Type always initializes memory to a valid T.
        unsafe { self.memory.as_ref() }
    }
}

impl<T, A> core::ops::DerefMut for ManagedSlice<T, A>
where
    A: alloc::Allocator,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety: Type always initializes memory to a valid T.
        unsafe { self.memory.as_mut() }
    }
}
