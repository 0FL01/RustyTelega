//! Isolate the bundled source face from an installed face with the same name.

pub(super) const FAMILY: &str = "Qll Intro";

fn u16_at(data: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_be_bytes(data.get(at..at + 2)?.try_into().ok()?))
}

fn u32_at(data: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(data.get(at..at + 4)?.try_into().ok()?))
}

fn checksum(data: &[u8]) -> u32 {
    data.chunks(4)
        .map(|chunk| {
            chunk
                .iter()
                .fold(0u32, |word, byte| (word << 8) | u32::from(*byte))
                << ((4 - chunk.len()) * 8)
        })
        .fold(0, u32::wrapping_add)
}

pub(super) fn alias_face(data: &[u8]) -> Option<Vec<u8>> {
    let table = |tag: &[u8]| {
        (0..usize::from(u16_at(data, 4)?)).find_map(|index| {
            let record = 12 + index * 16;
            (data.get(record..record + 4)? == tag).then_some((
                record,
                u32_at(data, record + 8)? as usize,
                u32_at(data, record + 12)? as usize,
            ))
        })
    };
    let (name_record, name, name_len) = table(b"name")?;
    let (_, head, _) = table(b"head")?;
    let storage = name + usize::from(u16_at(data, name + 4)?);
    let mut out = data.to_vec();
    for index in 0..usize::from(u16_at(data, name + 2)?) {
        let record = name + 6 + index * 12;
        if !matches!(u16_at(data, record + 6)?, 1 | 3 | 4 | 6 | 16) {
            continue;
        }
        let start = storage + usize::from(u16_at(data, record + 10)?);
        let length = usize::from(u16_at(data, record + 8)?);
        let field = out.get_mut(start..start + length)?;
        for (before, after) in [("Open Sans", FAMILY), ("OpenSans", "QllIntro")] {
            let bytes = |text: &str| -> Vec<u8> {
                if u16_at(data, record) == Some(1) {
                    text.as_bytes().to_vec()
                } else {
                    text.encode_utf16().flat_map(u16::to_be_bytes).collect()
                }
            };
            let (before, after) = (bytes(before), bytes(after));
            for at in 0..field.len().saturating_sub(before.len()).saturating_add(1) {
                if field.get(at..at + before.len()) == Some(before.as_slice()) {
                    field.get_mut(at..at + after.len())?.copy_from_slice(&after);
                }
            }
        }
    }
    let name_checksum = checksum(out.get(name..name + name_len)?);
    out.get_mut(name_record + 4..name_record + 8)?
        .copy_from_slice(&name_checksum.to_be_bytes());
    out.get_mut(head + 8..head + 12)?.fill(0);
    let adjustment = 0xb1b0afbau32.wrapping_sub(checksum(&out));
    out.get_mut(head + 8..head + 12)?
        .copy_from_slice(&adjustment.to_be_bytes());
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intro_alias_keeps_the_source_tables_and_valid_font_checksum() {
        for source in [
            include_bytes!("../../../../assets/fonts/OpenSans-Regular.ttf").as_slice(),
            include_bytes!("../../../../assets/fonts/OpenSans-SemiBold.ttf").as_slice(),
            include_bytes!("../../../../assets/fonts/OpenSans-Italic.ttf").as_slice(),
            include_bytes!("../../../../assets/fonts/OpenSans-SemiBoldItalic.ttf").as_slice(),
        ] {
            let aliased = alias_face(source).expect("bundled font name table");
            assert_eq!(aliased.len(), source.len());
            assert_eq!(checksum(&aliased), 0xb1b0afba);
            assert!(
                aliased
                    .windows(FAMILY.len())
                    .any(|bytes| bytes == FAMILY.as_bytes())
            );
            assert_ne!(aliased, source);
        }
    }
}
