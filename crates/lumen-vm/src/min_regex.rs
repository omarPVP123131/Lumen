// Minimal regex engine — AST-based, recursive backtracking
#![allow(unused_assignments)]

#[derive(Clone)]
enum Piece {
    Lit(char),
    Any,
    Digit, NonDigit, Word, NonWord, Space, NonSpace,
    Class { chars: Vec<char>, ranges: Vec<(char, char)>, negated: bool },
    Quant { inner: Box<Piece>, min: usize, max: usize },
    Start, End,
    Capture { inner: Vec<Piece>, idx: usize },
    Alt(Vec<Vec<Piece>>),
}

pub struct Regex {
    pieces: Vec<Piece>,
    groups: usize,
}

impl Regex {
    pub fn new(pattern: &str) -> Result<Self, String> {
        let chars: Vec<char> = pattern.chars().collect();
        let mut pi = 0;
        let (pieces, groups) = parse_alt(&chars, &mut pi)?;
        if pi != chars.len() { return Err("Trailing characters".into()); }
        Ok(Regex { pieces, groups })
    }

    pub fn is_match(&self, text: &str) -> bool {
        let cs: Vec<char> = text.chars().collect();
        if self.pieces.len() > 0 {
            if let Piece::Start = self.pieces[0] {
                return try_match(&self.pieces, &cs, 0, 0).is_some();
            }
        }
        for i in 0..=cs.len() {
            if let Some(_) = try_match(&self.pieces, &cs, i, 0) { return true; }
        }
        false
    }

    pub fn captures(&self, text: &str) -> Vec<String> {
        let cs: Vec<char> = text.chars().collect();
        let mut caps = vec![String::new(); self.groups + 1];
        let mut groups = vec![(0usize, 0usize); self.groups];
        let mut group_idx = 0;
        if self.pieces.len() > 0 {
            if let Piece::Start = self.pieces[0] {
                if let Some((end, _)) = try_match_cap(&self.pieces, &cs, 0, 0, &mut groups, &mut group_idx) {
                    caps[0] = cs[..end].iter().collect();
                    for (i, (s, e)) in groups.iter().enumerate() {
                        if i < self.groups && *s < *e {
                            caps[i + 1] = cs[*s..*e].iter().collect();
                        }
                    }
                    return caps;
                }
            }
        }
        for i in 0..=cs.len() {
            if let Some((end, _)) = try_match_cap(&self.pieces, &cs, i, 0, &mut groups, &mut group_idx) {
                caps[0] = cs[i..end].iter().collect();
                for (i, (s, e)) in groups.iter().enumerate() {
                    if i < self.groups && *s < *e {
                        caps[i + 1] = cs[*s..*e].iter().collect();
                    }
                }
                return caps;
            }
        }
        caps
    }

    pub fn replace(&self, text: &str, replacement: &str) -> String {
        let cs: Vec<char> = text.chars().collect();
        let mut res = String::new();
        let mut pos = 0;
        while pos < cs.len() {
            if let Some(end) = try_match(&self.pieces, &cs, pos, 0) {
                res.push_str(replacement);
                pos = end;
            } else {
                res.push(cs[pos]);
                pos += 1;
            }
        }
        res
    }
}

fn try_match(pieces: &[Piece], cs: &[char], ci: usize, pi: usize) -> Option<usize> {
    let mut caps = vec![(0usize, 0usize); 20];
    let mut gi = 0;
    try_match_cap(pieces, cs, ci, pi, &mut caps, &mut gi).map(|(e, _)| e)
}

fn try_match_cap(pieces: &[Piece], cs: &[char], mut ci: usize, mut pi: usize, caps: &mut Vec<(usize, usize)>, gi: &mut usize) -> Option<(usize, usize)> {
    while pi < pieces.len() {
        match &pieces[pi] {
            Piece::Start => { if ci != 0 { return None; } pi += 1; }
            Piece::End => { if ci != cs.len() { return None; } pi += 1; }
            Piece::Any => {
                if ci >= cs.len() || cs[ci] == '\n' { return None; }
                ci += 1; pi += 1;
            }
            Piece::Lit(c) => {
                if ci >= cs.len() || cs[ci] != *c { return None; }
                ci += 1; pi += 1;
            }
            Piece::Digit => {
                if ci >= cs.len() || !cs[ci].is_ascii_digit() { return None; }
                ci += 1; pi += 1;
            }
            Piece::NonDigit => {
                if ci >= cs.len() || cs[ci].is_ascii_digit() { return None; }
                ci += 1; pi += 1;
            }
            Piece::Word => {
                if ci >= cs.len() || !(cs[ci].is_ascii_alphanumeric() || cs[ci] == '_') { return None; }
                ci += 1; pi += 1;
            }
            Piece::NonWord => {
                if ci >= cs.len() || cs[ci].is_ascii_alphanumeric() || cs[ci] == '_' { return None; }
                ci += 1; pi += 1;
            }
            Piece::Space => {
                if ci >= cs.len() || !cs[ci].is_ascii_whitespace() { return None; }
                ci += 1; pi += 1;
            }
            Piece::NonSpace => {
                if ci >= cs.len() || cs[ci].is_ascii_whitespace() { return None; }
                ci += 1; pi += 1;
            }
            Piece::Class { chars, ranges, negated } => {
                if ci >= cs.len() { return None; }
                let ch = cs[ci];
                let mut matched = chars.contains(&ch);
                for (lo, hi) in ranges { if ch >= *lo && ch <= *hi { matched = true; } }
                if *negated { matched = !matched; }
                if !matched { return None; }
                ci += 1; pi += 1;
            }
            Piece::Quant { inner, min, max } => {
                // Greedy: try max first, then reduce
                let mut count = 0;
                while count < *max {
                    match try_match_cap(&[*(*inner).clone()], cs, ci, 0, caps, gi) {
                        Some((end, _)) => { ci = end; count += 1; }
                        None => break,
                    }
                }
                while count >= *min {
                    let saved_ci = ci;
                    // Restore ci for the backtrack position
                    match try_match_cap(&pieces[pi + 1..], cs, ci, 0, caps, gi) {
                        Some((end, _)) => { return Some((end, ci)); }
                        None => {
                            // Backtrack: undo one match
                            count -= 1;
                            // Need to re-match one fewer time
                            ci = saved_ci;
                            break;
                        }
                    }
                }
                // Couldn't find a valid count
                return None;
            }
            Piece::Capture { inner, idx } => {
                let start = ci;
                let mut inner_gi = 0;
                match try_match_cap(inner, cs, ci, 0, caps, &mut inner_gi) {
                    Some((end, _)) => {
                        if *idx < caps.len() { caps[*idx] = (start, end); }
                        ci = end; pi += 1;
                    }
                    None => return None,
                }
            }
            Piece::Alt(alternatives) => {
                for alt in alternatives {
                    if let Some((end, _)) = try_match_cap(alt, cs, ci, 0, caps, gi) {
                        return Some((end, ci));
                    }
                }
                return None;
            }
        }
    }
    Some((ci, ci))
}

fn parse_alt(chars: &[char], pi: &mut usize) -> Result<(Vec<Piece>, usize), String> {
    let mut alts = Vec::new();
    loop {
        let (pieces, groups) = parse_concat(chars, pi)?;
        alts.push((pieces, groups));
        if *pi < chars.len() && chars[*pi] == '|' {
            *pi += 1;
        } else { break; }
    }
    if alts.len() == 1 {
        Ok(alts.remove(0))
    } else {
        let mut groups = 0;
        let alt_pieces: Vec<Vec<Piece>> = alts.into_iter().map(|(p, g)| { groups += g; p }).collect();
        Ok((vec![Piece::Alt(alt_pieces)], groups))
    }
}

fn parse_concat(chars: &[char], pi: &mut usize) -> Result<(Vec<Piece>, usize), String> {
    let mut pieces = Vec::new();
    let mut groups = 0;
    while *pi < chars.len() {
        let c = chars[*pi];
        if c == '|' || c == ')' { break; }
        let piece = parse_piece(chars, pi, &mut groups)?;
        // Check for quantifier
        if *pi < chars.len() {
            match chars[*pi] {
                '*' => { *pi += 1; pieces.push(Piece::Quant { inner: Box::new(piece), min: 0, max: usize::MAX }); continue; }
                '+' => { *pi += 1; pieces.push(Piece::Quant { inner: Box::new(piece), min: 1, max: usize::MAX }); continue; }
                '?' => { *pi += 1; pieces.push(Piece::Quant { inner: Box::new(piece), min: 0, max: 1 }); continue; }
                _ => {}
            }
        }
        pieces.push(piece);
    }
    Ok((pieces, groups))
}

fn parse_piece(chars: &[char], pi: &mut usize, groups: &mut usize) -> Result<Piece, String> {
    if *pi >= chars.len() { return Err("Unexpected end".into()); }
    let c = chars[*pi];
    *pi += 1;
    match c {
        '^' => Ok(Piece::Start),
        '$' => Ok(Piece::End),
        '.' => Ok(Piece::Any),
        '\\' => {
            if *pi >= chars.len() { return Err("Trailing backslash".into()); }
            let esc = chars[*pi]; *pi += 1;
            match esc {
                'd' => Ok(Piece::Digit),
                'D' => Ok(Piece::NonDigit),
                'w' => Ok(Piece::Word),
                'W' => Ok(Piece::NonWord),
                's' => Ok(Piece::Space),
                'S' => Ok(Piece::NonSpace),
                c => Ok(Piece::Lit(c)),
            }
        }
        '[' => {
            let negated = *pi < chars.len() && chars[*pi] == '^';
            if negated { *pi += 1; }
            let mut cls_chars = Vec::new();
            let mut ranges = Vec::new();
            while *pi < chars.len() && chars[*pi] != ']' {
                if *pi + 2 < chars.len() && chars[*pi + 1] == '-' && chars[*pi + 2] != ']' {
                    ranges.push((chars[*pi], chars[*pi + 2]));
                    *pi += 3;
                } else {
                    cls_chars.push(chars[*pi]);
                    *pi += 1;
                }
            }
            if *pi < chars.len() { *pi += 1; }
            Ok(Piece::Class { chars: cls_chars, ranges, negated })
        }
        '(' => {
            *groups += 1;
            let idx = *groups;
            let (inner, _) = parse_alt(chars, pi)?;
            if *pi >= chars.len() || chars[*pi] != ')' { return Err("Unmatched '('".into()); }
            *pi += 1;
            Ok(Piece::Capture { inner, idx })
        }
        ')' => Err("Unmatched ')'".into()),
        '|' => Err("Unexpected '|'".into()),
        _ => Ok(Piece::Lit(c)),
    }
}
