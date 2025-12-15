use alloc::vec::Vec; 
use alloc::vec;
use alloc::format;
use core::convert::TryInto; // Nécessaire pour convertir le slice en tableau [u8; 4]
use crate::storage::{BlockDevice, Error};
use crate::boot_sector::Fat32BootSector;
use crate::dir_entry::DirEntry;
use crate::alloc::string::ToString;

// Valeurs spéciales de la FAT32 pour l'interprétation des entrées
const FAT32_EOC_MIN: u32 = 0x0FFFFFF8; // Début de la plage "End Of Chain"

/// Représente l'état monté d'un volume FAT32
pub struct FatVolume<D: BlockDevice> {
    pub device: D,
    // Paramètres clés extraits du Boot Sector :
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub fat_start_sector: u32,
    pub data_start_sector: u32,
    pub root_cluster: u32,
    pub cluster_size_bytes: u32,
    pub sectors_per_fat: u32,
    pub current_dir_cluster: u32,
}

impl<D: BlockDevice> FatVolume<D> {
    /// Initialise un nouveau volume en lisant le Boot Sector (LBA 0).
    pub fn new(mut device: D) -> Result<Self, Error> {
        let mut buffer = vec![0u8; D::SECTOR_SIZE as usize];

        // 1. Lire le secteur de démarrage
        device.read_sector(0, &mut buffer)?;

        // 2. Caster le buffer en structure

        // # SAFETY: 
        // - Le buffer `buffer` est un tableau de 512 octets (BlockDevice::SECTOR_SIZE), 
        //   ce qui est la taille exacte de `Fat32BootSector` (en supposant que le padding a été corrigé dans boot_sector.rs).
        // - La référence créée (`bs_ref`) est valide tant que le buffer est vivant.
        // - L'accès est en lecture seule, ce qui minimise les risques liés à l'alignement non garanti par #[repr(packed)].
        let bs_ref: &Fat32BootSector = unsafe {
            // Conversion du pointeur *const [u8; 512] en *const Fat32BootSector
            &*(buffer.as_ptr() as *const Fat32BootSector)
        };

        // 3. Validation et Calculs

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
            sectors_per_fat,
            current_dir_cluster: bs_ref.root_cluster,
        })
    }

    // ----------------------------------------------------------------------
    // ÉTAPE 5 : LOGIQUE DE NAVIGATION FAT
    // ----------------------------------------------------------------------

    /// Lit l'entrée 32 bits correspondant au cluster_id dans la FAT.
    /// Retourne la valeur de l'entrée FAT (masquée à 28 bits).
    pub fn get_next_cluster(&mut self, cluster_id: u32) -> Result<u32, Error> {
        // FAT32 utilise 4 octets par entrée
        let entry_size_bytes = 4; 
        
        if cluster_id < 2 {
            // Clusters 0 et 1 sont réservés
            return Err(Error::InvalidParameter); 
        }

        let byte_offset = cluster_id * entry_size_bytes;
        let sector_size = self.bytes_per_sector as u32;

        // 1. Calcul de l'emplacement
        let fat_sector_offset = byte_offset / sector_size;
        
        // Vérification des limites de la FAT
        if fat_sector_offset >= self.sectors_per_fat {
            return Err(Error::InvalidParameter); 
        }

        // LBA du secteur FAT à lire (en utilisant la première FAT)
        let lba = self.fat_start_sector + fat_sector_offset;
        
        // Offset en octets dans le secteur
        let sector_byte_offset = (byte_offset % sector_size) as usize;
        
        // 2. Lecture du secteur
        let mut buffer = vec![0u8; D::SECTOR_SIZE as usize];
        self.device.read_sector(lba, &mut buffer)?;
        
        // 3. Extraction de l'entrée (4 octets)
        // Note: La conversion TryInto::try_into est sûre ici car nous contrôlons les indices
        let entry_size = entry_size_bytes as usize;

        let raw_entry_bytes: [u8; 4] =
            buffer[sector_byte_offset..sector_byte_offset + entry_size]
                .try_into()
                .unwrap();


        // 4. Conversion et Masquage
        // FAT est Little Endian. Seuls les 28 bits de poids faible sont l'index de cluster.
        let fat_entry = u32::from_le_bytes(raw_entry_bytes) & 0x0FFFFFFF;

        if Self::is_eoc(fat_entry) {
            return Ok(0);
        }

        Ok(fat_entry)
    }
    
    /// Convertit un Cluster ID (>= 2) en son Secteur de Données (LBA) de départ.
    pub fn cluster_to_lba(&self, cluster_id: u32) -> Result<u32, Error> {
        if cluster_id < 2 {
            return Err(Error::InvalidParameter);
        }
        
        // LBA = DataStartSector + (ClusterID - 2) * SectorsPerCluster
        // Le cluster 2 est le premier cluster de la zone de données.
        let lba = self.data_start_sector 
                + (cluster_id - 2) * self.sectors_per_cluster as u32;
        
        Ok(lba)
    }

    /// Vérifie si l'entrée FAT retournée indique la fin d'une chaîne.
    /// Utilisé pour savoir quand arrêter la lecture d'un fichier/répertoire.
    pub fn is_eoc(fat_entry: u32) -> bool {
        fat_entry >= FAT32_EOC_MIN
    }

    /// Lit un cluster complet dans le buffer fourni.
    /// `buffer` doit être de taille `cluster_size_bytes`.
    pub fn read_cluster(&mut self, cluster_id: u32, buffer: &mut [u8]) -> Result<(), Error> {
        if cluster_id < 2 {
            return Err(Error::InvalidParameter);
        }

        let cluster_size = self.cluster_size_bytes as usize;
        if buffer.len() != cluster_size {
            return Err(Error::InvalidParameter);
        }

        let first_sector_lba = self.cluster_to_lba(cluster_id)?;
        let sectors_per_cluster = self.sectors_per_cluster as u32;

        for i in 0..sectors_per_cluster {
            let sector_lba = first_sector_lba + i;
            let start = (i * self.bytes_per_sector as u32) as usize;
            let end = start + self.bytes_per_sector as usize;

            self.device.read_sector(sector_lba, &mut buffer[start..end])?;
        }

        Ok(())
    }

    pub fn read_dir(&mut self, start_cluster: u32) -> Result<Vec<DirEntry>, Error> {
        let mut entries = Vec::new();
        let mut current_cluster = start_cluster;

        let cluster_size = self.cluster_size_bytes as usize;
        let mut buffer = vec![0u8; cluster_size];

        loop {
            self.read_cluster(current_cluster, &mut buffer)?;

            for i in 0..(cluster_size / 32) {
                let entry = &buffer[i * 32..(i + 1) * 32];

                let first_byte = entry[0];

                if first_byte == 0x00 {
                    return Ok(entries); // fin du répertoire
                }

                if first_byte == 0xE5 {
                    continue; // supprimé
                }

                let attr = entry[11];

                if attr == 0x0F {
                    continue; // LFN
                }

                let is_dir = attr & 0x10 != 0;

                let name = core::str::from_utf8(&entry[0..8])
                    .unwrap_or("")
                    .trim_end();

                let ext = core::str::from_utf8(&entry[8..11])
                    .unwrap_or("")
                    .trim_end();

                let full_name = if ext.is_empty() {
                    name.to_string()
                } else {
                    format!("{}.{}", name, ext)
                };

                let cluster_low =
                    u16::from_le_bytes(entry[26..28].try_into().unwrap()) as u32;
                let cluster_high =
                    u16::from_le_bytes(entry[20..22].try_into().unwrap()) as u32;
                let first_cluster = (cluster_high << 16) | cluster_low;

                let size =
                    u32::from_le_bytes(entry[28..32].try_into().unwrap());

                entries.push(DirEntry {
                    name: full_name,
                    first_cluster,
                    size,
                    is_dir,
                });
            }

            let next = self.get_next_cluster(current_cluster)?;
            if Self::is_eoc(next) {
                break;
            }

            current_cluster = next;
        }

        Ok(entries)
    }

    /// Lit le contenu d’un fichier sur plusieurs clusters en suivant la FAT
    pub fn read_file(
        &mut self,
        first_cluster: u32,
        file_size: u32,
    ) -> Result<Vec<u8>, Error> {
        if first_cluster < 2 {
            return Err(Error::InvalidParameter);
        }

        let mut buffer = Vec::with_capacity(file_size as usize);
        let mut current_cluster = first_cluster;

        let cluster_size = self.cluster_size_bytes as usize;
        let mut cluster_buf = vec![0u8; cluster_size];

        loop {
            // Lire le cluster courant
            self.read_cluster(current_cluster, &mut cluster_buf)?;

            // Ajouter les octets au buffer final
            if buffer.len() + cluster_size > file_size as usize {
                let remaining = file_size as usize - buffer.len();
                buffer.extend_from_slice(&cluster_buf[..remaining]);
            } else {
                buffer.extend_from_slice(&cluster_buf);
            }

            // Vérifier si on a tout lu
            if buffer.len() >= file_size as usize {
                break;
            }

            // Passer au cluster suivant via la FAT
            let next = self.get_next_cluster(current_cluster)?;
            if Self::is_eoc(next) {
                break;
            }

            current_cluster = next;
        }

        Ok(buffer)
    }

    pub fn read_file_by_name(
        &mut self,
        dir_cluster: u32,
        filename: &str,
    ) -> Result<Vec<u8>, Error> {
        let entries = self.read_dir(dir_cluster)?;

        for entry in entries {
            if entry.name.eq_ignore_ascii_case(filename) && !entry.is_dir {
                return self.read_file(entry.first_cluster, entry.size);
            }
        }

        Err(Error::InvalidParameter)
    }

    pub fn cd(&mut self, dir_name: &str) -> Result<(), Error> {
        let entries = self.read_dir(self.current_dir_cluster)?;

        for entry in entries {
            if entry.name.eq_ignore_ascii_case(dir_name) && entry.is_dir {
                self.current_dir_cluster = entry.first_cluster;
                return Ok(());
            }
        }

        Err(Error::InvalidParameter) // répertoire introuvable
    }
}