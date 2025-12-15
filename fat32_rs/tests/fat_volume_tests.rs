use fat32_rs::fat_volume::FatVolume;
use fat32_rs::storage::{BlockDevice, Error};

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

    /// Remplit un secteur avec un octet spécifique
    fn fill_sector(&mut self, sector_index: usize, byte: u8) {
        self.sectors[sector_index].fill(byte);
    }
}

impl BlockDevice for MockBlockDevice {
    fn read_sector(&mut self, sector_index: u32, buffer: &mut [u8]) -> Result<(), Error> {
        buffer.copy_from_slice(&self.sectors[sector_index as usize]);
        Ok(())
    }
}

// is_eoc
#[test]
fn is_eoc_detects_valid_eoc_low_bound() {
    assert!(FatVolume::<MockBlockDevice>::is_eoc(0x0FFFFFF8));
}

#[test]
fn is_eoc_detects_valid_eoc_high_bound() {
    assert!(FatVolume::<MockBlockDevice>::is_eoc(0x0FFFFFFF));
}

#[test]
fn is_eoc_rejects_non_eoc_value() {
    assert!(!FatVolume::<MockBlockDevice>::is_eoc(0x00000005));
}


// cluster_to_lba
#[test]
fn cluster_to_lba_root_cluster_maps_to_data_start() {
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

    assert_eq!(vol.cluster_to_lba(2).unwrap(), 10);
}

#[test]
fn cluster_to_lba_maps_following_clusters_correctly() {
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

    assert_eq!(vol.cluster_to_lba(3).unwrap(), 12);
}

#[test]
fn cluster_to_lba_rejects_invalid_cluster() {
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
    assert!(vol.cluster_to_lba(1).is_err());
}


//get_next_cluster
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

#[test]
fn get_next_cluster_returns_none_on_eoc() {
    let mut dev = MockBlockDevice::new(10);

    let fat_sector = &mut dev.sectors[1];
    fat_sector[8..12].copy_from_slice(&0x0FFFFFFF_u32.to_le_bytes());

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
    assert_eq!(next, 0); // ou None selon ton impl
}

#[test]
fn get_next_cluster_returns_none_on_free_cluster() {
    let mut dev = MockBlockDevice::new(10);

    let fat_sector = &mut dev.sectors[1];
    fat_sector[8..12].copy_from_slice(&0u32.to_le_bytes());

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
    assert_eq!(next, 0); // ou None
}


//read_cluster
#[test]
fn read_cluster_reads_correct_bytes() {
    let mut dev = MockBlockDevice::new(10);
    dev.fill_sector(2, 0xAA);
    dev.fill_sector(3, 0xBB);

    let mut vol = FatVolume {
        device: dev,
        bytes_per_sector: 512,
        sectors_per_cluster: 2,
        fat_start_sector: 1,
        data_start_sector: 2,
        root_cluster: 2,
        cluster_size_bytes: 1024,
        sectors_per_fat: 1,
        current_dir_cluster: 2,
    };

    let mut buffer = vec![0u8; 1024];
    vol.read_cluster(2, &mut buffer).unwrap();

    assert_eq!(buffer[..512], vec![0xAA; 512][..]);
    assert_eq!(buffer[512..], vec![0xBB; 512][..]);
}

#[test]
fn read_cluster_invalid_buffer_size() {
    let dev = MockBlockDevice::new(10);
    let mut vol = FatVolume {
        device: dev,
        bytes_per_sector: 512,
        sectors_per_cluster: 2,
        fat_start_sector: 1,
        data_start_sector: 2,
        root_cluster: 2,
        cluster_size_bytes: 1024,
        sectors_per_fat: 1,
        current_dir_cluster: 2,
    };
    let mut buffer = vec![0u8; 512]; // trop petit
    assert!(vol.read_cluster(2, &mut buffer).is_err());
}


//cd
#[test]
fn cd_changes_directory() {
    struct DirMock {
        sectors: Vec<[u8; 512]>,
    }

    impl DirMock {
        fn new() -> Self {
            let mut sectors = vec![[0u8; 512]; 10];
            // Cluster 2 = secteur 2
            sectors[2][0..11].copy_from_slice(b"TEST       "); // nom du dossier + padding
            sectors[2][11] = 0x10; // attribut répertoire
            sectors[2][20..22].copy_from_slice(&0u16.to_le_bytes()); // cluster_high
            sectors[2][26..28].copy_from_slice(&3u16.to_le_bytes()); // cluster_low = 3
            Self { sectors }
        }
    }

    impl BlockDevice for DirMock {
        fn read_sector(&mut self, sector_index: u32, buffer: &mut [u8]) -> Result<(), Error> {
            buffer.copy_from_slice(&self.sectors[sector_index as usize]);
            Ok(())
        }
    }

    let dev = DirMock::new();

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

    vol.cd("TEST").unwrap();
    assert_eq!(vol.current_dir_cluster, 3);
}



#[test]
fn cd_returns_error_when_not_found() {
    struct DirMock;
    impl BlockDevice for DirMock {
        fn read_sector(&mut self, _sector_index: u32, buffer: &mut [u8]) -> Result<(), Error> {
            buffer.fill(0x00); // répertoire vide
            Ok(())
        }
    }

    let dev = DirMock;
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

    assert!(vol.cd("NOTFOUND").is_err());
}