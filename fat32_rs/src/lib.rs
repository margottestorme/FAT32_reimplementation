//! FAT32 implementation (no_std with optional std for tests/CLI)

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

// Modules communs (fonctionnent en no_std)
pub mod storage;       // BlockDevice abstraction
pub mod boot_sector;   // FAT32 BPB parsing
pub mod fat_volume;    // FAT32 volume logic

// Modules spécifiques à std (ex: fichiers hôte, images, etc.)
#[cfg(feature = "std")]
pub mod host_device;

// Exemple simple de fonction
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_works() {
        assert_eq!(add(2, 2), 4);
    }
}
