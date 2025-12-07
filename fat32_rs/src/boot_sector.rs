// L'attribut `packed` est nécessaire pour lire les structures binaires du disque
// sans que le compilateur n'insère de padding pour l'alignement.
#[repr(packed)] 
#[derive(Debug, Clone, Copy)]
pub struct Fat32BootSector {
    // 0x00 - 0x0A (11 octets)
    pub boot_jmp_code: [u8; 3],
    pub oem_name: [u8; 8],
    
    // --- BPB (BIOS Parameter Block) STANDARD ---
    // 0x0B - 0x17 (13 octets)
    pub bytes_per_sector: u16,      // 0x0B
    pub sectors_per_cluster: u8,    // 0x0D
    pub reserved_sector_count: u16, // 0x0E
    pub fat_count: u8,              // 0x10
    pub root_entry_count: u16,      // 0x11 (Toujours 0 pour FAT32)
    pub total_sectors_16: u16,      // 0x13 (Toujours 0 pour FAT32)
    pub media_type: u8,             // 0x15
    pub sectors_per_fat_16: u16,    // 0x16 (Toujours 0 pour FAT32)
    
    // 0x18 - 0x23 (12 octets) : Padding/Champs FAT12/16 (non utilisés pour FAT32, ou ignorés)
    pub padding_fat12_16: [u8; 12],
    
    // --- BPB FAT32 Étendu (Extended BPB) ---
    // 0x24 - 0x27 (4 octets)
    pub sectors_per_fat_32: u32,    // 0x24 (Le champ critique pour FAT32)
    
    // 0x28 - 0x2B (4 octets)
    pub extended_flags: u16,        // 0x28
    pub fs_version: u16,            // 0x2A
    
    // 0x2C - 0x2F (4 octets)
    pub root_cluster: u32,          // 0x2C (Cluster de départ du répertoire racine)
    
    // 0x30 - 0x47 (24 octets) : FSInfo Sector, Backup Boot Sector, et padding
    pub fs_info_sector: u16,        // 0x30
    pub backup_boot_sector: u16,    // 0x32
    pub reserved_zero: [u8; 12],    // 0x34
    
    // 0x40 - 0x47 (8 octets) : Le reste du BPB étendu (drive number, signature)
    pub drive_number: u8,           // 0x40
    pub reserved_1: u8,             // 0x41
    pub extended_boot_signature: u8,// 0x42 (Doit être 0x29)
    pub volume_id: u32,             // 0x43
    
    // 0x47 - 0x51 (11 octets)
    pub volume_label: [u8; 11],     // 0x47
    
    // 0x52 - 0x5A (8 octets)
    pub filesystem_type: [u8; 8],   // 0x52 ("FAT32   ")
    
    // --- PADDING FINAL ---
    // Le secteur fait 512 octets. La signature finale est à 0x1FE (510ème octet).
    // On a utilisé 90 octets (0x5A + 1). 510 - 90 = 420 octets restants.
    pub boot_code: [u8; 420],       // 0x5B - 0x1FD
    
    // --- SIGNATURE DE FIN ---
    // 0x1FE - 0x1FF (2 octets)
    pub boot_signature: u16,        // 0x1FE (Doit être 0xAA55)
}