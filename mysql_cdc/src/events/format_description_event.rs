use crate::constants::checksum_type::ChecksumType;
use crate::events::event_header::EventHeader;
use crate::events::event_type::EventType;
use crate::{constants, errors::Error};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read, Seek, SeekFrom};
use serde::{Deserialize, Serialize};

const EVENT_TYPES_OFFSET: u8 = 2 + 50 + 4 + 1;

/// Written as the first event in binlog file or when replication is started.
/// See <a href="https://mariadb.com/kb/en/library/format_description_event/">MariaDB docs</a>
/// See <a href="https://dev.mysql.com/doc/internals/en/format-description-event.html">MySQL docs</a>
/// See <a href="https://mariadb.com/kb/en/library/5-slave-registration/#events-transmission-after-com_binlog_dump">start events flow</a>
#[derive(Debug,Serialize,Deserialize)]
pub struct FormatDescriptionEvent {
    /// Gets binary log format version. This should always be 4.
    pub binlog_version: u16,

    /// Gets MariaDB/MySQL server version name.
    pub server_version: String,

    /// Gets checksum algorithm type.
    pub checksum_type: ChecksumType,
}

impl FormatDescriptionEvent {
    /// Supports all versions of MariaDB and MySQL 5.0+ (V4 header format).
    pub fn parse(cursor: &mut Cursor<&[u8]>, header: &EventHeader) -> Result<Self, Error> {
        let binlog_version = cursor.read_u16::<LittleEndian>()?;

        // Read server version
        let mut server_version = [0u8; 50];
        cursor.read_exact(&mut server_version)?;
        let mut slice: &[u8] = &server_version;
        if let Some(zero_index) = server_version.iter().position(|&b| b == 0) {
            slice = &server_version[..zero_index];
        }
        let server_version = std::str::from_utf8(slice)?.to_string();

        // Redundant timestamp & header length which is always 19
        cursor.seek(SeekFrom::Current(5))?;

        // Get size of the event payload to determine beginning of the checksum part
        let seek_len = EventType::FormatDescriptionEvent as i64 - 1;
        cursor.seek(SeekFrom::Current(seek_len))?;
        let payload_length = cursor.read_u8()?;

        let mut checksum_type = ChecksumType::None;
        if payload_length != header.event_length as u8 - constants::EVENT_HEADER_SIZE as u8 {
            let skip = payload_length as i64
                - EVENT_TYPES_OFFSET as i64
                - EventType::FormatDescriptionEvent as i64;

            cursor.seek(SeekFrom::Current(skip))?;
            checksum_type = ChecksumType::from_code(cursor.read_u8()?)?;
        }

        Ok(Self {
            binlog_version,
            server_version,
            checksum_type,
        })
    }
}
