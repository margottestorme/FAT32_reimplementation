use crate::storage::{BlockDevice, Error};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};

/// Implémentation de BlockDevice utilisant un fichier (image disque) sur l'hôte (Linux).
#[cfg(feature = "std")]
pub struct HostDevice(File);

#[cfg(feature = "std")]
impl HostDevice {
    pub fn new(path: &str) -> io::Result<Self> {
        Ok(HostDevice(File::open(path)?))
    }
}

#[cfg(feature = "std")]
impl BlockDevice for HostDevice {
    fn read_sector(&mut self, sector_index: u32, buffer: &mut [u8]) -> Result<(), Error> {
        if buffer.len() != Self::SECTOR_SIZE {
            return Err(Error::InvalidParameter);
        }
        
        let offset = sector_index as u64 * Self::SECTOR_SIZE as u64;

        // Déplace le curseur et lit
        match self.0.seek(SeekFrom::Start(offset)) {
            Ok(_) => match self.0.read_exact(buffer) {
                Ok(_) => Ok(()),
                Err(_) => Err(Error::IoError),
            },
            Err(_) => Err(Error::IoError),
        }
    }
}