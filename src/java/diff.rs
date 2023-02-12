pub fn apply(orig: &str, diff: &str) -> Option<String> {
    if orig.is_empty() {
        return None;
    }
    let mut result = Vec::from_iter(orig.chars());
    let mut pos = (result.len() - 1) as isize;
    let mut chars = diff.chars();
    // TODO: replace with next_chunk when stable
    while let (Some(cmd), Some(param)) = (chars.next(), chars.next()) {
        match cmd {
            '-' => {
                assert!(param.is_ascii_lowercase());
                let par_num = ((param as u8) - b'a') as isize;
                pos -= par_num;
            }
            'R' => *result.get_mut(usize::try_from(pos).ok()?)? = param,
            'D' => {
                assert!(param.is_ascii_lowercase());
                let par_num = ((param as u8) - b'a') as isize;
                let e = usize::try_from(pos).ok()?;
                pos -= par_num;
                let s = usize::try_from(pos).ok()?;
                result.drain(s..=e);
            }
            'I' => {
                pos += 1;
                result.insert(usize::try_from(pos).ok()?, param);
            }
            _ => unreachable!(),
        }
        pos -= 1;
    }
    if result.is_empty() {
        None
    } else {
        Some(result.into_iter().collect())
    }
}
