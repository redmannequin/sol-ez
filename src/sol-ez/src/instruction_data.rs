use core::usize;

use borsh::BorshDeserialize;
use pinocchio::program_error::ProgramError;

#[derive(Debug)]
pub struct InstructionData<'data, const N: usize> {
    pub ix: [u8; N],
    pub data: &'data [u8],
}

impl<'data, const N: usize> InstructionData<'data, N> {
    pub fn new(data: &'data [u8]) -> Result<Self, ProgramError> {
        if data.len() < N {
            return Err(ProgramError::InvalidInstructionData);
        }

        let (ix, data) = data.split_at(N);
        let ix = {
            let mut data = [0; N];
            data.copy_from_slice(ix);
            data
        };

        Ok(InstructionData { ix, data })
    }

    pub fn deserialize_data<T>(mut self) -> Result<T, ProgramError>
    where
        T: BorshDeserialize,
    {
        T::deserialize(&mut self.data).map_err(|_| ProgramError::BorshIoError)
    }
}
