use fat32_rs::{fat_volume::FatVolume, host_device::HostDevice};
use fat32_rs::storage::BlockDevice;

fn main() -> Result<(), fat32_rs::storage::Error> {
    let path = "fat32.img";

    let dev = HostDevice::new(path).map_err(|_| fat32_rs::storage::Error::IoError)?;
    let mut vol = FatVolume::new(dev)?;

    println!("--- Info FAT32 ---");
    println!("Bytes per sector: {}", vol.bytes_per_sector);
    println!("Sectors per cluster: {}", vol.sectors_per_cluster);
    println!("Sectors per FAT: {}", vol.sectors_per_fat);
    println!("FAT start sector: {}", vol.fat_start_sector);
    println!("Data start sector: {}", vol.data_start_sector);
    println!("Root cluster: {}", vol.root_cluster);
    println!("Cluster size (bytes): {}", vol.cluster_size_bytes);

    // Lire le cluster racine
    let root_cluster_id = vol.root_cluster;
    let root_lba = vol.cluster_to_lba(root_cluster_id)?;
    println!("Root cluster LBA start: {}", root_lba);

    let mut cluster_data = vec![0u8; vol.cluster_size_bytes as usize];
    for i in 0..vol.sectors_per_cluster as u32 {
        vol.device.read_sector(
            root_lba + i,
            &mut cluster_data[(i as usize) * vol.bytes_per_sector as usize..((i+1) as usize) * vol.bytes_per_sector as usize]
        )?;
    }

    // Parcourir les entrées du répertoire
    for i in 0..(cluster_data.len() / 32) {
        let entry = &cluster_data[i*32..(i+1)*32];
        let first = entry[0];
        if first == 0x00 { break; }
        if first == 0xE5 { continue; }
        if first < 0x20 { continue; }

        let attr = entry[11];
        if attr == 0x0F { continue; } // LFN

        let name = std::str::from_utf8(&entry[0..8]).unwrap_or("").trim_end();
        let ext  = std::str::from_utf8(&entry[8..11]).unwrap_or("").trim_end();

        if ext.is_empty() {
            println!("Found entry: {}", name);
        } else {
            println!("Found entry: {}.{}", name, ext);
        }
    }

    Ok(())
}
