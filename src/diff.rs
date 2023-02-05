pub fn apply(orig: &str, diff: &str) -> String {
    if orig.is_empty() {
        return orig.into();
    }
    let mut _orig = Vec::from_iter(orig.chars());
    let mut pos = _orig.len() - 1;
    let mut chars = diff.chars();
    // TODO: replace with next_chunk when stable
    while let (Some(cmd), Some(param)) = (chars.next(), chars.next()) {
        match cmd {
            '-' => {
                assert!(param.is_ascii_lowercase());
                let par_num = ((param as u8) - b'a') as usize;
                pos -= par_num;
            }
            'R' => _orig[pos] = param,
            'D' => {
                assert!(param.is_ascii_lowercase());
                let par_num = ((param as u8) - b'a') as usize;
                let o = pos;
                pos -= par_num;
                _orig.drain(pos..=o);
            }
            'I' => {
                pos += 1;
                _orig.insert(pos, param);
            }
            _ => unreachable!(),
        }
        if pos == 0 {
            break;
        }
        pos -= 1;
    }
    _orig.into_iter().collect()
}
