// Minimal regex engine — AST-based, recursive backtracking
#![allow(unused_assignments)]

#[derive(Clone)]
enum Piece {
    Lit(char),
    Any,
    Digit,
    NonDigit,
    Word,
    NonWord,
    Space,
    NonSpace,
    Class {
        chars: Vec<char>,
        ranges: Vec<(char, char)>,
        negated: bool,
    },
    Quant {
        inner: Box<Piece>,
        min: usize,
        max: usize,
    },
    Start,
    End,
    Capture {
        inner: Vec<Piece>,
        idx: usize,
    },
    Alt(Vec<Vec<Piece>>),
    /// Lookahead positivo (?=...) - cero ancho
    Look(Vec<Piece>),
    /// Lookahead negativo (?!...) - cero ancho
    NegLook(Vec<Piece>),
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
        if pi != chars.len() {
            return Err("Trailing characters".into());
        }
        Ok(Regex { pieces, groups })
    }

    pub fn is_match(&self, text: &str) -> bool {
        let cs: Vec<char> = text.chars().collect();
        if !self.pieces.is_empty() {
            if let Piece::Start = self.pieces[0] {
                return try_match(&self.pieces, &cs, 0, 0).is_some();
            }
        }
        for i in 0..=cs.len() {
            if try_match(&self.pieces, &cs, i, 0).is_some() {
                return true;
            }
        }
        false
    }

    pub fn captures(&self, text: &str) -> Vec<String> {
        let cs: Vec<char> = text.chars().collect();
        let mut caps = vec![String::new(); self.groups + 1];
        // v3.5.42: los índices de grupo del parser son 1-based (el primer
        // grupo es 1); el vector de spans debe tener largo groups+1 y el
        // grupo g vive en groups[g]. Antes se asignaba largo groups y se
        // copiaba corrido (groups[0]→caps[1]) — el grupo 1 quedaba vacío y
        // el último grupo se descartaba con el guard del matcher.
        let anchored = matches!(self.pieces.first(), Some(Piece::Start));
        let intentos: Vec<usize> = if anchored {
            vec![0]
        } else {
            (0..=cs.len()).collect()
        };
        for i in intentos {
            let mut groups = vec![(0usize, 0usize); self.groups + 1];
            let mut group_idx = 0;
            if let Some((end, _)) =
                try_match_cap(&self.pieces, &cs, i, 0, &mut groups, &mut group_idx)
            {
                caps[0] = cs[i..end].iter().collect();
                for g in 1..=self.groups {
                    let (s, e) = groups[g];
                    if s < e {
                        caps[g] = cs[s..e].iter().collect();
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
            let mut caps = vec![(0usize, 0usize); self.groups + 1];
            let mut gi = 0;
            if let Some((end, _)) = try_match_cap(&self.pieces, &cs, pos, 0, &mut caps, &mut gi) {
                // v3.5.42: caps[0] = coincidencia completa — $0 / ${0}
                // expanden al match entero (antes quedaba (0,0) → vacío).
                caps[0] = (pos, end);
                // v3.4.0: expansión de $1..$9 y ${n} sobre las capturas
                let rc: Vec<char> = replacement.chars().collect();
                let mut i = 0;
                while i < rc.len() {
                    if rc[i] == '$' && i + 1 < rc.len() {
                        if rc[i + 1] == '{' {
                            if let Some(close) = rc[i + 2..].iter().position(|&c| c == '}') {
                                let num: String = rc[i + 2..i + 2 + close].iter().collect();
                                if let Ok(n) = num.trim().parse::<usize>() {
                                    if n < caps.len() && caps[n].1 > caps[n].0 {
                                        res.extend(&cs[caps[n].0..caps[n].1]);
                                    }
                                }
                                i += 3 + close;
                                continue;
                            }
                        } else if rc[i + 1].is_ascii_digit() {
                            let n = rc[i + 1].to_digit(10).unwrap() as usize;
                            if n < caps.len() && caps[n].1 > caps[n].0 {
                                res.extend(&cs[caps[n].0..caps[n].1]);
                            }
                            i += 2;
                            continue;
                        }
                    }
                    res.push(rc[i]);
                    i += 1;
                }
                pos = if end > pos { end } else { pos + 1 };
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

fn try_match_cap(
    pieces: &[Piece],
    cs: &[char],
    mut ci: usize,
    mut pi: usize,
    caps: &mut Vec<(usize, usize)>,
    _gi: &mut usize,
) -> Option<(usize, usize)> {
    while pi < pieces.len() {
        match &pieces[pi] {
            Piece::Start => {
                if ci != 0 {
                    return None;
                }
                pi += 1;
            }
            Piece::End => {
                if ci != cs.len() {
                    return None;
                }
                pi += 1;
            }
            Piece::Any => {
                if ci >= cs.len() || cs[ci] == '\n' {
                    return None;
                }
                ci += 1;
                pi += 1;
            }
            Piece::Lit(c) => {
                if ci >= cs.len() || cs[ci] != *c {
                    return None;
                }
                ci += 1;
                pi += 1;
            }
            Piece::Digit => {
                if ci >= cs.len() || !cs[ci].is_ascii_digit() {
                    return None;
                }
                ci += 1;
                pi += 1;
            }
            Piece::NonDigit => {
                if ci >= cs.len() || cs[ci].is_ascii_digit() {
                    return None;
                }
                ci += 1;
                pi += 1;
            }
            Piece::Word => {
                if ci >= cs.len() || !(cs[ci].is_ascii_alphanumeric() || cs[ci] == '_') {
                    return None;
                }
                ci += 1;
                pi += 1;
            }
            Piece::NonWord => {
                if ci >= cs.len() || cs[ci].is_ascii_alphanumeric() || cs[ci] == '_' {
                    return None;
                }
                ci += 1;
                pi += 1;
            }
            Piece::Space => {
                if ci >= cs.len() || !cs[ci].is_ascii_whitespace() {
                    return None;
                }
                ci += 1;
                pi += 1;
            }
            Piece::NonSpace => {
                if ci >= cs.len() || cs[ci].is_ascii_whitespace() {
                    return None;
                }
                ci += 1;
                pi += 1;
            }
            Piece::Class {
                chars,
                ranges,
                negated,
            } => {
                if ci >= cs.len() {
                    return None;
                }
                let ch = cs[ci];
                let mut matched = chars.contains(&ch);
                for (lo, hi) in ranges {
                    if ch >= *lo && ch <= *hi {
                        matched = true;
                    }
                }
                if *negated {
                    matched = !matched;
                }
                if !matched {
                    return None;
                }
                ci += 1;
                pi += 1;
            }
            Piece::Quant { inner, min, max } => {
                // Greedy: try max first, then reduce
                let mut count = 0;
                while count < *max {
                    match try_match_cap(&[*(*inner).clone()], cs, ci, 0, caps, _gi) {
                        Some((end, _)) => {
                            ci = end;
                            count += 1;
                        }
                        None => break,
                    }
                }
                // Semántica del motor: el bloque se evalúa SOLO una vez (el
                // cuerpo siempre retorna o hace break), por eso es un `if`.
                if count >= *min {
                    let saved_ci = ci;
                    // Restore ci for the backtrack position
                    match try_match_cap(&pieces[pi + 1..], cs, ci, 0, caps, _gi) {
                        Some((end, _)) => {
                            return Some((end, ci));
                        }
                        None => {
                            // Backtrack: undo one match
                            count -= 1;
                            // Need to re-match one fewer time
                            ci = saved_ci;
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
                        if *idx < caps.len() {
                            caps[*idx] = (start, end);
                        }
                        ci = end;
                        pi += 1;
                    }
                    None => return None,
                }
            }
            Piece::Look(inner) => {
                try_match_cap(inner, cs, ci, 0, caps, _gi)?;
                pi += 1; // cero ancho: no consume
            }
            Piece::NegLook(inner) => {
                if try_match_cap(inner, cs, ci, 0, caps, _gi).is_some() {
                    return None;
                }
                pi += 1; // cero ancho: no consume
            }
            Piece::Alt(alternatives) => {
                for alt in alternatives {
                    if let Some((end, _)) = try_match_cap(alt, cs, ci, 0, caps, _gi) {
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
        } else {
            break;
        }
    }
    if alts.len() == 1 {
        Ok(alts.remove(0))
    } else {
        let mut groups = 0;
        let alt_pieces: Vec<Vec<Piece>> = alts
            .into_iter()
            .map(|(p, g)| {
                groups += g;
                p
            })
            .collect();
        Ok((vec![Piece::Alt(alt_pieces)], groups))
    }
}

fn parse_concat(chars: &[char], pi: &mut usize) -> Result<(Vec<Piece>, usize), String> {
    let mut pieces = Vec::new();
    let mut groups = 0;
    while *pi < chars.len() {
        let c = chars[*pi];
        if c == '|' || c == ')' {
            break;
        }
        let piece = parse_piece(chars, pi, &mut groups)?;
        // Check for quantifier
        if *pi < chars.len() {
            match chars[*pi] {
                '*' => {
                    *pi += 1;
                    pieces.push(Piece::Quant {
                        inner: Box::new(piece),
                        min: 0,
                        max: usize::MAX,
                    });
                    continue;
                }
                '+' => {
                    *pi += 1;
                    pieces.push(Piece::Quant {
                        inner: Box::new(piece),
                        min: 1,
                        max: usize::MAX,
                    });
                    continue;
                }
                '?' => {
                    *pi += 1;
                    pieces.push(Piece::Quant {
                        inner: Box::new(piece),
                        min: 0,
                        max: 1,
                    });
                    continue;
                }
                '{' => {
                    // v3.4.2: cuantificador acotado {m}, {m,}, {m,n}; malformado → '{' literal
                    let save = *pi;
                    *pi += 1;
                    let mut min_s = String::new();
                    while *pi < chars.len() && chars[*pi].is_ascii_digit() {
                        min_s.push(chars[*pi]);
                        *pi += 1;
                    }
                    if !min_s.is_empty() {
                        let mut max_opt: Option<String> = None;
                        if *pi < chars.len() && chars[*pi] == ',' {
                            *pi += 1;
                            let mut ms = String::new();
                            while *pi < chars.len() && chars[*pi].is_ascii_digit() {
                                ms.push(chars[*pi]);
                                *pi += 1;
                            }
                            max_opt = Some(ms);
                        }
                        if *pi < chars.len() && chars[*pi] == '}' {
                            *pi += 1;
                            let min: usize = min_s.parse().unwrap_or(0);
                            let max: usize = match &max_opt {
                                None => min,
                                Some(ms) if ms.is_empty() => usize::MAX,
                                Some(ms) => ms.parse().unwrap_or(min),
                            };
                            pieces.push(Piece::Quant {
                                inner: Box::new(piece),
                                min,
                                max,
                            });
                            continue;
                        }
                    }
                    *pi = save; // no acotador válido → cae como pieza normal
                }
                _ => {}
            }
        }
        pieces.push(piece);
    }
    Ok((pieces, groups))
}

fn parse_piece(chars: &[char], pi: &mut usize, groups: &mut usize) -> Result<Piece, String> {
    if *pi >= chars.len() {
        return Err("Unexpected end".into());
    }
    let c = chars[*pi];
    *pi += 1;
    match c {
        '^' => Ok(Piece::Start),
        '$' => Ok(Piece::End),
        '.' => Ok(Piece::Any),
        '\\' => {
            if *pi >= chars.len() {
                return Err("Trailing backslash".into());
            }
            let esc = chars[*pi];
            *pi += 1;
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
            if negated {
                *pi += 1;
            }
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
            if *pi < chars.len() {
                *pi += 1;
            }
            Ok(Piece::Class {
                chars: cls_chars,
                ranges,
                negated,
            })
        }
        '(' => {
            // v3.4.6: lookahead negativo (?!...)
            if *pi + 1 < chars.len() && chars[*pi] == '?' && chars[*pi + 1] == '!' {
                *pi += 2;
                let (inner, _) = parse_alt(chars, pi)?;
                if *pi >= chars.len() || chars[*pi] != ')' {
                    return Err("Unmatched '('".into());
                }
                *pi += 1;
                return Ok(Piece::NegLook(inner));
            }
            // v3.4.5: lookahead positivo (?=...)
            if *pi + 1 < chars.len() && chars[*pi] == '?' && chars[*pi + 1] == '=' {
                *pi += 2;
                let (inner, _) = parse_alt(chars, pi)?;
                if *pi >= chars.len() || chars[*pi] != ')' {
                    return Err("Unmatched '('".into());
                }
                *pi += 1;
                return Ok(Piece::Look(inner));
            }
            // v3.4.4: grupo NO capturante `(?:...)` — desciende sin grabar
            // captura (idx::MAX supera el guard `idx < caps.len()` del matcher)
            if *pi + 1 < chars.len() && chars[*pi] == '?' && chars[*pi + 1] == ':' {
                *pi += 2;
                let (inner, _) = parse_alt(chars, pi)?;
                if *pi >= chars.len() || chars[*pi] != ')' {
                    return Err("Unmatched '('".into());
                }
                *pi += 1;
                return Ok(Piece::Capture {
                    inner,
                    idx: usize::MAX,
                });
            }
            *groups += 1;
            let idx = *groups;
            let (inner, _) = parse_alt(chars, pi)?;
            if *pi >= chars.len() || chars[*pi] != ')' {
                return Err("Unmatched '('".into());
            }
            *pi += 1;
            Ok(Piece::Capture { inner, idx })
        }
        ')' => Err("Unmatched ')'".into()),
        '|' => Err("Unexpected '|'".into()),
        _ => Ok(Piece::Lit(c)),
    }
}
