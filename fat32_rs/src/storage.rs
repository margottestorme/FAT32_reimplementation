// Définition d'une erreur simple pour l'I/O
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    IoError,
    InvalidParameter,
    InvalidFatSignature,
}

/// Trait pour abstraire l'accès au stockage (disque)
pub trait BlockDevice {
    /// Taille standard d'un secteur FAT32
    const SECTOR_SIZE: usize = 512; 

    /// Lit un secteur (512 octets) du périphérique.
    /// `sector_index` est le LBA (Logical Block Address)
    /// Le buffer DOIT être de taille SECTOR_SIZE.
    fn read_sector(&mut self, sector_index: u32, buffer: &mut [u8]) -> Result<(), Error>;
}
