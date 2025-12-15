# FAT32_reimplementation

## 📌 Overview

This project is a **from-scratch reimplementation of a FAT32 filesystem reader** in Rust. It must operate in a **`no_std`** environment and rely only on the `alloc` crate for dynamic memory.

The goal is to provide a minimal yet functional filesystem layer capable of:

* Listing directory contents (`ls`-like)
* Navigating directories (`cd`-like)
* Reading files (`cat`-like)
* Maintaining a current working directory state

A minimal CLI interface is optional but recommended.

Writing and creating files will be implemented **in a second phase** once the read-only features are stable.

---

## 🧱 Project Requirements

### Mandatory

* Rust project using **`no_std`**
* Use of the **`alloc`** crate is allowed (and expected)
* Must run in a **Linux environment**
* Must support:

  * Listing files from a given path
  * Reading files from a given path
  * Handling relative & absolute paths
  * Keeping track of the current directory
* **Quality code required**: clear structure, idiomatic Rust, proper error handling
* **Tests are mandatory** (unit + integration)
* **Unsafe code must be fully documented** with rustdoc (`/// # Safety`) and detailed comments
* Must be **done alone**
* Must be submitted as a **Git bundle** (`git bundle create ...`)
* **Few or no commits = 0** → proper commit history required
* Any uncredited external code = **0**

## 🗂️ Project Structure           

```
src/
 ├── lib.rs       # Core library, mod declarations, no_std
 ├── storage.rs      # BlockDevice trait & simple error type
 ├── host_device.rs        # File-backed BlockDevice implementation (require std)
 ├── fat_volume.rs         # FAT32 volume logic, cluster reading, FAT navigation, read_dir, read_file
 ├── fat_volume_tests.rs      #Unit tests for FAT32 logic volume
 ├── dir_entry.rs     # Directory entry struct
 ├── boot_sector.rs         # FAT32 BIOS Parameter Block Parser
 ├── bin/       # Contains the executable binaries (CLI) for interacting with the project
    ├── cli.rs      # Mini-shell CLI for ls, cat, cd, exit
    ├── debug.rs    #Debug info/testing
tests/
 ├── integration.rs   # Integration test using a real FAT32 image 

```
---

## 🧩 Features Implemented

* **Read-only FAT32 support**
  * Boot Sector & BPB parsing
  * FAT32 cluster chain navigation
  * Directory entry parsing (short names only)
  * Read file content
  * List directory contents
  * Current working directory tracking
* **CLI** (`cli.rs`) supporting:
  * `ls`
  * `cat <file>`
  * `cd /` (change to root)
  * `exit`

---

## 🚨 Limitations

* No support for writing files or directories
* `cd ..` and relative paths (except `/`) are **not implemented**
* Only short 8.3 filenames supported
* Long File Names (LFN) are **not supported**
* Requires a pre-existing FAT32 image

---

## 🛠️ Preparing a FAT32 Image for Testing

Your  must create a FAT32 image before running `debug.rs` or `cli.rs`. For example:

```bash
# create a 16 MiB image
dd if=/dev/zero of=fat32.img bs=1M count=16

# format as FAT32
mkfs.vfat -F 32 fat32.img

# mount and add test files
mkdir mnt
sudo mount -o loop fat32.img mnt
echo "Hello" > mnt/HELLO.TXT
echo "test" > mnt/TEST.TXT
mkdir mnt/DIR
echo "directory test" > mnt/DIR/TEST_DIR.TXT
sudo umount mnt
rmdir mnt

```
---

## 🚀 Building and Running
Run CLI or debug binaries (requires std feature):
```
cargo run --features std --bin debug
cargo run --features std --bin cli
```
---

## ✅ How to test

This project requires a FAT32 disk image named `fat32.img`
placed at the root of the repository.

To run all tests:

```bash
cargo test --features std
```

## 📄 References and Credits

* **Documentation and Technical References**
  * [FAT32 (Wikipedia, FR)](https://fr.wikipedia.org/wiki/FAT32)
  * [Reference Implementation (vfat-rs)](https://github.com/FedericoPonzi/vfat-rs/tree/master)

* **Multimedia Resources**
  * [Relevant YouTube Tutorial](https://youtu.be/7o3qx66uLz8?si=O02DE-eeQRlQ3T2P)

* **Language Model (LLM) Assistance**
  * Design and debugging assistance provided by Gemini.
  * Code and documentation assistance provided by ChatGPT.

---