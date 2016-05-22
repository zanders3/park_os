use io::ide_disk::IdeDisk;
use io::membuffer::MemBuffer;
use core::marker::PhantomData;

#[derive(Debug)]
pub enum FSType {
	Unsupported,
	Fat12,
	Fat16
}

pub struct FatFS<'l> {
	pub fs_type:FSType,
	fat_sector: u16,
	root_dir_sector: u16,
	disk: &'l mut IdeDisk
}

impl<'l> FatFS<'l> {
	pub fn init_fs(disk:&mut IdeDisk) -> Result<FatFS, &'static str> {
		let mut boot_sector = MemBuffer::new();
		let size = try!(disk.read(0, &mut boot_sector));

		let sector_length = boot_sector.get_u16(11);//=512
		let sectors_per_cluster = boot_sector.get_u8(13) as u16;
		let num_reserved_sectors = boot_sector.get_u16(14);//=1
		let num_fats = boot_sector.get_u8(16) as u16;//=2
		let root_entry_count = boot_sector.get_u16(17);
		let total_sectors = boot_sector.get_u16(19);
		let fat_size = boot_sector.get_u16(22);//=2

		let num_root_dir_sectors = ((root_entry_count * 32) + (sector_length - 1)) / sector_length;
		let data_sectors = total_sectors - (num_reserved_sectors + (num_fats * fat_size) + num_root_dir_sectors);
		let total_clusters = data_sectors / sectors_per_cluster;

		Ok(FatFS {
			fs_type: match total_clusters {
				0...4085 => FSType::Fat12,
				4096...65525 => FSType::Fat16,
				_ => FSType::Unsupported,
			},
			fat_sector: num_reserved_sectors,
			root_dir_sector: num_reserved_sectors + (num_fats * fat_size),
			disk: disk
		})
	}

	pub fn list_directory(&mut self) -> Result<DirectoryIterator, &'static str> {
		let mut root_dir = MemBuffer::new();
		let size = try!(self.disk.read(self.root_dir_sector as u64, &mut root_dir));

		Ok(DirectoryIterator {
			buffer: root_dir,
			entry_idx: 0
		})
	}
}

pub struct DirectoryEntry {
	name:[u8;11]
}

impl DirectoryEntry {
	pub fn get_name(&self) -> &str {
		unsafe { ::core::str::from_utf8_unchecked(&self.name) }
	}
}

pub struct DirectoryIterator {
	buffer:MemBuffer,
	entry_idx:usize
}

impl DirectoryIterator {
	fn get_name(name:&[u8]) -> [u8;11] {
		let mut arr : [u8;11] = [0;11];
		for i in 0..11 {
			arr[i] = name[i];
		}
		arr
	}
}

impl Iterator for DirectoryIterator {
	type Item = DirectoryEntry;

	fn next(&mut self) -> Option<DirectoryEntry> {
		let entry_type = self.buffer.get_u8(self.entry_idx);
		if entry_type == 0 {
			return None;
		}

		let name = self.buffer.get_slice(self.entry_idx, 11);
		self.entry_idx += 32;
		Some(DirectoryEntry {
			name: DirectoryIterator::get_name(name)
		})
	}
}


