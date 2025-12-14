use crate::fat_volume::FatVolume;
use crate::storage::{BlockDevice, Error};

/// Mock minimal de BlockDevice pour les tests unitaires
struct MockBlockDevice {
    sectors: Vec<[u8; 512]>,
}

impl MockBlockDevice {
    fn new(sector_count: usize) -> Self {
        Self {
            sectors: vec![[0u8; 512]; sector_count],
        }
    }
}

impl BlockDevice for MockBlockDevice {
    fn read_sector(&mut self, sector_index: u32, buffer: &mut [u8]) -> Result<(), Error> {
        buffer.copy_from_slice(&self.sectors[sector_index as usize]);
        Ok(())
    }
}

#[test]
fn test_is_eoc() {
    assert!(FatVolume::<MockBlockDevice>::is_eoc(0x0FFFFFF8));
    assert!(FatVolume::<MockBlockDevice>::is_eoc(0x0FFFFFFF));
    assert!(!FatVolume::<MockBlockDevice>::is_eoc(0x00000005));
}

#[test]
fn test_cluster_to_lba() {
    let dev = MockBlockDevice::new(100);

    let vol = FatVolume {
        device: dev,
        bytes_per_sector: 512,
        sectors_per_cluster: 2,
        fat_start_sector: 1,
        data_start_sector: 10,
        root_cluster: 2,
        cluster_size_bytes: 1024,
        sectors_per_fat: 1,
        current_dir_cluster: 2,
    };

    // cluster 2 → data_start
    assert_eq!(vol.cluster_to_lba(2).unwrap(), 10);

    // cluster 3 → data_start + 2 sectors
    assert_eq!(vol.cluster_to_lba(3).unwrap(), 12);
}

#[test]
fn test_get_next_cluster_simple() {
    let mut dev = MockBlockDevice::new(10);

    // FAT entry for cluster 2 → next cluster = 3
    // FAT starts at sector 1, entry offset = cluster_id * 4
    let fat_sector = &mut dev.sectors[1];
    fat_sector[8..12].copy_from_slice(&3u32.to_le_bytes());

    let mut vol = FatVolume {
        device: dev,
        bytes_per_sector: 512,
        sectors_per_cluster: 1,
        fat_start_sector: 1,
        data_start_sector: 2,
        root_cluster: 2,
        cluster_size_bytes: 512,
        sectors_per_fat: 1,
        current_dir_cluster: 2,
    };

    let next = vol.get_next_cluster(2).unwrap();
    assert_eq!(next, 3);
}
