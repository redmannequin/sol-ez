use borsh::BorshDeserialize;
use pinocchio::program_error::ProgramError;

#[derive(Debug)]
pub struct InstructionData<'data> {
    pub ix: [u8; 4],
    pub data: &'data [u8],
}

impl<'data> InstructionData<'data> {
    pub fn new(data: &'data [u8]) -> Result<Self, ProgramError> {
        if data.len() < 4 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let (ix, data) = data.split_at(4);
        let ix = [ix[0], ix[1], ix[2], ix[3]];
        Ok(InstructionData { ix, data })
    }

    pub fn deserialize_data<T>(mut self) -> Result<T, ProgramError>
    where
        T: BorshDeserialize,
    {
        T::deserialize(&mut self.data).map_err(|_| ProgramError::BorshIoError)
    }
}
