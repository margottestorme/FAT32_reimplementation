use fat32_rs::{fat_volume::FatVolume, host_device::HostDevice};

#[test]
fn can_list_root_and_read_file() {
    let dev = HostDevice::new("fat32.img")
        .expect("fat32.img missing");

    let mut vol = FatVolume::new(dev)
        .expect("Failed to mount FAT32 image");

    // Lire la racine
    let entries = vol.read_dir(vol.root_cluster)
        .expect("Failed to read root directory");

    assert!(
        entries.iter().any(|e| e.name == "HELLO.TXT"),
        "HELLO.TXT not found in root"
    );

    // Lire le fichier
    let content = vol.read_file_by_name(vol.root_cluster, "HELLO.TXT")
        .expect("Failed to read HELLO.TXT");

    assert!(
        !content.is_empty(),
        "HELLO.TXT is empty"
    );
}
