use alloc::vec::Vec;
use crate::storage::{BlockDevice, Error};
use crate::boot_sector::Fat32BootSector;

/// Représente l'état monté d'un volume FAT32
pub struct FatVolume<D: BlockDevice> {
    device: D,
    // Paramètres clés extraits du Boot Sector :
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub fat_start_sector: u32,
    pub data_start_sector: u32,
    pub root_cluster: u32,
    pub cluster_size_bytes: u32,
    // ...
}

impl<D: BlockDevice> FatVolume<D> {
    /// Initialise un nouveau volume en lisant le Boot Sector (LBA 0).
    pub fn new(mut device: D) -> Result<Self, Error> {
        let mut buffer = [0u8; BlockDevice::SECTOR_SIZE];

        // 1. Lire le secteur de démarrage
        device.read_sector(0, &mut buffer)?;

        // 2. Caster le buffer en structure

        // # SAFETY: 
        // - Le buffer `buffer` est un tableau de 512 octets (BlockDevice::SECTOR_SIZE), 
        //   ce qui est la taille exacte de `Fat32BootSector` si elle est correctement définie 
        //   avec #[repr(packed)].
        // - La référence créée (`bs_ref`) est valide tant que le buffer est vivant.
        // - L'alignement n'est pas garanti par `#[repr(packed)]`, mais l'accès est en lecture 
        //   seule, et l'architecture cible (Linux x86/ARM) supporte généralement ces accès.
        let bs_ref: &Fat32BootSector = unsafe {
            // Conversion du pointeur *const [u8; 512] en *const Fat32BootSector
            // La bonne pratique serait d'utiliser une crate pour la désérialisation 
            // sûre (ex: `zerocopy`), mais ce cast direct est courant dans les projets no_std.
            &*(buffer.as_ptr() as *const Fat32BootSector)
        };

        // 3. Validation et Calculs

        // Les données sont stockées en Little Endian (convention FAT), 
        // bs_ref.boot_signature est donc 0x55AA sur Little Endian si le fichier est correct.
        if bs_ref.boot_signature != 0xAA55 {
            return Err(Error::InvalidFatSignature); 
        }

        let fat_count = bs_ref.fat_count as u32; // Généralement 2
        let sectors_per_fat = bs_ref.sectors_per_fat_32;
        let reserved_sectors = bs_ref.reserved_sector_count as u32;

        let fat_start_sector = reserved_sectors;

        // Data Area commence après les secteurs réservés et toutes les tables FAT
        let data_start_sector = fat_start_sector + (fat_count * sectors_per_fat);

        let cluster_size_bytes = bs_ref.bytes_per_sector as u32 * bs_ref.sectors_per_cluster as u32;

        Ok(FatVolume {
            device,
            bytes_per_sector: bs_ref.bytes_per_sector,
            sectors_per_cluster: bs_ref.sectors_per_cluster,
            fat_start_sector,
            data_start_sector,
            root_cluster: bs_ref.root_cluster,
            cluster_size_bytes,
        })
    }
}