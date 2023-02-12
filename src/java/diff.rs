pub fn apply(orig: &str, diff: &str) -> Option<String> {
    if orig.is_empty() {
        return None;
    }
    let mut result = Vec::from_iter(orig.chars());
    let mut pos = result.len();
    let mut chars = diff.chars();
    // TODO: replace with next_chunk when stable
    while let (Some(cmd), Some(param)) = (chars.next(), chars.next()) {
        pos = pos.checked_sub(1)?;
        match cmd {
            '-' => {
                assert!(param.is_ascii_lowercase());
                let par_num = ((param as u8) - b'a') as usize;
                pos = pos.checked_sub(par_num)?;
            }
            'R' => result[pos] = param,
            'D' => {
                assert!(param.is_ascii_lowercase());
                let par_num = ((param as u8) - b'a') as usize;
                let o = pos;
                pos = pos.checked_sub(par_num)?;
                result.drain(pos..=o);
            }
            'I' => {
                pos += 1;
                result.insert(pos, param);
            }
            _ => unreachable!(),
        }
    }
    Some(result.into_iter().collect())
}
