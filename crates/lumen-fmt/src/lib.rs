use lumen_lexer::Lexer;
use lumen_parser::ast::*;
use lumen_parser::Parser;

/// Formatea código fuente LÚMEN.
/// Retorna el código formateado o una lista de errores.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct FmtConfig {
    pub indent_spaces: usize,
}

impl Default for FmtConfig {
    fn default() -> Self {
        Self { indent_spaces: 4 }
    }
}

pub fn format_source(source: &str) -> Result<String, Vec<String>> {
    let config = load_config();
    format_source_with_config(source, &config)
}

pub fn load_config() -> FmtConfig {
    if let Ok(content) = std::fs::read_to_string(".lumen-fmt.toml") {
        toml::from_str(&content).unwrap_or_default()
    } else {
        FmtConfig::default()
    }
}

pub fn format_source_with_config(source: &str, config: &FmtConfig) -> Result<String, Vec<String>> {
    let lexer = Lexer::new(source);
    let (tokens, lex_errors) = lexer.tokenize();
    if !lex_errors.is_empty() {
        return Err(lex_errors
            .iter()
            .map(|e| {
                format!(
                    "{} [{}:{}]: {} ({})",
                    e.code, e.pos.line, e.pos.col, e.message, e.suggestion
                )
            })
            .collect());
    }

    let parser = Parser::new(tokens);
    let (program, parse_errors) = parser.parse();
    if !parse_errors.is_empty() {
        return Err(parse_errors
            .iter()
            .map(|e| {
                format!(
                    "{} [{}:{}]: {} ({})",
                    e.code, e.span.start.line, e.span.start.col, e.message, e.suggestion
                )
            })
            .collect());
    }

    let mut fmt = Formatter::new(config.indent_spaces);
    fmt.fmt_program(&program);
    let formateado = fmt.output.trim_end().to_string() + "\n";
    // BUG-066: el lexer descarta los comentarios y el formateador reimprime
    // desde el AST, así que `lumen fmt` los BORRABA todos. Se reinyectan aquí.
    Ok(reinyectar_comentarios(source, &formateado))
}

/// BUG-066: devuelve el código con los comentarios del original recolocados.
///
/// El AST no guarda comentarios, así que se trabaja sobre el texto: para cada
/// comentario del fuente se recuerda la línea de CÓDIGO a la que acompañaba
/// (la siguiente con contenido, o la propia si es un comentario de cola) y se
/// vuelve a colgar de esa misma línea ya formateada. Es conservador: si algo no
/// cuadra, los comentarios sobrantes se añaden al final en vez de perderse.
fn reinyectar_comentarios(original: &str, formateado: &str) -> String {
    let comentarios = extraer_comentarios(original);
    if comentarios.is_empty() {
        return formateado.to_string();
    }

    // Clave de emparejamiento: el código de la línea sin espacios ni comentario.
    // Se cuenta la aparición n-ésima para no confundir líneas repetidas.
    let mut pendientes_antes: Vec<(String, usize, Vec<String>)> = Vec::new();
    let mut colas: Vec<(String, usize, String)> = Vec::new();
    let mut vistas: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut sueltos_iniciales: Vec<String> = Vec::new();
    let mut bloque_previo: Vec<String> = Vec::new();

    for linea in original.lines() {
        let (codigo, comentario) = partir_comentario(linea);
        let clave = normalizar(codigo);
        if clave.is_empty() {
            if let Some(c) = comentario {
                bloque_previo.push(c);
            }
            continue;
        }
        let n = vistas.entry(clave.clone()).or_insert(0);
        *n += 1;
        let ocurrencia = *n;
        if !bloque_previo.is_empty() {
            pendientes_antes.push((
                clave.clone(),
                ocurrencia,
                std::mem::take(&mut bloque_previo),
            ));
        }
        if let Some(c) = comentario {
            colas.push((clave, ocurrencia, c));
        }
    }
    // Comentarios al final del archivo, sin código detrás.
    sueltos_iniciales.extend(bloque_previo);

    let mut salida = String::new();
    let mut vistas_fmt: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut usados_antes = vec![false; pendientes_antes.len()];
    let mut usadas_colas = vec![false; colas.len()];

    for linea in formateado.lines() {
        let (codigo, _) = partir_comentario(linea);
        let clave = normalizar(codigo);
        if clave.is_empty() {
            salida.push_str(linea);
            salida.push('\n');
            continue;
        }
        let n = vistas_fmt.entry(clave.clone()).or_insert(0);
        *n += 1;
        let ocurrencia = *n;
        let sangria: String = linea.chars().take_while(|c| c.is_whitespace()).collect();

        for (i, (k, occ, bloque)) in pendientes_antes.iter().enumerate() {
            if !usados_antes[i] && *k == clave && *occ == ocurrencia {
                for c in bloque {
                    salida.push_str(&sangria);
                    salida.push_str(c);
                    salida.push('\n');
                }
                usados_antes[i] = true;
            }
        }

        salida.push_str(linea);
        for (i, (k, occ, c)) in colas.iter().enumerate() {
            if !usadas_colas[i] && *k == clave && *occ == ocurrencia {
                salida.push_str("  ");
                salida.push_str(c);
                usadas_colas[i] = true;
            }
        }
        salida.push('\n');
    }

    // Nada debe perderse: lo que no encontró sitio se añade al final.
    let mut restos: Vec<String> = Vec::new();
    for (i, (_, _, bloque)) in pendientes_antes.iter().enumerate() {
        if !usados_antes[i] {
            restos.extend(bloque.clone());
        }
    }
    for (i, (_, _, c)) in colas.iter().enumerate() {
        if !usadas_colas[i] {
            restos.push(c.clone());
        }
    }
    restos.extend(sueltos_iniciales);
    if !restos.is_empty() {
        if !salida.ends_with('\n') {
            salida.push('\n');
        }
        for c in restos {
            salida.push_str(&c);
            salida.push('\n');
        }
    }
    colapsar_blancos(&salida)
}

/// Deja como mucho una línea en blanco seguida, para que el formateo sea
/// idempotente: al recolocar un comentario podía quedar un hueco doble que la
/// pasada siguiente eliminaba, y el resultado no convergía.
fn colapsar_blancos(texto: &str) -> String {
    let mut salida = String::with_capacity(texto.len());
    let mut blancos = 0usize;
    for linea in texto.lines() {
        if linea.trim().is_empty() {
            blancos += 1;
            if blancos > 1 {
                continue;
            }
        } else {
            blancos = 0;
        }
        salida.push_str(linea);
        salida.push('\n');
    }
    salida
}

/// ¿Tiene el fuente algún comentario? (evita trabajo si no)
fn extraer_comentarios(fuente: &str) -> Vec<String> {
    fuente
        .lines()
        .filter_map(|l| partir_comentario(l).1)
        .collect()
}

/// Parte una línea en (código, comentario), respetando las cadenas de texto.
/// Sólo trata `//`: los comentarios de bloque se dejan como están para no
/// romper nada (el formateador tampoco los genera).
fn partir_comentario(linea: &str) -> (&str, Option<String>) {
    let bytes: Vec<char> = linea.chars().collect();
    let mut en_texto = false;
    let mut escape = false;
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if escape {
            escape = false;
        } else if c == '\\' && en_texto {
            escape = true;
        } else if c == '"' {
            en_texto = !en_texto;
        } else if !en_texto && c == '/' && i + 1 < bytes.len() && bytes[i + 1] == '/' {
            let idx: usize = bytes[..i].iter().map(|c| c.len_utf8()).sum();
            let (code, com) = linea.split_at(idx);
            return (code, Some(com.trim_end().to_string()));
        }
        i += 1;
    }
    (linea, None)
}

/// Normaliza una línea de código para poder emparejarla antes y después de
/// formatear: sin espacios y sin el comentario.
fn normalizar(codigo: &str) -> String {
    codigo.chars().filter(|c| !c.is_whitespace()).collect()
}

struct Formatter {
    output: String,
    indent: usize,
    indent_spaces: usize,
}

impl Formatter {
    fn new(indent_spaces: usize) -> Self {
        Self {
            output: String::new(),
            indent: 0,
            indent_spaces,
        }
    }

    fn push(&mut self, s: &str) {
        self.output.push_str(s);
    }
    fn push_indent(&mut self) {
        for _ in 0..self.indent {
            for _ in 0..self.indent_spaces {
                self.output.push(' ');
            }
        }
    }
    fn newline(&mut self) {
        self.output.push('\n');
    }
    /// BUG-056: `fmt_block` termina con un salto de línea, así que encadenar
    /// `} sino {` o `} atrapar (e) {` producía `}\n sino {`: el `sino` quedaba
    /// en su propia línea y con un espacio suelto delante. Se recorta el salto
    /// antes de seguir escribiendo en la misma línea.
    fn unnewline(&mut self) {
        while self.output.ends_with('\n') {
            self.output.pop();
        }
    }
    fn indent_inc(&mut self) {
        self.indent += 1;
    }
    fn indent_dec(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }

    fn fmt_program(&mut self, program: &[DeclOrStmt]) {
        for (i, node) in program.iter().enumerate() {
            if i > 0 {
                self.newline();
            }
            self.fmt_decl_or_stmt(node);
        }
    }

    fn fmt_decl_or_stmt(&mut self, node: &DeclOrStmt) {
        match node {
            DeclOrStmt::Decl(d) => self.fmt_decl(d),
            DeclOrStmt::Stmt(s) => self.fmt_stmt(s, true),
        }
    }

    fn fmt_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Variable {
                var_type,
                name,
                init,
                ..
            } => {
                self.push_indent();
                self.push(&format_type(var_type));
                self.push(" ");
                self.push(name);
                if let Some(val) = init {
                    self.push(" = ");
                    self.fmt_expr(val);
                }
                self.push(";");
                self.newline();
            }
            Decl::Const {
                var_type,
                name,
                value,
                ..
            } => {
                self.push_indent();
                self.push("const ");
                self.push(&format_type(var_type));
                self.push(" ");
                self.push(name);
                self.push(" = ");
                self.fmt_expr(value);
                self.push(";");
                self.newline();
            }
            Decl::Function {
                return_type,
                name,
                params,
                body,
                type_params,
                ..
            } => {
                if !type_params.is_empty() {
                    self.push_indent();
                    self.push("funcion ");
                    self.push(&format_type(return_type));
                    self.push(" ");
                    self.push(name);
                    self.push("<");
                    self.push(&type_params.join(", "));
                    self.push(">");
                    self.push("(");
                    self.fmt_params(params);
                    self.push(") ");
                } else {
                    self.push_indent();
                    self.push("funcion ");
                    self.push(&format_type(return_type));
                    self.push(" ");
                    self.push(name);
                    self.push("(");
                    self.fmt_params(params);
                    self.push(") ");
                }
                self.fmt_block(body);
            }
            Decl::Struct {
                name,
                fields,
                type_params,
                type_param_bounds,
                ..
            } => {
                self.push_indent();
                self.push("estructura ");
                self.push(name);
                // BUG-067: `fmt` ignoraba los parámetros de tipo y convertía
                // `estructura Par<T, U>` en `estructura Par`, con lo que el
                // fichero formateado YA NO COMPILABA (los campos de tipo `T`
                // pasaban a ser un struct inexistente). Igual en v2.4.6.
                if !type_params.is_empty() {
                    self.push("<");
                    let partes: Vec<String> = type_params
                        .iter()
                        .map(|tp| match type_param_bounds.iter().find(|(n, _)| n == tp) {
                            Some((_, bound)) => format!("{}: {}", tp, bound),
                            None => tp.clone(),
                        })
                        .collect();
                    self.push(&partes.join(", "));
                    self.push(">");
                }
                self.push(" {");
                self.newline();
                self.indent_inc();
                for f in fields {
                    self.push_indent();
                    self.push(&f.name);
                    self.push(": ");
                    self.push(&format_type(&f.field_type));
                    self.push(",");
                    self.newline();
                }
                self.indent_dec();
                self.push_indent();
                self.push("}");
                self.newline();
            }
            Decl::Enum { name, variants, .. } => {
                self.push_indent();
                self.push("enum ");
                self.push(name);
                self.push(" { ");
                for (i, v) in variants.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.push(&v.name);
                    if !v.types.is_empty() {
                        self.push("(");
                        for (j, t) in v.types.iter().enumerate() {
                            if j > 0 {
                                self.push(", ");
                            }
                            self.push(&format_type(t));
                        }
                        self.push(")");
                    }
                }
                self.push(" }");
                self.newline();
            }
            Decl::Rasgo { name, methods, .. } => {
                self.push_indent();
                self.push("rasgo ");
                self.push(name);
                self.push(" {");
                self.newline();
                self.indent_inc();
                for m in methods {
                    self.push_indent();
                    self.push("funcion ");
                    self.push(&format_type(&m.return_type));
                    self.push(" ");
                    self.push(&m.name);
                    self.push("(");
                    for (j, p) in m.params.iter().enumerate() {
                        if j > 0 {
                            self.push(", ");
                        }
                        self.push(&p.name);
                        self.push(": ");
                        self.push(&format_type(&p.param_type));
                    }
                    self.push(");");
                    self.newline();
                }
                self.indent_dec();
                self.push_indent();
                self.push("}");
                self.newline();
            }
            Decl::ImplRasgo {
                trait_name,
                target_type,
                methods,
                ..
            } => {
                self.push_indent();
                self.push("impl ");
                // BUG-131: un `impl` INHERENTE (`impl C { ... }`, sin rasgo)
                // tiene `trait_name` vacío, pero se emitía la plantilla del
                // `impl Rasgo para Tipo` igual: salía `impl  para C`, que no es
                // sintaxis válida. El formateador lo detectaba al revalidar y
                // se negaba a formatear el fichero entero, así que un archivo
                // correcto quedaba sin formatear y con un aviso confuso.
                if trait_name.is_empty() {
                    self.push(&format_type(target_type));
                } else {
                    self.push(trait_name);
                    self.push(" para ");
                    self.push(&format_type(target_type));
                }
                self.push(" {");
                self.newline();
                self.indent_inc();
                for m in methods {
                    if let Decl::Function {
                        return_type,
                        name,
                        params,
                        body,
                        ..
                    } = m
                    {
                        self.push_indent();
                        self.push("funcion ");
                        self.push(&format_type(return_type));
                        self.push(" ");
                        self.push(name);
                        self.push("(");
                        self.fmt_params(params);
                        self.push(") ");
                        self.fmt_block(body);
                    }
                }
                self.indent_dec();
                self.push_indent();
                self.push("}");
                self.newline();
            }
            // BUG-068: `Decl::Destructure` no tenía brazo y caía en el `_ => {}`
            // de abajo, así que `lumen fmt` BORRABA la declaración entera
            // (`entero x, texto y = (1, "hola");` desaparecía del archivo y el
            // resultado ya no compilaba). Igual en v2.4.6.
            Decl::Destructure { targets, init, .. } => {
                self.push_indent();
                let partes: Vec<String> = targets
                    .iter()
                    .map(|t| match &t.var_type {
                        Some(ty) => format!("{} {}", format_type(ty), t.name),
                        None => t.name.clone(),
                    })
                    .collect();
                self.push(&partes.join(", "));
                self.push(" = ");
                self.fmt_expr(init);
                self.push(";");
                self.newline();
            } // Sin `_ => {}` a propósito: todas las variantes de `Decl` están
              // cubiertas. Si se añade una nueva, el compilador obliga a
              // formatearla en vez de borrarla en silencio (BUG-068).
        }
    }

    fn fmt_stmt(&mut self, stmt: &Stmt, top_level: bool) {
        match stmt {
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                if top_level {
                    self.push_indent();
                }
                self.push("si ");
                self.fmt_condition(condition);
                self.push(" ");
                self.fmt_block(then_body);
                if let Some(ref eb) = else_body {
                    if !eb.is_empty() {
                        self.unnewline();
                        self.push(" sino ");
                        if eb.len() == 1 {
                            if let DeclOrStmt::Stmt(Stmt::If { .. }) = &eb[0] {
                                self.fmt_stmt(
                                    if let DeclOrStmt::Stmt(s) = &eb[0] {
                                        s
                                    } else {
                                        return;
                                    },
                                    false,
                                );
                            } else {
                                self.fmt_block(eb);
                            }
                        } else {
                            self.fmt_block(eb);
                        }
                    }
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                if top_level {
                    self.push_indent();
                }
                self.push("mientras ");
                self.fmt_condition(condition);
                self.push(" ");
                self.fmt_block(body);
            }
            Stmt::ForEach {
                var_name,
                expr,
                body,
                ..
            } => {
                if top_level {
                    self.push_indent();
                }
                self.push("para ");
                self.push(var_name);
                self.push(" en ");
                self.fmt_expr(expr);
                self.push(" ");
                self.fmt_block(body);
            }
            Stmt::Return { value, .. } => {
                if top_level {
                    self.push_indent();
                }
                self.push("retornar");
                if let Some(v) = value {
                    self.push(" ");
                    self.fmt_expr(v);
                }
                self.push(";");
                self.newline();
            }
            Stmt::Break { label, .. } => {
                if top_level {
                    self.push_indent();
                }
                self.push("romper");
                if let Some(l) = label {
                    self.push(" ");
                    self.push(l);
                }
                self.push(";");
                self.newline();
            }
            Stmt::Continue { label, .. } => {
                if top_level {
                    self.push_indent();
                }
                self.push("continuar");
                if let Some(l) = label {
                    self.push(" ");
                    self.push(l);
                }
                self.push(";");
                self.newline();
            }
            Stmt::Expr { expr, .. } => {
                if top_level {
                    self.push_indent();
                }
                self.fmt_expr(expr);
                self.push(";");
                self.newline();
            }
            Stmt::Assignment { name, value, .. } => {
                if top_level {
                    self.push_indent();
                }
                self.push(name);
                self.push(" = ");
                self.fmt_expr(value);
                self.push(";");
                self.newline();
            }
            // BUG-053: estas cinco sentencias no tenían brazo y caían en el
            // `_ => {}` del final, así que `lumen fmt` las BORRABA del archivo.
            // `ArraySet` (`l[j] = x;`) y `FieldAssign` (`p.x = 1;`) son de lo
            // más común: el ejemplo de ordenación por burbuja quedaba sin sus
            // asignaciones y, aun compilando, dejaba de ordenar.
            Stmt::ArraySet {
                arr, index, value, ..
            } => {
                if top_level {
                    self.push_indent();
                }
                self.fmt_expr(arr);
                self.push("[");
                self.fmt_expr(index);
                self.push("] = ");
                self.fmt_expr(value);
                self.push(";");
                self.newline();
            }
            Stmt::FieldAssign {
                expr, field, value, ..
            } => {
                if top_level {
                    self.push_indent();
                }
                self.fmt_expr(expr);
                self.push(".");
                self.push(field);
                self.push(" = ");
                self.fmt_expr(value);
                self.push(";");
                self.newline();
            }
            Stmt::Destructure { targets, value, .. } => {
                if top_level {
                    self.push_indent();
                }
                let parts: Vec<String> = targets
                    .iter()
                    .map(|t| match &t.var_type {
                        Some(ty) => format!("{} {}", format_type(ty), t.name),
                        None => t.name.clone(),
                    })
                    .collect();
                self.push(&parts.join(", "));
                self.push(" = ");
                self.fmt_expr(value);
                self.push(";");
                self.newline();
            }
            Stmt::IfLet {
                pattern,
                value,
                then_body,
                else_body,
                ..
            } => {
                if top_level {
                    self.push_indent();
                }
                self.push("si sea ");
                self.fmt_expr(pattern);
                self.push(" = ");
                self.fmt_expr(value);
                self.push(" ");
                self.fmt_block(then_body);
                if let Some(eb) = else_body {
                    self.push(" sino ");
                    self.fmt_block(eb);
                }
                self.newline();
            }
            Stmt::GuardLet {
                pattern,
                value,
                else_body,
                ..
            } => {
                if top_level {
                    self.push_indent();
                }
                self.push("sea ");
                self.fmt_expr(pattern);
                self.push(" = ");
                self.fmt_expr(value);
                self.push(" sino ");
                self.fmt_block(else_body);
                self.newline();
            }
            Stmt::Match {
                expr,
                arms,
                default,
                ..
            } => {
                if top_level {
                    self.push_indent();
                }
                self.push("elegir (");
                self.fmt_expr(expr);
                self.push(") {");
                self.newline();
                self.indent_inc();
                for arm in arms {
                    self.push_indent();
                    self.push("caso ");
                    self.fmt_expr(&arm.value);
                    // BUG-069: no se emitían las alternativas de un patrón OR,
                    // así que `caso Rojo | Amarillo:` se formateaba como
                    // `caso Rojo:`. Se perdían ramas enteras EN SILENCIO y el
                    // `elegir` pasaba a ser no exhaustivo (E080). En v2.4.6 igual.
                    for alt in &arm.alt_values {
                        self.push(" | ");
                        self.fmt_expr(alt);
                    }
                    if let Some(ref guard) = arm.guard {
                        self.push(" si ");
                        self.fmt_expr(guard);
                    }
                    self.push(":");
                    // BUG-053: emitir `caso X: { ... }` hacía que la siguiente
                    // pasada leyera esas llaves como un bloque anidado y lo
                    // volviera a envolver, creciendo en cada formateo. El
                    // cuerpo va indentado y sin llaves, que es como se escribe.
                    self.newline();
                    self.indent_inc();
                    for node in &arm.body {
                        self.fmt_decl_or_stmt(node);
                    }
                    self.indent_dec();
                }
                // BUG-070: el caso `defecto` vive en un campo aparte de
                // `Stmt::Match` que el formateador ignoraba, así que
                // desaparecía del archivo. Sin él, un `elegir` que sólo era
                // exhaustivo gracias al `defecto` dejaba de compilar (E080).
                if let Some(cuerpo) = default {
                    self.push_indent();
                    self.push("defecto:");
                    self.newline();
                    self.indent_inc();
                    for node in cuerpo {
                        self.fmt_decl_or_stmt(node);
                    }
                    self.indent_dec();
                }
                self.indent_dec();
                self.push_indent();
                self.push("}");
                self.newline();
            }
            Stmt::Block { stmts, .. } => {
                self.fmt_block(stmts);
            }
            Stmt::Posponer { body, .. } => {
                if top_level {
                    self.push_indent();
                }
                self.push("posponer ");
                self.fmt_block(body);
            }
            Stmt::TryCatch {
                try_body,
                err_var,
                catch_body,
                ..
            } => {
                if top_level {
                    self.push_indent();
                }
                self.push("intentar ");
                self.fmt_block(try_body);
                self.unnewline();
                self.push(&format!(" atrapar ({}) ", err_var));
                self.fmt_block(catch_body);
            }
            Stmt::Import { path, alias, .. } => {
                if top_level {
                    self.push_indent();
                }
                self.push("importar ");
                self.push(&format!("\"{}\"", path));
                if let Some(a) = alias {
                    self.push(" como ");
                    self.push(a);
                }
                self.push(";");
                self.newline();
            }
            Stmt::InlineAsm { code, .. } => {
                if top_level {
                    self.push_indent();
                }
                self.push(&format!("ensamblador {{ \"{}\" }}", escape_string(code)));
                self.newline();
            }
            Stmt::InlineC { code, .. } => {
                if top_level {
                    self.push_indent();
                }
                self.push(&format!("bloque_c {{ \"{}\" }}", escape_string(code)));
                self.newline();
            }
            Stmt::InlineRust { code, .. } => {
                if top_level {
                    self.push_indent();
                }
                self.push(&format!("bloque_rust {{ \"{}\" }}", escape_string(code)));
                self.newline();
            }
            _ => {}
        }
    }

    fn fmt_block(&mut self, body: &[DeclOrStmt]) {
        self.push("{");
        self.newline();
        self.indent_inc();
        for node in body {
            self.fmt_decl_or_stmt(node);
        }
        self.indent_dec();
        self.push_indent();
        self.push("}");
        self.newline();
    }

    /// BUG-053: `si`/`mientras` envuelven la condición en paréntesis, pero el
    /// parser conserva los del fuente en un `Expr::Grouping`, así que cada
    /// pasada del formateador añadía un nivel más: `(i < n)` → `((i < n))` →
    /// `(((i < n)))`. El formateo dejaba de ser idempotente.
    fn fmt_condition(&mut self, cond: &Expr) {
        self.push("(");
        match cond {
            Expr::Grouping { expr, .. } => self.fmt_expr(expr),
            other => self.fmt_expr(other),
        }
        self.push(")");
    }

    fn fmt_params(&mut self, params: &[Param]) {
        for (i, p) in params.iter().enumerate() {
            if i > 0 {
                self.push(", ");
            }
            if p.name == "self" || p.name == "yo" {
                // BUG-132: se emitía sólo `self`, tirando el `prestado mut` del
                // receptor. El método pasaba a recibirlo por valor, así que
                // `self.v = n` dejaba de mutar el struct original: el
                // formateador cambiaba la semántica del programa en silencio.
                // El receptor se declara con su tipo completo
                // (`prestado mut C self`); emitir sólo `prestado mut self`
                // no parsea (E011).
                match &p.param_type {
                    Type::Prestado { .. } | Type::Dueno(_) => {
                        self.push(&format_type(&p.param_type));
                        self.push(" ");
                    }
                    _ => {}
                }
                self.push(&p.name);
            } else {
                self.push(&format_type(&p.param_type));
                self.push(" ");
                self.push(&p.name);
                // BUG-053: el valor por defecto se perdía, y con él la
                // posibilidad de llamar a la función con menos argumentos.
                if let Some(d) = &p.default {
                    self.push(" = ");
                    self.fmt_expr(d);
                }
            }
        }
    }

    fn fmt_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Int { value, .. } => self.push(&value.to_string()),
            // BUG-053: `10.0` se reescribía como `10`, convirtiendo una
            // división real en entera (`10.0 / 4.0` pasaba de `2.5` a `2`).
            Expr::Float { value, .. } => {
                let t = value.to_string();
                if t.contains('.') || t.contains('e') || t.contains("inf") || t.contains("NaN") {
                    self.push(&t);
                } else {
                    self.push(&format!("{}.0", t));
                }
            }
            Expr::Str { value, .. } => self.push(&format!("\"{}\"", escape_string(value))),
            Expr::Bool { value, .. } => self.push(if *value { "verdadero" } else { "falso" }),
            Expr::Ident { name, .. } => self.push(name),
            Expr::Binary {
                op, left, right, ..
            } => {
                self.fmt_expr(left);
                self.push(" ");
                self.push(fmt_binop(op));
                self.push(" ");
                self.fmt_expr(right);
            }
            Expr::Unary { op, operand, .. } => {
                self.push(fmt_unop(op));
                self.fmt_expr(operand);
            }
            Expr::Ternary {
                condition,
                true_branch,
                false_branch,
                ..
            } => {
                self.fmt_expr(condition);
                self.push(" ? ");
                self.fmt_expr(true_branch);
                self.push(" : ");
                self.fmt_expr(false_branch);
            }
            Expr::Call {
                callee,
                args,
                type_args,
                ..
            } => {
                self.fmt_expr(callee);
                if !type_args.is_empty() {
                    self.push("<");
                    for (i, t) in type_args.iter().enumerate() {
                        if i > 0 {
                            self.push(", ");
                        }
                        self.push(&format_type(t));
                    }
                    self.push(">");
                }
                self.push("(");
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.fmt_expr(a);
                }
                self.push(")");
            }
            Expr::List { items, .. } => {
                self.push("[");
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.fmt_expr(item);
                }
                self.push("]");
            }
            Expr::Index { expr, index, .. } => {
                self.fmt_expr(expr);
                self.push("[");
                self.fmt_expr(index);
                self.push("]");
            }
            Expr::MethodCall {
                expr, method, args, ..
            } => {
                self.fmt_expr(expr);
                self.push(".");
                self.push(method);
                self.push("(");
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.fmt_expr(a);
                }
                self.push(")");
            }
            Expr::FieldAccess { expr, field, .. } => {
                self.fmt_expr(expr);
                self.push(".");
                self.push(field);
            }
            Expr::SafeFieldAccess { expr, field, .. } => {
                self.fmt_expr(expr);
                self.push("?.");
                self.push(field);
            }
            Expr::Elvis { expr, default, .. } => {
                self.fmt_expr(expr);
                self.push(" ?: ");
                self.fmt_expr(default);
            }
            Expr::Comprehension {
                expr,
                var_name,
                iter,
                condition,
                ..
            } => {
                self.push("[");
                self.fmt_expr(expr);
                self.push(&format!(" para {} en ", var_name));
                self.fmt_expr(iter);
                if let Some(cond) = condition {
                    self.push(" si ");
                    self.fmt_expr(cond);
                }
                self.push("]");
            }
            Expr::Query {
                var_name,
                source,
                where_clause,
                select_expr,
                ..
            } => {
                self.push(&format!("consultar {} en ", var_name));
                self.fmt_expr(source);
                if let Some(w) = where_clause {
                    self.push(" donde ");
                    self.fmt_expr(w);
                }
                self.push(" seleccionar ");
                self.fmt_expr(select_expr);
            }
            Expr::StructInit {
                struct_name: name,
                fields,
                ..
            } => {
                self.push(name);
                self.push(" { ");
                for (i, (fname, fexpr)) in fields.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.push(fname);
                    self.push(": ");
                    self.fmt_expr(fexpr);
                }
                self.push(" }");
            }
            Expr::Grouping { expr, .. } => {
                // BUG-071: un `Expr::Cast` ya se imprime entre paréntesis, así
                // que envolverlo otra vez añadía un nivel EN CADA PASADA:
                // `(x como entero)` → `((x como entero))` → ... El fichero
                // crecía sin límite al formatear repetidamente y el formateo
                // dejaba de ser idempotente. Si lo de dentro se autoparentiza,
                // no se duplica. Nótese que NO se pueden quitar los paréntesis
                // en general: `((a / b) como entero)` los necesita.
                if matches!(expr.as_ref(), Expr::Cast { .. } | Expr::Grouping { .. }) {
                    self.fmt_expr(expr);
                } else {
                    self.push("(");
                    self.fmt_expr(expr);
                    self.push(")");
                }
            }
            Expr::Cast {
                expr, cast_type, ..
            } => {
                // BUG-071: el cast se envuelve en paréntesis y el parser guarda
                // los del fuente como `Expr::Grouping`, así que cada pasada
                // añadía un nivel: `(x como entero)` → `((x como entero))` →
                // ... El formateo no era idempotente y el fichero crecía en
                // cada `lumen fmt`. Mismo caso que BUG-053 en las condiciones:
                // se desenvuelve el `Grouping` redundante.
                self.push("(");
                self.fmt_expr(expr);
                self.push(" como ");
                self.push(&format_type(cast_type));
                self.push(")");
            }
            Expr::Tuple { items, .. } => {
                self.push("(");
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.fmt_expr(item);
                }
                self.push(")");
            }
            Expr::EnumCtor {
                enum_name,
                variant,
                args,
                ..
            } => {
                self.push(enum_name);
                self.push("::");
                self.push(variant);
                if !args.is_empty() {
                    self.push("(");
                    for (i, a) in args.iter().enumerate() {
                        if i > 0 {
                            self.push(", ");
                        }
                        self.fmt_expr(a);
                    }
                    self.push(")");
                }
            }
            // BUG-053: sin estos brazos, `fmt` borraba la expresión entera.
            // `exito(...)`/`error(...)` desaparecían de los `retornar`, y `t.0`
            // dejaba `imprimir()` sin argumentos.
            Expr::Exito { expr, .. } => {
                self.push("exito(");
                self.fmt_expr(expr);
                self.push(")");
            }
            Expr::Error { expr, .. } => {
                self.push("error(");
                self.fmt_expr(expr);
                self.push(")");
            }
            Expr::Algun { expr, .. } => {
                self.push("algun(");
                self.fmt_expr(expr);
                self.push(")");
            }
            Expr::Ninguno { .. } => self.push("ninguno"),
            Expr::Intentar { expr, .. } => {
                self.push("intentar ");
                self.fmt_expr(expr);
            }
            Expr::Esperar { expr, .. } => {
                self.push("esperar ");
                self.fmt_expr(expr);
            }
            Expr::TupleAccess { expr, index, .. } => {
                self.fmt_expr(expr);
                self.push(&format!(".{}", index));
            }
            Expr::Range {
                start,
                end,
                inclusive,
                ..
            } => {
                self.fmt_expr(start);
                self.push(if *inclusive { "..=" } else { ".." });
                self.fmt_expr(end);
            }
            Expr::Comptime { expr, .. } => {
                self.push("en_tiempo_compilacion { ");
                self.fmt_expr(expr);
                self.push(" }");
            }
            // BUG-053: sin este brazo, una lambda caía en el `_ => {}` de abajo
            // y `lumen fmt` la BORRABA: `sea f = funcion(entero x) {...};` se
            // reescribía como `Infer f = ;`, destruyendo el fichero del usuario.
            Expr::Lambda { params, body, .. } => {
                self.push("funcion(");
                self.fmt_params(params);
                self.push(") ");
                self.fmt_block(body);
            } // BUG-053: sin `_ => {}`. Todas las variantes de `Expr` están
              // cubiertas y así debe seguir: si se añade una nueva al AST, el
              // compilador obliga a formatearla en vez de borrarla en silencio.
        }
    }
}

fn format_type(t: &Type) -> String {
    match t {
        Type::Entero => "entero".to_string(),
        Type::Decimal => "decimal".to_string(),
        Type::Texto => "texto".to_string(),
        Type::Booleano => "booleano".to_string(),
        Type::Numero => "numero".to_string(),
        Type::Lista(inner) => format!("lista<{}>", format_type(inner)),
        Type::Resultado { ok, err } => {
            format!("resultado<{}, {}>", format_type(ok), format_type(err))
        }
        Type::Opcion(inner) => format!("opcion<{}>", format_type(inner)),
        Type::Func {
            param_types,
            return_type,
        } => {
            let params: Vec<String> = param_types.iter().map(format_type).collect();
            format!(
                "funcion({}) {}",
                params.join(", "),
                format_type(return_type)
            )
        }
        // BUG-053: `sea x = ...` se parsea con el tipo centinela `Infer`. El
        // formateador lo escupía tal cual (`Infer x = ...`), que no es sintaxis
        // válida de LÚMEN y dejaba el fichero sin compilar.
        Type::Struct(name) if name == "Infer" => "sea".to_string(),
        Type::Struct(name) => name.clone(),
        Type::Tuple(types) => {
            let inner: Vec<String> = types.iter().map(format_type).collect();
            format!("({})", inner.join(", "))
        }
        Type::GenericStruct { name, .. } => name.clone(),
        Type::ImplTrait(name) => format!("impl {}", name),
        Type::Prestado { inner, mutable } => {
            if *mutable {
                format!("prestado mut {}", format_type(inner))
            } else {
                format!("prestado {}", format_type(inner))
            }
        }
        Type::Dueno(inner) => format!("dueno {}", format_type(inner)),
    }
}

fn fmt_binop(op: &BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Concat => "++",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Mod => "%",
        BinOp::Equal => "==",
        BinOp::NotEqual => "!=",
        BinOp::Less => "<",
        BinOp::LessEqual => "<=",
        BinOp::Greater => ">",
        BinOp::GreaterEqual => ">=",
        BinOp::And => "&&",
        BinOp::Or => "||",
        BinOp::BitOr => "|",
        BinOp::BitAnd => "&",
        BinOp::BitXor => "^",
        BinOp::ShiftLeft => "<<",
        BinOp::ShiftRight => ">>",
    }
}

fn fmt_unop(op: &UnOp) -> &'static str {
    match op {
        UnOp::Negate => "-",
        UnOp::Not => "!",
        UnOp::BitNot => "~",
    }
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple() {
        let result = format_source("imprimir(\"hola\");").unwrap();
        assert!(result.contains("hola"));
    }

    #[test]
    fn test_format_function() {
        let src = "funcion entero suma(entero a,entero b){retornar a+b;}";
        let result = format_source(src).unwrap();
        assert!(result.contains("funcion entero suma"));
    }
}

#[cfg(test)]
mod tests_v3 {
    use super::format_source_with_config;
    use super::FmtConfig;

    fn fmt(src: &str) -> String {
        format_source_with_config(src, &FmtConfig::default()).expect("debe formatear")
    }

    /// Formatear dos veces debe dar lo mismo que formatear una.
    fn assert_idempotente(src: &str) {
        let una = fmt(src);
        let dos = fmt(&una);
        assert_eq!(una, dos, "el formateo no es idempotente");
    }

    // BUG-066: `fmt` borraba TODOS los comentarios del archivo.
    #[test]
    fn bug066_conserva_los_comentarios() {
        let src = "// cabecera\nentero x = 1; // cola\n// suelto\nimprimir(x);\n";
        let salida = fmt(src);
        assert!(
            salida.contains("// cabecera"),
            "falta la cabecera:\n{salida}"
        );
        assert!(salida.contains("// cola"), "falta el de cola:\n{salida}");
        assert!(salida.contains("// suelto"), "falta el suelto:\n{salida}");
    }

    #[test]
    fn bug066_no_confunde_las_barras_dentro_de_un_texto() {
        let src = "texto url = \"http://ejemplo.com\";\nimprimir(url);\n";
        let salida = fmt(src);
        assert!(
            salida.contains("\"http://ejemplo.com\""),
            "se comió parte del texto:\n{salida}"
        );
    }

    // BUG-067: se perdían los parámetros de tipo de los structs genéricos.
    #[test]
    fn bug067_conserva_los_genericos_de_un_struct() {
        let src = "estructura Par<T, U> {\n    primero: T,\n    segundo: U,\n}\n";
        let salida = fmt(src);
        assert!(
            salida.contains("estructura Par<T, U>"),
            "perdió los parámetros de tipo:\n{salida}"
        );
    }

    // BUG-068: `Decl::Destructure` no tenía brazo y desaparecía del archivo.
    #[test]
    fn bug068_conserva_la_declaracion_con_destructuring() {
        let src = "entero x, texto y = (1, \"hola\");\nimprimir(x);\n";
        let salida = fmt(src);
        assert!(
            salida.contains("entero x, texto y"),
            "borró la declaración:\n{salida}"
        );
    }

    // BUG-069: se perdían las alternativas de un patrón OR.
    #[test]
    fn bug069_conserva_los_patrones_or() {
        let src = "elegir (n) {\n    caso 1 | 2 | 3:\n        imprimir(\"pocos\");\n    defecto:\n        imprimir(\"muchos\");\n}\n";
        let salida = fmt(src);
        assert!(
            salida.contains("caso 1 | 2 | 3:"),
            "perdió las alternativas:\n{salida}"
        );
    }

    // BUG-070: el caso `defecto` vive en un campo aparte y se ignoraba.
    #[test]
    fn bug070_conserva_el_caso_defecto() {
        let src = "elegir (n) {\n    caso 1:\n        imprimir(\"uno\");\n    defecto:\n        imprimir(\"otro\");\n}\n";
        let salida = fmt(src);
        assert!(
            salida.contains("defecto:"),
            "perdió el caso defecto:\n{salida}"
        );
    }

    // BUG-071: cada pasada añadía un paréntesis a los casts.
    #[test]
    fn bug071_los_casts_no_acumulan_parentesis() {
        let src = "entero t = (x como entero);\n";
        assert_idempotente(src);
        let una = fmt(src);
        let dos = fmt(&una);
        assert!(
            !dos.contains("((("),
            "los paréntesis siguen creciendo:\n{dos}"
        );
    }

    #[test]
    fn bug071_no_rompe_la_precedencia_de_un_cast() {
        // Quitar los paréntesis aquí cambiaría el significado a `a / (b como entero)`.
        let src = "entero t = ((a / b) como entero);\n";
        let salida = fmt(src);
        assert!(
            salida.contains("(a / b)"),
            "se perdieron los paréntesis necesarios:\n{salida}"
        );
        assert_idempotente(src);
    }

    #[test]
    fn el_formateo_es_idempotente_en_construcciones_variadas() {
        for src in [
            "// c\nentero x = 1;\n",
            "estructura P<T> {\n    v: T,\n}\n",
            "entero a, entero b = (1, 2);\n",
            "elegir (n) {\n    caso 1 | 2:\n        imprimir(1);\n    defecto:\n        imprimir(0);\n}\n",
            "si (x > 0) {\n    imprimir(1);\n} sino {\n    imprimir(2);\n}\n",
        ] {
            assert_idempotente(src);
        }
    }
}
