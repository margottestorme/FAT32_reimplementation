use fat32_rs::{fat_volume::FatVolume, host_device::HostDevice};
//use fat32_rs::storage::BlockDevice;
use std::io::{self, Write};

fn main() -> Result<(), fat32_rs::storage::Error> {
    let path = "fat32.img";

    let dev = HostDevice::new(path).map_err(|_| fat32_rs::storage::Error::IoError)?;
    let mut vol = FatVolume::new(dev)?;
    let mut current_dir_cluster = vol.root_cluster;

    fn change_dir(vol: &mut FatVolume<HostDevice>, current: &mut u32, dirname: &str) -> Result<(), fat32_rs::storage::Error> {
        if dirname == "/" {
            *current = vol.root_cluster;
            return Ok(());
        }

        let entries = vol.read_dir(*current)?;
        for e in entries {
            // On compare le nom normalisé exactement comme dans read_dir
            let entry_name = e.name.trim_end();
            if entry_name.eq_ignore_ascii_case(dirname) && e.is_dir {
                *current = e.first_cluster;
                return Ok(());
            }
        }

        println!("Directory not found: {}", dirname);
        Ok(())
    }

    println!("--- Info FAT32 ---");
    println!("Bytes per sector: {}", vol.bytes_per_sector);
    println!("Sectors per cluster: {}", vol.sectors_per_cluster);
    println!("Sectors per FAT: {}", vol.sectors_per_fat);
    println!("FAT start sector: {}", vol.fat_start_sector);
    println!("Data start sector: {}", vol.data_start_sector);
    println!("Root cluster: {}", vol.root_cluster);
    println!("Cluster size (bytes): {}", vol.cluster_size_bytes);

    println!("\n--- Mini-shell FAT32 ---");
    println!("Commandes disponibles : ls | cat <file> | cd <dir> | exit");

    loop {
        // Afficher le répertoire courant
        print!("FAT32:{}> ", current_dir_cluster);
        io::stdout().flush().unwrap();

        // Lire la ligne entrée
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // Séparer la commande et les arguments
        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd = parts[0].to_lowercase();

        match cmd.as_str() {
            "exit" => break,
            "ls" => {
                let entries = vol.read_dir(current_dir_cluster)?;
                for e in entries {
                    if e.is_dir {
                        println!("[DIR ] {}", e.name);
                    } else {
                        println!("[FILE] {} ({} bytes)", e.name, e.size);
                    }
                }
            },
            "cat" => {
                if parts.len() < 2 {
                    println!("Usage: cat <file>");
                    continue;
                }
                let filename = parts[1];
                match vol.read_file_by_name(current_dir_cluster, filename) {
                    Ok(content) => println!("{}", String::from_utf8_lossy(&content)),
                    Err(_) => println!("File not found: {}", filename),
                }
            },
            "cd" => {
                if parts.len() < 2 {
                    println!("Usage: cd <dir>");
                    continue;
                }
                let dirname = parts[1];
                change_dir(&mut vol, &mut current_dir_cluster, dirname)?;
            },
                _ => println!("Commande inconnue: {}", cmd),
            }
        }

    Ok(())
}
