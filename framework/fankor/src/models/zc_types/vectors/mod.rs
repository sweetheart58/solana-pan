pub use fnk::*;

mod fnk;

use crate::errors::FankorResult;
use crate::models::{ZCMut, ZeroCopyType, ZC};
use borsh::BorshDeserialize;
use solana_program::account_info::AccountInfo;

impl<T: ZeroCopyType> ZeroCopyType for Vec<T> {
    fn byte_size_from_instance(&self) -> usize {
        let mut size = 0;

        let len = self.len() as u32;
        size += len.byte_size_from_instance();

        for v in self {
            size += v.byte_size_from_instance();
        }

        size
    }

    fn byte_size(mut bytes: &[u8]) -> FankorResult<usize> {
        let mut size = 0;

        let bytes2 = &mut bytes;
        let len = u32::deserialize(bytes2)?;
        size += len as usize;

        for _ in 0..len {
            bytes = &bytes[size..];
            size += T::byte_size(bytes)?;
        }

        Ok(size)
    }
}

impl<'info, 'a, T: ZeroCopyType> ZC<'info, 'a, Vec<T>> {
    // GETTERS ----------------------------------------------------------------

    /// The length of the vector.
    pub fn len(&self) -> FankorResult<usize> {
        let bytes = (*self.info.data).borrow();
        let mut bytes = &bytes[self.offset..];
        let len = u32::deserialize(&mut bytes)?;

        Ok(len as usize)
    }

    /// Whether the vector is empty or not
    pub fn is_empty(&self) -> FankorResult<bool> {
        Ok(self.len()? == 0)
    }

    // METHODS ----------------------------------------------------------------

    /// Gets the element at the specified position.
    pub fn get_zc_index(&self, index: usize) -> FankorResult<Option<ZC<'info, 'a, T>>> {
        let bytes = (*self.info.data).borrow();
        let mut bytes = &bytes[self.offset..];
        let initial_size = bytes.len();

        let len = u32::deserialize(&mut bytes)?;

        let index = index as u32;
        if index >= len {
            return Ok(None);
        }

        for i in 0..len {
            if i == index {
                return Ok(Some(ZC {
                    info: self.info,
                    offset: self.offset + initial_size - bytes.len(),
                    _data: std::marker::PhantomData,
                }));
            }

            let size = T::byte_size(bytes)?;
            bytes = &bytes[size..];
        }

        Ok(None)
    }

    pub fn iter(&self) -> Iter<'info, 'a, T> {
        Iter {
            info: self.info,
            offset: self.offset,
            len: self
                .len()
                .expect("Failed to get length of zc-vector in iterator"),
            index: 0,
            _data: std::marker::PhantomData,
        }
    }
}

impl<'info, 'a, T: ZeroCopyType> ZCMut<'info, 'a, Vec<T>> {
    // METHODS ----------------------------------------------------------------

    /// Gets the element at the specified position.
    pub fn get_mut_zc_index(&mut self, index: usize) -> FankorResult<Option<ZCMut<'info, 'a, T>>> {
        let bytes = (*self.info.data).borrow();
        let mut bytes = &bytes[self.offset..];
        let initial_size = bytes.len();

        let len = u32::deserialize(&mut bytes)?;

        let index = index as u32;
        if index >= len {
            return Ok(None);
        }

        for i in 0..len {
            if i == index {
                return Ok(Some(ZCMut {
                    info: self.info,
                    offset: self.offset + initial_size - bytes.len(),
                    _data: std::marker::PhantomData,
                }));
            }

            let size = T::byte_size(bytes)?;
            bytes = &bytes[size..];
        }

        Ok(None)
    }

    pub fn iter_mut(&mut self) -> IterMut<'info, 'a, T> {
        IterMut {
            info: self.info,
            offset: self.offset,
            len: self
                .to_ref()
                .len()
                .expect("Failed to get length of zc-vector in iterator"),
            index: 0,
            _data: std::marker::PhantomData,
        }
    }
}

impl<'info, 'a, T: ZeroCopyType> IntoIterator for ZC<'info, 'a, Vec<T>> {
    type Item = ZC<'info, 'a, T>;
    type IntoIter = Iter<'info, 'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            info: self.info,
            offset: self.offset,
            len: self
                .len()
                .expect("Failed to get length of zc-vector in iterator"),
            index: 0,
            _data: std::marker::PhantomData,
        }
    }
}

impl<'info, 'a, T: ZeroCopyType> IntoIterator for ZCMut<'info, 'a, Vec<T>> {
    type Item = ZCMut<'info, 'a, T>;
    type IntoIter = IterMut<'info, 'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            info: self.info,
            offset: self.offset,
            len: self
                .to_ref()
                .len()
                .expect("Failed to get length of zc-vector in iterator"),
            index: 0,
            _data: std::marker::PhantomData,
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct Iter<'info, 'a, T: ZeroCopyType> {
    pub(crate) info: &'info AccountInfo<'info>,
    pub(crate) len: usize,
    pub(crate) index: usize,
    pub(crate) offset: usize,
    pub(crate) _data: std::marker::PhantomData<(T, &'a ())>,
}

impl<'info, 'a, T: ZeroCopyType> Iterator for Iter<'info, 'a, T> {
    type Item = ZC<'info, 'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            return None;
        }

        let result = ZC {
            info: self.info,
            offset: self.offset,
            _data: std::marker::PhantomData,
        };

        let bytes = (*self.info.data).borrow();
        let bytes = &bytes[self.offset..];

        self.offset += T::byte_size(bytes).expect("Deserialization failed in vector iterator");
        self.index += 1;

        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.len - self.index;

        (size, Some(size))
    }
}

impl<'info, 'a, T: ZeroCopyType> ExactSizeIterator for Iter<'info, 'a, T> {}

pub struct IterMut<'info, 'a, T: ZeroCopyType> {
    pub(crate) info: &'info AccountInfo<'info>,
    pub(crate) len: usize,
    pub(crate) index: usize,
    pub(crate) offset: usize,
    pub(crate) _data: std::marker::PhantomData<(T, &'a mut ())>,
}

impl<'info, 'a, T: ZeroCopyType> Iterator for IterMut<'info, 'a, T> {
    type Item = ZCMut<'info, 'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            return None;
        }

        let result = ZCMut {
            info: self.info,
            offset: self.offset,
            _data: std::marker::PhantomData,
        };

        let bytes = (*self.info.data).borrow();
        let bytes = &bytes[self.offset..];

        self.offset += T::byte_size(bytes).expect("Deserialization failed in vector iterator");
        self.index += 1;

        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.len - self.index;

        (size, Some(size))
    }
}

impl<'info, 'a, T: ZeroCopyType> ExactSizeIterator for IterMut<'info, 'a, T> {}
