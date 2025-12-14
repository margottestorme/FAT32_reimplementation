use fat32_rs::{fat_volume::FatVolume, host_device::HostDevice};

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

    println!("--- LS / ---");

    let entries = vol.read_dir(vol.current_dir_cluster)?;

    for e in entries {
        if e.is_dir {
            println!("[DIR ] {}", e.name);
        } else {
            println!("[FILE] {} ({} bytes, cluster {})", e.name, e.size, e.first_cluster);
        }
    }

    println!("\n--- CAT HELLO.TXT ---");

    let content = vol.read_file_by_name(vol.current_dir_cluster, "HELLO.TXT")?;
    println!("{}", String::from_utf8_lossy(&content));


    println!("\n--- CAT TEST.TXT ---");

    let content = vol.read_file_by_name(vol.current_dir_cluster, "TEST.TXT")?;
    println!("{}", String::from_utf8_lossy(&content));


    println!("\n--- CAT NOPE.TXT ---");

    match vol.read_file_by_name(vol.current_dir_cluster, "NOPE.TXT") {
        Ok(_) => println!("ERROR: shouldn't exist"),
        Err(_) => println!("File not found"),
    }

    Ok(())
}
