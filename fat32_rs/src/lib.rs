#![no_std]
extern crate alloc;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub mod storage; // module BlockDevice
pub mod boot_sector; // module pour les structures du BPB
pub mod fat_volume; // module pour la structure principale du volume

#[cfg(feature = "std")]
pub mod host_device;

//#[cfg(test)]
//mod tests {
//    use super::*;

//    #[test]
//    fn it_works() {
//        let result = add(2, 2);
//        assert_eq!(result, 4);
//    }
//}
