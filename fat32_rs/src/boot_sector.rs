#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Fat32BootSector {
    pub boot_jmp_code: [u8; 3],
    pub oem_name: [u8; 8],

    // Standard BPB
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sector_count: u16,
    pub fat_count: u8,
    pub root_entry_count: u16,
    pub total_sectors_16: u16,
    pub media_type: u8,
    pub sectors_per_fat_16: u16,

    pub padding_fat12_16: [u8; 12],

    // FAT32 extended BPB
    pub sectors_per_fat_32: u32,
    pub extended_flags: u16,
    pub fs_version: u16,
    pub root_cluster: u32,

    pub fs_info_sector: u16,
    pub backup_boot_sector: u16,
    pub reserved_zero: [u8; 12],

    pub drive_number: u8,
    pub reserved_1: u8,
    pub extended_boot_signature: u8,
    pub volume_id: u32,

    pub volume_label: [u8; 11],
    pub filesystem_type: [u8; 8],

    pub boot_code: [u8; 420],

    pub boot_signature: u16,
}
