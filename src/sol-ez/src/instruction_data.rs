use core::usize;

use borsh::BorshDeserialize;
use pinocchio::program_error::ProgramError;

use crate::split_at_fixed_unchecked;

#[derive(Debug)]
pub struct InstructionData<'data, const N: usize> {
    pub ix: &'data [u8; N],
    pub data: &'data [u8],
}

impl<'data, const N: usize> InstructionData<'data, N> {
    pub fn new(data: &'data [u8]) -> Result<Self, ProgramError> {
        if data.len() < N {
            return Err(ProgramError::InvalidInstructionData);
        }
        // SAFETY: the size of data is already checked
        let (ix, data) = unsafe { split_at_fixed_unchecked(data) };
        Ok(InstructionData { ix, data })
    }

    pub fn deserialize_data<T>(mut self) -> Result<T, ProgramError>
    where
        T: BorshDeserialize,
    {
        T::deserialize(&mut self.data).map_err(|_| ProgramError::BorshIoError)
    }
}
