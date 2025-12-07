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

### Optional (bonus)

* CLI interface
* Use of:

  * Miri
  * Kani
  * MIRAI
  * Fuzzers

---

## 🗂️ Project Structure (Recommended)

```
src/
 ├── lib.rs
 ├── block.rs       # BlockDevice trait & implementations
 ├── bpb.rs         # BIOS Parameter Block parser
 ├── fat.rs         # FAT table parsing & cluster chain following
 ├── cluster.rs     # Cluster-to-LBA logic & cluster reading
 ├── dir.rs         # Directory entry parsing (short + optional long names)
 ├── path.rs        # Path resolution, relative/absolute, ., ..
 ├── fs.rs          # High-level FileSystem struct and API
 ├── util.rs        # Helpers (endianness, errors, types)
cli/ (optional)
 tests/
```

---

## 🧩 Features

### Implemented in Phase 1 (Read-Only)

* **Boot Sector & BPB parsing**
* **FAT32 cluster chain navigation**
* **Directory entry parsing (short names)**
* **Optional:** Long File Name (LFN) support
* **Path resolution**
* **Read file content**
* **List directory contents**

### Phase 2 (Write Support – Later)

* Allocate new clusters
* Modify FAT entries
* Create directory entries
* Write file content

---

## 📐 Technical Notes

* FAT32 uses **little-endian** for all numeric fields
* Sector size must be read from BPB (typically 512 bytes)
* FAT32 End-of-Chain markers: `>= 0x0FFFFFF8`
* Directory entries are 32 bytes each
* Root directory is stored as a **cluster**, not at a fixed position
* Long File Names use multiple 0x0F entries preceding a short entry

---

## 🧪 Testing Strategy

### Unit Tests

* BPB parsing
* LBA calculations
* FAT entry parsing
* Directory parsing
* Path resolution

### Integration Tests

Use a real FAT32 image created with:

```
dd if=/dev/zero of=fat32.img bs=1M count=16
mkfs.vfat -F 32 fat32.img
```

And test:

* `ls /`
* `ls /folder/subfolder`
* `cat /file.txt`
* Relative paths: `./`, `../`, `dir/file`
* Current directory state

---

## 🚧 Build & Run

### Build (no_std library)

```
cargo build --release
```

### Run the test binaries (with std enabled in test mode)

```
cargo test
```

### Optional CLI

```
cargo run -- ls /path
cargo run -- cat /readme.txt
cargo run -- cd folder
```

---

## 🛡️ Code Safety

* Any use of `unsafe` **must** include:

  * a `/// # Safety` rustdoc section
  * a detailed explanation of invariants
* Verify with:

  * `cargo miri test`
  * Optional static analysis (Kani/MIRAI)

---

## 📦 Submission (Git Bundle)

To generate the final deliverable:

```
git bundle create fat32_project.bundle --all
```

Verify the bundle:

```
git clone fat32_project.bundle test-repo
```

Check the commit history is present and clean.

---

## 📄 Credits

If you use external code:

* Must credit in the README and inside the source file
* Must respect original license

Not crediting external code = **grade 0**.

---