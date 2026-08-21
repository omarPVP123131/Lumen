use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use lumen_lexer::token::Span;
use lumen_lexer::Lexer;
use lumen_parser::ast::*;
use lumen_parser::Parser;

/// Prefijo virtual para módulos resueltos desde memoria (playground WASM).
/// Los paths con este prefijo NO existen en disco; `read_module_source`
/// los resuelve desde `ModuleLoader::memory_files`.
pub const VIRTUAL_MEM_PREFIX: &str = "__lumen_mem__";

/// True si el path pertenece al filesystem virtual (playground WASM).
/// Las rutas virtuales NO deben tocar disco: en wasm el fs paniquea.
fn is_virtual(path: &Path) -> bool {
    path.components()
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .map(|s| s == VIRTUAL_MEM_PREFIX)
        .unwrap_or(false)
}

#[derive(Debug)]
pub enum ModuleError {
    Io { path: PathBuf, message: String },
    Lex { path: PathBuf, details: Vec<String> },
    Parse { path: PathBuf, details: Vec<String> },
    Circular { path: PathBuf, span: Span },
}

pub struct ModuleLoader {
    search_paths: Vec<PathBuf>,
    visited: HashSet<PathBuf>,
    emitted: HashSet<PathBuf>,
    known_prefixes: HashSet<String>,
    memory_files: HashMap<String, String>,
}

// Stdlib embebida en el binario, generada por `build.rs` a partir de
// `stdlib/*.nv`. BUG-152: sin esto, un binario instalado fuera del repo no
// encontraba NINGUNO de los 69 modulos importables y `importar "texto";`
// fallaba nada mas instalar. Viaja dentro del ejecutable, asi que no hay que
// distribuir la stdlib aparte ni acertar con una ruta de instalacion.
include!(concat!(env!("OUT_DIR"), "/embedded_stdlib.rs"));

/// La stdlib embebida como mapa `nombre -> fuente`, lista para el loader.
fn stdlib_embebida() -> HashMap<String, String> {
    STDLIB_FILES
        .iter()
        .map(|(n, src)| ((*n).to_string(), (*src).to_string()))
        .collect()
}

impl ModuleLoader {
    pub fn new(search_paths: Vec<PathBuf>) -> Self {
        Self {
            search_paths,
            visited: HashSet::new(),
            emitted: HashSet::new(),
            known_prefixes: HashSet::new(),
            // BUG-152: la stdlib embebida se precarga SIEMPRE. Es el ultimo
            // recurso de `resolve_path`: el disco sigue teniendo prioridad, de
            // modo que `-L` y una stdlib local siguen ganando y quien la edite
            // en el repo la ve al instante. Lo que cambia es que dejar de
            // encontrarla ya no es posible.
            memory_files: stdlib_embebida(),
        }
    }

    pub fn with_default_search_paths() -> Self {
        Self::new(Vec::new())
    }

    /// Crea un loader con un filesystem virtual en memoria (clave = nombre
    /// base del archivo, p. ej. `"texto.nv"`). Los imports se resuelven desde
    /// memoria ANTES de tocar disco; el comportamiento de disco queda intacto
    /// si la clave no existe. Usado por el runtime WASM (playground).
    pub fn with_memory_files(memory_files: HashMap<String, String>) -> Self {
        let mut loader = Self::with_default_search_paths();
        // Los ficheros del llamador se superponen a la stdlib embebida: el
        // playground puede sobrescribir un modulo, pero no pierde el resto.
        loader.memory_files.extend(memory_files);
        loader
    }

    /// Resuelve un import desde el filesystem virtual en memoria por nombre
    /// base (p. ej. `texto.nv` → `__lumen_mem__/texto.nv`).
    fn resolve_from_memory(&self, name: &str) -> Option<PathBuf> {
        if name.starts_with(VIRTUAL_MEM_PREFIX) {
            return Some(PathBuf::from(name));
        }
        let base = Path::new(name).file_name()?.to_str()?;
        let extensions = [".nv", ".lumen"];
        let mut candidates = vec![base.to_string()];
        for ext in &extensions {
            if !base.ends_with(ext) {
                candidates.push(format!("{}{}", base, ext));
            }
        }
        for c in candidates {
            if self.memory_files.contains_key(&c) {
                return Some(PathBuf::from(format!("{}/{}", VIRTUAL_MEM_PREFIX, c)));
            }
        }
        None
    }

    /// Lee el contenido de un módulo: desde memoria si el path es virtual,
    /// desde disco en caso contrario.
    fn read_module_source(&self, path: &Path) -> Result<String, ModuleError> {
        if path.starts_with(VIRTUAL_MEM_PREFIX) {
            if let Some(name) = path.file_name().and_then(|f| f.to_str()) {
                if let Some(src) = self.memory_files.get(name) {
                    return Ok(src.clone());
                }
            }
            return Err(ModuleError::Io {
                path: path.to_path_buf(),
                message: format!("Módulo virtual no encontrado: '{}'", path.display()),
            });
        }
        fs::read_to_string(path).map_err(|e| ModuleError::Io {
            path: path.to_path_buf(),
            message: format!("No se pudo leer '{}': {}", path.display(), e),
        })
    }

    pub fn resolve_imports(
        &mut self,
        source: &str,
        base_path: &Path,
    ) -> Result<Program, ModuleError> {
        self.visited.clear();
        self.emitted.clear();
        self.known_prefixes.clear();
        let program = parse_source(source, base_path)?;
        self.flatten(program, base_path)
    }

    fn flatten(&mut self, program: Program, current_path: &Path) -> Result<Program, ModuleError> {
        // Canonicalizar para comparar rutas de forma robusta (Windows: fs::canonicalize
        // añade el prefijo \\?\ — comparar crudo vs canonical nunca da igualdad).
        let current_norm = if is_virtual(current_path) {
            // En wasm el fs paniquea: las rutas virtuales se usan tal cual.
            current_path.to_path_buf()
        } else {
            fs::canonicalize(current_path).unwrap_or_else(|_| current_path.to_path_buf())
        };
        let mut result = Vec::new();
        for node in program {
            match node {
                DeclOrStmt::Stmt(Stmt::Import { path, alias, span }) => {
                    if path == "ingles" || path == "english" {
                        continue;
                    }
                    let current_dir = if is_virtual(&current_norm) || current_norm.is_dir() {
                        current_norm.clone()
                    } else {
                        current_norm
                            .parent()
                            .unwrap_or(Path::new("."))
                            .to_path_buf()
                    };
                    let resolved = self.resolve_path(&path, &current_dir, &current_norm)?;
                    if resolved == current_norm {
                        // Self-import: el archivo se importa a sí mismo por nombre
                        // (p. ej. `examples/graficos_avanzado.nv` importando
                        // "graficos_avanzado.nv"). No-op para romper el ciclo.
                        continue;
                    }
                    if !self.emitted.insert(resolved.clone()) {
                        // Ya aplanado antes (import directo + transitivo):
                        // p. ej. `tui_core.nv` vía `tui.nv` y también directo.
                        // La copia ya insertada lleva su prefijo correcto.
                        continue;
                    }
                    if !self.visited.insert(resolved.clone()) {
                        return Err(ModuleError::Circular {
                            path: resolved,
                            span,
                        });
                    }
                    let source = self.read_module_source(&resolved)?;
                    let imported_program = parse_source(&source, &resolved)?;
                    let _parent = resolved.parent().unwrap_or(Path::new("."));
                    let flat = self.flatten(imported_program, &resolved)?;
                    self.visited.remove(&resolved);
                    let prefix = alias.unwrap_or_else(|| {
                        // BUG-154: cuando el import resuelve a un PAQUETE (un
                        // directorio con `lumen.toml`), el prefijo debe ser el
                        // nombre por el que se importa, no el del fichero de
                        // entrada. `importar "libreria";` sobre un paquete cuya
                        // entrada es `src/main.nv` producia funciones
                        // `main_sumar` en vez de `libreria_sumar`, justo lo que
                        // `lumen install` promete al terminar. El nombre del
                        // fichero de entrada es un detalle interno del paquete:
                        // renombrar `main.nv` a `lib.nv` no puede cambiar la API
                        // que ven sus consumidores.
                        if let Some(nombre) = self.nombre_de_paquete(&resolved, &path) {
                            return nombre;
                        }
                        resolved
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("module")
                            .to_string()
                    });
                    self.known_prefixes.insert(prefix.clone());
                    let mut prefixed = flat;
                    prefix_program(&mut prefixed, &prefix, &self.known_prefixes);
                    result.extend(prefixed);
                }
                other => result.push(other),
            }
        }
        Ok(result)
    }

    /// BUG-154: si `resolved` es el fichero de entrada de un paquete (un
    /// directorio con `lumen.toml`), devuelve el nombre por el que se importa.
    /// Se toma del propio texto del `importar` —su ultimo segmento—, que es lo
    /// que el usuario escribe y lo que `lumen install` le dijo que escribiera.
    /// Solo se aplica si ese directorio es realmente un paquete resuelto por
    /// `check_package_dir`; para un `.nv` suelto el prefijo sigue siendo el
    /// nombre del fichero, como siempre.
    fn nombre_de_paquete(&self, resolved: &Path, texto_import: &str) -> Option<String> {
        // El nombre que escribio el usuario, sin ruta ni extension.
        let pedido = Path::new(texto_import).file_stem()?.to_str()?.to_string();
        if pedido.is_empty() {
            return None;
        }
        // ¿Hay un directorio-paquete, entre los sitios donde se busca, cuya
        // entrada sea exactamente este fichero?
        let mut bases: Vec<PathBuf> = self.search_paths.clone();
        if let Some(padre) = resolved.parent() {
            bases.push(padre.to_path_buf());
            if let Some(abuelo) = padre.parent() {
                bases.push(abuelo.to_path_buf());
            }
        }
        for base in bases {
            let dir = base.join(&pedido);
            if !dir.is_dir() {
                continue;
            }
            if let Some(entrada) = self.check_package_dir(&dir) {
                let entrada_norm = fs::canonicalize(&entrada).unwrap_or(entrada);
                let resolved_norm =
                    fs::canonicalize(resolved).unwrap_or_else(|_| resolved.to_path_buf());
                if entrada_norm == resolved_norm {
                    return Some(pedido);
                }
            }
        }
        None
    }

    fn check_package_dir(&self, dir: &Path) -> Option<PathBuf> {
        if !dir.is_dir() {
            return None;
        }
        let manifest = dir.join("lumen.toml");
        if manifest.is_file() {
            if let Ok(content) = fs::read_to_string(&manifest) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("principal =") || trimmed.starts_with("main =") {
                        if let Some(val) = trimmed.split('=').nth(1) {
                            let entry_file = val.trim().trim_matches('"').trim();
                            let target = dir.join(entry_file);
                            if target.is_file() {
                                return fs::canonicalize(&target).ok().or(Some(target));
                            }
                        }
                    }
                }
            }
        }
        let candidates = [
            "src/main.nv",
            "src/lib.nv",
            "main.nv",
            "mod.nv",
            "lib.nv",
            "index.nv",
        ];
        for c in &candidates {
            let p = dir.join(c);
            if p.is_file() {
                return fs::canonicalize(&p).ok().or(Some(p));
            }
        }
        if let Some(stem) = dir.file_name().and_then(|s| s.to_str()) {
            let p = dir.join(format!("{}.nv", stem));
            if p.is_file() {
                return fs::canonicalize(&p).ok().or(Some(p));
            }
        }
        None
    }

    fn resolve_path(
        &self,
        path: &str,
        current_dir: &Path,
        current_path: &Path,
    ) -> Result<PathBuf, ModuleError> {
        let extensions = [".nv", ".lumen"];
        // Playground WASM: si el importador vive en el filesystem virtual,
        // todo lo que puede resolver está en memoria — nunca tocar disco
        // (en wasm fs::exists/canonicalize paniquean).
        if is_virtual(current_dir) {
            if let Some(mem) = self.resolve_from_memory(path) {
                return Ok(mem);
            }
            return Err(ModuleError::Io {
                path: current_dir.join(path),
                message: format!("Módulo no encontrado en la stdlib embebida: '{}'", path),
            });
        }
        // Skip de auto-importación: si la ruta as-is cae sobre el archivo que se
        // está aplanando (p. ej. `examples/graficos_avanzado.nv` importando
        // "graficos_avanzado.nv"), continuar hacia los search_paths (stdlib).
        let is_self = |p: &Path| -> bool {
            if is_virtual(p) {
                return false;
            }
            fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf()) == current_path
        };

        // Check if path is a package directory
        let cur_dir = current_dir.join(path);
        if cur_dir.is_dir() {
            if let Some(entry) = self.check_package_dir(&cur_dir) {
                if !is_self(&entry) {
                    return Ok(entry);
                }
            }
        }
        for sp in &self.search_paths {
            let sp_cand = sp.join(path);
            if sp_cand.is_dir() {
                if let Some(entry) = self.check_package_dir(&sp_cand) {
                    if !is_self(&entry) {
                        return Ok(entry);
                    }
                }
            }
        }

        if path.contains('.') || path.contains('/') || path.contains('\\') {
            // Try as-is (full path with extension)
            let p = current_dir.join(path);
            if p.exists() && !is_self(&p) {
                return Ok(fs::canonicalize(&p).unwrap_or(p));
            }
            for sp in &self.search_paths {
                let p = sp.join(path);
                if p.exists() && !is_self(&p) {
                    return Ok(fs::canonicalize(&p).unwrap_or(p));
                }
            }
            // Check subdirectories of search_paths (e.g. stdlib/compiler/)
            for sp in &self.search_paths {
                if let Ok(entries) = fs::read_dir(sp) {
                    for entry in entries.flatten() {
                        let sub = entry.path();
                        if sub.is_dir() {
                            let p = sub.join(path);
                            if p.exists() && !is_self(&p) {
                                return Ok(fs::canonicalize(&p).unwrap_or(p));
                            }
                        }
                    }
                }
            }
            // Try with extensions
            for ext in &extensions {
                let p = current_dir.join(format!("{}{}", path, ext));
                if p.exists() && !is_self(&p) {
                    return Ok(fs::canonicalize(&p).unwrap_or(p));
                }
            }
            for sp in &self.search_paths {
                for ext in &extensions {
                    let p = sp.join(format!("{}{}", path, ext));
                    if p.exists() && !is_self(&p) {
                        return Ok(fs::canonicalize(&p).unwrap_or(p));
                    }
                }
            }
            for sp in &self.search_paths {
                if let Ok(entries) = fs::read_dir(sp) {
                    for entry in entries.flatten() {
                        let sub = entry.path();
                        if sub.is_dir() {
                            for ext in &extensions {
                                let p = sub.join(format!("{}{}", path, ext));
                                if p.exists() && !is_self(&p) {
                                    return Ok(fs::canonicalize(&p).unwrap_or(p));
                                }
                            }
                        }
                    }
                }
            }
            if let Some(mem) = self.resolve_from_memory(path) {
                return Ok(mem);
            }
            Err(ModuleError::Io {
                path: current_dir.join(path),
                message: format!("Archivo no encontrado: '{}'", path),
            })
        } else {
            for ext in &extensions {
                let p = current_dir.join(format!("{}{}", path, ext));
                if p.exists() && !is_self(&p) {
                    return Ok(fs::canonicalize(&p).unwrap_or(p));
                }
            }
            for sp in &self.search_paths {
                for ext in &extensions {
                    let p = sp.join(format!("{}{}", path, ext));
                    if p.exists() && !is_self(&p) {
                        return Ok(fs::canonicalize(&p).unwrap_or(p));
                    }
                }
            }
            for sp in &self.search_paths {
                if let Ok(entries) = fs::read_dir(sp) {
                    for entry in entries.flatten() {
                        let sub = entry.path();
                        if sub.is_dir() {
                            for ext in &extensions {
                                let p = sub.join(format!("{}{}", path, ext));
                                if p.exists() && !is_self(&p) {
                                    return Ok(fs::canonicalize(&p).unwrap_or(p));
                                }
                            }
                        }
                    }
                }
            }
            if let Some(mem) = self.resolve_from_memory(path) {
                return Ok(mem);
            }
            Err(ModuleError::Io {
                path: current_dir.join(format!("{}.nv", path)),
                message: format!("Módulo no encontrado: '{}'", path),
            })
        }
    }
}

fn parse_source(source: &str, path: &Path) -> Result<Program, ModuleError> {
    // Caché de imports a nivel de proceso: clave = path canónico + mtime.
    // Evita re-lexear/re-parsear los mismos módulos entre invocaciones
    // (lumen serve, LSP, tests). Solo se cachean archivos reales (mtime
    // presente); los virtuales del playground se parsean siempre.
    use std::sync::{Mutex, OnceLock};
    use std::time::SystemTime;
    static SOURCE_CACHE: OnceLock<Mutex<HashMap<PathBuf, (SystemTime, Program)>>> = OnceLock::new();
    let cache = SOURCE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mtime = std::fs::metadata(path).and_then(|m| m.modified()).ok();
    if let Some(mt) = mtime {
        let cache = cache.lock().unwrap();
        if let Some((cm, p)) = cache.get(path) {
            if *cm == mt {
                return Ok(p.clone());
            }
        }
    }
    let lexer = Lexer::new(source);
    let (tokens, lex_errors) = lexer.tokenize();
    if !lex_errors.is_empty() {
        return Err(ModuleError::Lex {
            path: path.to_path_buf(),
            details: lex_errors
                .iter()
                .map(|e| {
                    format!(
                        "{} [{}:{}]: {} ({})",
                        e.code, e.pos.line, e.pos.col, e.message, e.suggestion
                    )
                })
                .collect(),
        });
    }
    let parser = Parser::new(tokens);
    let (program, parse_errors) = parser.parse();
    if !parse_errors.is_empty() {
        return Err(ModuleError::Parse {
            path: path.to_path_buf(),
            details: parse_errors
                .iter()
                .map(|e| {
                    format!(
                        "{} [{}:{}]: {} ({})",
                        e.code, e.span.start.line, e.span.start.col, e.message, e.suggestion
                    )
                })
                .collect(),
        });
    }
    if let Some(mt) = mtime {
        cache
            .lock()
            .unwrap()
            .insert(path.to_path_buf(), (mt, program.clone()));
    }
    Ok(program)
}

fn collect_module_declarations(program: &Program) -> HashSet<String> {
    let mut names = HashSet::new();
    for item in program {
        match item {
            DeclOrStmt::Decl(decl) => match decl {
                Decl::Variable { name, .. } => {
                    names.insert(name.clone());
                }
                Decl::Const { name, .. } => {
                    names.insert(name.clone());
                }
                Decl::Function { name, .. } => {
                    names.insert(name.clone());
                }
                Decl::Struct { name, .. } => {
                    names.insert(name.clone());
                }
                Decl::Enum { name, variants, .. } => {
                    names.insert(name.clone());
                    for v in variants {
                        names.insert(v.name.clone());
                    }
                }
                Decl::Rasgo { name, .. } => {
                    names.insert(name.clone());
                }
                Decl::Destructure { targets, .. } => {
                    for t in targets {
                        if t.name != "_" {
                            names.insert(t.name.clone());
                        }
                    }
                }
                Decl::ImplRasgo { .. } => {}
            },
            DeclOrStmt::Stmt(stmt) => {
                if let Stmt::Assignment { name, .. } = stmt {
                    names.insert(name.clone());
                }
            }
        }
    }
    names
}

fn prefix_program(program: &mut Program, prefix: &str, known: &HashSet<String>) {
    let module_decls = collect_module_declarations(program);
    let mut locals = HashSet::new();
    for node in program.iter_mut() {
        prefix_node(node, prefix, &mut locals, true, known, &module_decls);
    }
}

fn prefix_node(
    node: &mut DeclOrStmt,
    prefix: &str,
    locals: &mut HashSet<String>,
    top_level: bool,
    known: &HashSet<String>,
    module_decls: &HashSet<String>,
) {
    match node {
        DeclOrStmt::Decl(d) => prefix_decl(d, prefix, locals, top_level, known, module_decls),
        DeclOrStmt::Stmt(s) => prefix_stmt(s, prefix, locals, top_level, known, module_decls),
    }
}

fn prefix_decl(
    decl: &mut Decl,
    prefix: &str,
    locals: &mut HashSet<String>,
    top_level: bool,
    known: &HashSet<String>,
    module_decls: &HashSet<String>,
) {
    match decl {
        Decl::Variable {
            var_type,
            name,
            init,
            ..
        } => {
            prefix_type(var_type, prefix, known);
            if top_level {
                if !is_known_prefixed(name, known) {
                    *name = format!("{}_{}", prefix, name);
                }
            } else {
                locals.insert(name.clone());
            }
            if let Some(expr) = init {
                prefix_expr(expr, prefix, locals, known, module_decls);
            }
        }
        Decl::Const {
            var_type,
            name,
            value,
            ..
        } => {
            prefix_type(var_type, prefix, known);
            if top_level {
                if !is_known_prefixed(name, known) {
                    *name = format!("{}_{}", prefix, name);
                }
            } else {
                locals.insert(name.clone());
            }
            prefix_expr(value, prefix, locals, known, module_decls);
        }
        Decl::Destructure { targets, init, .. } => {
            for target in targets.iter_mut() {
                if let Some(ref mut t_type) = target.var_type {
                    prefix_type(t_type, prefix, known);
                }
                if target.name != "_" {
                    if top_level {
                        if !is_known_prefixed(&target.name, known) {
                            target.name = format!("{}_{}", prefix, target.name);
                        }
                    } else {
                        locals.insert(target.name.clone());
                    }
                }
            }
            prefix_expr(init, prefix, locals, known, module_decls);
        }
        Decl::Function {
            return_type,
            name,
            params,
            body,
            type_params,
            ..
        } => {
            let type_params_set: HashSet<String> = type_params.iter().cloned().collect();
            prefix_type_with_params(return_type, prefix, &type_params_set, known);
            if top_level && !is_known_prefixed(name, known) {
                *name = format!("{}_{}", prefix, name);
            }
            for p in params.iter_mut() {
                prefix_type_with_params(&mut p.param_type, prefix, &type_params_set, known);
                if let Some(default) = &mut p.default {
                    prefix_expr(default, prefix, locals, known, module_decls);
                }
            }
            let mut func_locals = locals.clone();
            for p in params.iter() {
                func_locals.insert(p.name.clone());
            }
            for node in body.iter_mut() {
                prefix_node(node, prefix, &mut func_locals, false, known, module_decls);
            }
        }
        Decl::Struct {
            name,
            fields,
            type_params,
            ..
        } => {
            if top_level && !is_known_prefixed(name, known) {
                *name = format!("{}_{}", prefix, name);
            }
            let type_params_set: HashSet<String> = type_params.iter().cloned().collect();
            for field in fields.iter_mut() {
                prefix_type_with_params(&mut field.field_type, prefix, &type_params_set, known);
            }
        }
        Decl::Enum { name, variants, .. } => {
            if top_level && !is_known_prefixed(name, known) {
                *name = format!("{}_{}", prefix, name);
            }
            for variant in variants.iter_mut() {
                for t in variant.types.iter_mut() {
                    prefix_type(t, prefix, known);
                }
            }
        }
        Decl::Rasgo { name, methods, .. } => {
            if top_level && !is_known_prefixed(name, known) {
                *name = format!("{}_{}", prefix, name);
            }
            for method in methods.iter_mut() {
                prefix_type(&mut method.return_type, prefix, known);
                for p in method.params.iter_mut() {
                    prefix_type(&mut p.param_type, prefix, known);
                }
            }
        }
        Decl::ImplRasgo {
            trait_name: _,
            target_type,
            associated_types,
            methods,
            ..
        } => {
            prefix_type(target_type, prefix, known);
            for assoc in associated_types.iter_mut() {
                prefix_type(&mut assoc.target_type, prefix, known);
            }
            for method_decl in methods.iter_mut() {
                prefix_decl(method_decl, prefix, locals, false, known, module_decls);
            }
        }
    }
}

fn prefix_stmt(
    stmt: &mut Stmt,
    prefix: &str,
    locals: &mut HashSet<String>,
    _top_level: bool,
    known: &HashSet<String>,
    module_decls: &HashSet<String>,
) {
    match stmt {
        Stmt::Assignment { name, value, .. } => {
            if !locals.contains(name.as_str())
                && !is_known_prefixed(name, known)
                && (module_decls.contains(name.as_str()) || !is_builtin(name))
            {
                *name = format!("{}_{}", prefix, name);
            }
            prefix_expr(value, prefix, locals, known, module_decls);
        }
        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            prefix_expr(condition, prefix, locals, known, module_decls);
            let mut if_locals = locals.clone();
            for node in then_body.iter_mut() {
                prefix_node(node, prefix, &mut if_locals, false, known, module_decls);
            }
            if let Some(body) = else_body {
                let mut else_locals = locals.clone();
                for node in body.iter_mut() {
                    prefix_node(node, prefix, &mut else_locals, false, known, module_decls);
                }
            }
        }
        Stmt::While {
            condition, body, ..
        } => {
            prefix_expr(condition, prefix, locals, known, module_decls);
            let mut while_locals = locals.clone();
            for node in body.iter_mut() {
                prefix_node(node, prefix, &mut while_locals, false, known, module_decls);
            }
        }
        Stmt::For {
            init,
            condition,
            update,
            body,
            ..
        } => {
            let mut for_locals = locals.clone();
            if let Decl::Variable { name, .. } = init.as_mut() {
                for_locals.insert(name.clone());
            }
            prefix_expr(condition, prefix, &for_locals, known, module_decls);
            prefix_stmt(update, prefix, &mut for_locals, false, known, module_decls);
            for node in body.iter_mut() {
                prefix_node(node, prefix, &mut for_locals, false, known, module_decls);
            }
        }
        Stmt::Return { value, .. } => {
            if let Some(expr) = value {
                prefix_expr(expr, prefix, locals, known, module_decls);
            }
        }
        Stmt::ForEach {
            var_name,
            expr,
            body,
            ..
        } => {
            let mut foreach_locals = locals.clone();
            foreach_locals.insert(var_name.clone());
            prefix_expr(expr, prefix, &foreach_locals, known, module_decls);
            for node in body.iter_mut() {
                prefix_node(
                    node,
                    prefix,
                    &mut foreach_locals,
                    false,
                    known,
                    module_decls,
                );
            }
        }
        Stmt::Break { .. } | Stmt::Continue { .. } | Stmt::Import { .. } => {}
        Stmt::GuardLet {
            pattern,
            value,
            else_body,
            ..
        } => {
            prefix_expr(value, prefix, locals, known, module_decls);
            // BUG-134: idem para `guard sea`; además el binding sigue vivo
            // DESPUÉS del bloque, así que se registra también en `locals`.
            let mut bound = HashSet::new();
            collect_pattern_bindings(pattern, &mut bound);
            locals.extend(bound);
            let mut guard_locals = locals.clone();
            for node in else_body.iter_mut() {
                prefix_node(node, prefix, &mut guard_locals, false, known, module_decls);
            }
        }
        Stmt::Match {
            expr,
            arms,
            default,
            ..
        } => {
            prefix_expr(expr, prefix, locals, known, module_decls);
            for arm in arms.iter_mut() {
                prefix_expr(&mut arm.value, prefix, locals, known, module_decls);
                if let Some(ref mut guard) = arm.guard {
                    prefix_expr(guard, prefix, locals, known, module_decls);
                }
                let mut arm_locals = locals.clone();
                for node in arm.body.iter_mut() {
                    prefix_node(node, prefix, &mut arm_locals, false, known, module_decls);
                }
            }
            if let Some(body) = default {
                let mut def_locals = locals.clone();
                for node in body.iter_mut() {
                    prefix_node(node, prefix, &mut def_locals, false, known, module_decls);
                }
            }
        }
        Stmt::Expr { expr, .. } => {
            prefix_expr(expr, prefix, locals, known, module_decls);
        }
        Stmt::FieldAssign { expr, value, .. } => {
            prefix_expr(expr, prefix, locals, known, module_decls);
            prefix_expr(value, prefix, locals, known, module_decls);
        }
        Stmt::ArraySet {
            arr, index, value, ..
        } => {
            prefix_expr(arr, prefix, locals, known, module_decls);
            prefix_expr(index, prefix, locals, known, module_decls);
            prefix_expr(value, prefix, locals, known, module_decls);
        }
        Stmt::Block { stmts, .. } => {
            let mut block_locals = locals.clone();
            for node in stmts.iter_mut() {
                prefix_node(node, prefix, &mut block_locals, false, known, module_decls);
            }
        }
        Stmt::Destructure { targets, value, .. } => {
            for target in targets.iter_mut() {
                if target.name != "_" && !locals.contains(target.name.as_str()) {
                    target.name = format!("{}_{}", prefix, target.name);
                }
            }
            prefix_expr(value, prefix, locals, known, module_decls);
        }
        Stmt::IfLet {
            pattern,
            value,
            then_body,
            else_body,
            ..
        } => {
            prefix_expr(value, prefix, locals, known, module_decls);
            // BUG-134: las variables que liga el patrón son LOCALES del cuerpo.
            let mut bound = HashSet::new();
            collect_pattern_bindings(pattern, &mut bound);
            let mut then_locals = locals.clone();
            then_locals.extend(bound);
            for node in then_body.iter_mut() {
                prefix_node(node, prefix, &mut then_locals, false, known, module_decls);
            }
            if let Some(body) = else_body {
                let mut else_locals = locals.clone();
                for node in body.iter_mut() {
                    prefix_node(node, prefix, &mut else_locals, false, known, module_decls);
                }
            }
        }
        Stmt::Posponer { body, .. } => {
            let mut posp_locals = locals.clone();
            for node in body.iter_mut() {
                prefix_node(node, prefix, &mut posp_locals, false, known, module_decls);
            }
        }
        Stmt::TryCatch {
            try_body,
            err_var,
            catch_body,
            ..
        } => {
            let mut try_locals = locals.clone();
            for node in try_body.iter_mut() {
                prefix_node(node, prefix, &mut try_locals, false, known, module_decls);
            }
            let mut catch_locals = locals.clone();
            catch_locals.insert(err_var.clone());
            for node in catch_body.iter_mut() {
                prefix_node(node, prefix, &mut catch_locals, false, known, module_decls);
            }
        }
        Stmt::InlineAsm { .. } | Stmt::InlineC { .. } | Stmt::InlineRust { .. } => {}
    }
}

/// BUG-134: recoge los nombres que un patrón de `si sea` / `guard sea`
/// introduce como variables locales (`si sea exito(datos) = r`, `algun(v)`,
/// `error(e)`, y las variantes de enum con carga). Sin esto el aplanador de
/// módulos los tomaba por identificadores globales y les ponía el prefijo del
/// módulo (`m_datos`), de modo que el cuerpo referenciaba una variable que no
/// existía: E033 en cuanto alguien importaba el módulo.
fn collect_pattern_bindings(pattern: &Expr, out: &mut HashSet<String>) {
    match pattern {
        Expr::Ident { name, .. } => {
            out.insert(name.clone());
        }
        Expr::Call { args, .. } => {
            for a in args.iter() {
                collect_pattern_bindings(a, out);
            }
        }
        Expr::EnumCtor { args, .. } => {
            for a in args.iter() {
                collect_pattern_bindings(a, out);
            }
        }
        Expr::Tuple { items, .. } => {
            for it in items.iter() {
                collect_pattern_bindings(it, out);
            }
        }
        // `exito(v)`, `error(e)`, `algun(v)` tienen nodo propio en el AST, no
        // son llamadas genéricas: sin estos brazos el binding se escapaba.
        Expr::Exito { expr, .. } | Expr::Error { expr, .. } | Expr::Algun { expr, .. } => {
            collect_pattern_bindings(expr, out);
        }
        _ => {}
    }
}

fn prefix_expr(
    expr: &mut Expr,
    prefix: &str,
    locals: &HashSet<String>,
    known: &HashSet<String>,
    module_decls: &HashSet<String>,
) {
    match expr {
        Expr::Int { .. } | Expr::Float { .. } | Expr::Str { .. } | Expr::Bool { .. } => {}
        Expr::Ident { name, .. } => {
            if !locals.contains(name.as_str())
                && !is_known_prefixed(name, known)
                && (module_decls.contains(name.as_str()) || !is_builtin(name))
            {
                *name = format!("{}_{}", prefix, name);
            }
        }
        Expr::Binary { left, right, .. } => {
            prefix_expr(left, prefix, locals, known, module_decls);
            prefix_expr(right, prefix, locals, known, module_decls);
        }
        Expr::Unary { operand, .. } => {
            prefix_expr(operand, prefix, locals, known, module_decls);
        }
        Expr::Call {
            callee,
            args,
            type_args,
            ..
        } => {
            prefix_expr(callee, prefix, locals, known, module_decls);
            for arg in args.iter_mut() {
                prefix_expr(arg, prefix, locals, known, module_decls);
            }
            for ta in type_args.iter_mut() {
                prefix_type(ta, prefix, known);
            }
        }
        Expr::Grouping { expr: inner, .. } => {
            prefix_expr(inner, prefix, locals, known, module_decls);
        }
        Expr::Cast {
            expr: inner,
            cast_type,
            ..
        } => {
            prefix_expr(inner, prefix, locals, known, module_decls);
            if let Type::Struct(name) = cast_type {
                if !name.is_empty() && name != "Infer" && !is_known_prefixed(name, known) {
                    let prefixed = format!("{}_{}", prefix, name);
                    *name = prefixed;
                }
            } else if let Type::Lista(inner_t) = cast_type {
                if let Type::Struct(name) = inner_t.as_mut() {
                    if !name.is_empty() && name != "Infer" && !is_known_prefixed(name, known) {
                        let prefixed = format!("{}_{}", prefix, name);
                        *name = prefixed;
                    }
                }
            }
        }
        Expr::List { items, .. } => {
            for item in items.iter_mut() {
                prefix_expr(item, prefix, locals, known, module_decls);
            }
        }
        Expr::Range { start, end, .. } => {
            prefix_expr(start, prefix, locals, known, module_decls);
            prefix_expr(end, prefix, locals, known, module_decls);
        }
        Expr::Index {
            expr: target,
            index,
            ..
        } => {
            prefix_expr(target, prefix, locals, known, module_decls);
            prefix_expr(index, prefix, locals, known, module_decls);
        }
        Expr::MethodCall {
            expr: target, args, ..
        } => {
            prefix_expr(target, prefix, locals, known, module_decls);
            for arg in args.iter_mut() {
                prefix_expr(arg, prefix, locals, known, module_decls);
            }
        }
        Expr::Lambda { params, body, .. } => {
            let mut lambda_locals = locals.clone();
            for p in params.iter_mut() {
                prefix_type(&mut p.param_type, prefix, known);
                lambda_locals.insert(p.name.clone());
            }
            for node in body.iter_mut() {
                prefix_node(node, prefix, &mut lambda_locals, false, known, module_decls);
            }
        }
        Expr::StructInit {
            struct_name,
            fields,
            type_args,
            ..
        } => {
            // BUG-133: este era el único sitio que prefijaba un nombre de
            // struct SIN comprobar `is_known_prefixed`. En una cadena de
            // importaciones (`app` -> `nn` -> `tensor`), un literal
            // `GrafoAutograd { ... }` escrito dentro de `tensor.nv` ya había
            // recibido su prefijo al aplanar `tensor`, y al aplanar `nn` se le
            // volvía a aplicar: `nn_tensor_GrafoAutograd`, un tipo que no
            // existe (E062). La declaración sí estaba protegida, así que
            // declaración y uso dejaban de coincidir.
            if !is_known_prefixed(struct_name, known) {
                *struct_name = format!("{}_{}", prefix, struct_name);
            }
            for (_, value) in fields.iter_mut() {
                prefix_expr(value, prefix, locals, known, module_decls);
            }
            for ta in type_args.iter_mut() {
                prefix_type(ta, prefix, known);
            }
        }
        Expr::FieldAccess { expr: target, .. } => {
            prefix_expr(target, prefix, locals, known, module_decls);
        }
        Expr::Exito { expr: inner, .. } => {
            prefix_expr(inner, prefix, locals, known, module_decls);
        }
        Expr::Error { expr: inner, .. } => {
            prefix_expr(inner, prefix, locals, known, module_decls);
        }
        Expr::Intentar { expr: inner, .. } => {
            prefix_expr(inner, prefix, locals, known, module_decls);
        }
        Expr::Algun { expr: inner, .. } => {
            prefix_expr(inner, prefix, locals, known, module_decls);
        }
        Expr::Ninguno { .. } => {}
        Expr::Tuple { items, .. } => {
            for item in items.iter_mut() {
                prefix_expr(item, prefix, locals, known, module_decls);
            }
        }
        Expr::TupleAccess { expr: target, .. } => {
            prefix_expr(target, prefix, locals, known, module_decls);
        }
        Expr::EnumCtor {
            enum_name, args, ..
        } => {
            if !locals.contains(enum_name.as_str())
                && !is_known_prefixed(enum_name, known)
                && (module_decls.contains(enum_name.as_str()) || !is_builtin(enum_name))
            {
                *enum_name = format!("{}_{}", prefix, enum_name);
            }
            for arg in args.iter_mut() {
                prefix_expr(arg, prefix, locals, known, module_decls);
            }
        }
        Expr::Ternary {
            condition,
            true_branch,
            false_branch,
            ..
        } => {
            prefix_expr(condition, prefix, locals, known, module_decls);
            prefix_expr(true_branch, prefix, locals, known, module_decls);
            prefix_expr(false_branch, prefix, locals, known, module_decls);
        }
        Expr::SafeFieldAccess { expr: target, .. } => {
            prefix_expr(target, prefix, locals, known, module_decls);
        }
        Expr::Elvis {
            expr: target,
            default,
            ..
        } => {
            prefix_expr(target, prefix, locals, known, module_decls);
            prefix_expr(default, prefix, locals, known, module_decls);
        }
        Expr::Comprehension {
            expr: inner,
            var_name,
            iter,
            condition,
            ..
        } => {
            prefix_expr(iter, prefix, locals, known, module_decls);
            let mut comp_locals = locals.clone();
            comp_locals.insert(var_name.clone());
            prefix_expr(inner, prefix, &comp_locals, known, module_decls);
            if let Some(cond) = condition {
                prefix_expr(cond, prefix, &comp_locals, known, module_decls);
            }
        }
        Expr::Query {
            var_name,
            source,
            where_clause,
            order_by,
            select_expr,
            ..
        } => {
            prefix_expr(source, prefix, locals, known, module_decls);
            let mut q_locals = locals.clone();
            q_locals.insert(var_name.clone());
            if let Some(w) = where_clause {
                prefix_expr(w, prefix, &q_locals, known, module_decls);
            }
            if let Some(o) = order_by {
                prefix_expr(o, prefix, &q_locals, known, module_decls);
            }
            prefix_expr(select_expr, prefix, &q_locals, known, module_decls);
        }
        Expr::Esperar { expr, .. } => {
            prefix_expr(expr, prefix, locals, known, module_decls);
        }
        Expr::Comptime { expr, .. } => {
            prefix_expr(expr, prefix, locals, known, module_decls);
        }
    }
}

fn prefix_type_with_params(
    t: &mut Type,
    prefix: &str,
    type_params: &HashSet<String>,
    known: &HashSet<String>,
) {
    match t {
        Type::Struct(name) if type_params.contains(name.as_str()) => {
            // Don't prefix type parameter names
        }
        Type::GenericStruct { name, args } => {
            if !type_params.contains(name.as_str()) && !is_known_prefixed(name, known) {
                *name = format!("{}_{}", prefix, name);
            }
            for arg in args.iter_mut() {
                prefix_type_with_params(arg, prefix, type_params, known);
            }
        }
        Type::Lista(inner) => prefix_type_with_params(inner, prefix, type_params, known),
        Type::Func {
            param_types,
            return_type,
        } => {
            for p in param_types.iter_mut() {
                prefix_type_with_params(p, prefix, type_params, known);
            }
            prefix_type_with_params(return_type, prefix, type_params, known);
        }
        Type::Struct(name) if name == "Self" || name == "self" || name == "este" => {}
        Type::Struct(name) if name != "Infer" && !is_known_prefixed(name, known) => {
            *name = format!("{}_{}", prefix, name);
        }
        Type::Resultado { ok, err } => {
            prefix_type_with_params(ok, prefix, type_params, known);
            prefix_type_with_params(err, prefix, type_params, known);
        }
        Type::Opcion(inner) => {
            prefix_type_with_params(inner, prefix, type_params, known);
        }
        Type::Tuple(types) => {
            for t in types.iter_mut() {
                prefix_type_with_params(t, prefix, type_params, known);
            }
        }
        Type::Prestado { inner, .. } => {
            prefix_type_with_params(inner, prefix, type_params, known);
        }
        Type::Dueno(inner) => {
            prefix_type_with_params(inner, prefix, type_params, known);
        }
        _ => {}
    }
}

fn prefix_type(t: &mut Type, prefix: &str, known: &HashSet<String>) {
    match t {
        Type::Lista(inner) => prefix_type(inner, prefix, known),
        Type::Prestado { inner, .. } => prefix_type(inner, prefix, known),
        Type::Dueno(inner) => prefix_type(inner, prefix, known),
        Type::GenericStruct { name, args } => {
            if !is_known_prefixed(name, known) {
                *name = format!("{}_{}", prefix, name);
            }
            for arg in args.iter_mut() {
                prefix_type(arg, prefix, known);
            }
        }
        Type::Func {
            param_types,
            return_type,
        } => {
            for p in param_types.iter_mut() {
                prefix_type(p, prefix, known);
            }
            prefix_type(return_type, prefix, known);
        }
        Type::Struct(name) => {
            if name != "Infer"
                && name != "Self"
                && name != "self"
                && name != "este"
                && !is_known_prefixed(name, known)
            {
                *name = format!("{}_{}", prefix, name);
            }
        }
        Type::Resultado { ok, err } => {
            prefix_type(ok, prefix, known);
            prefix_type(err, prefix, known);
        }
        Type::Opcion(inner) => {
            prefix_type(inner, prefix, known);
        }
        Type::Tuple(types) => {
            for t in types.iter_mut() {
                prefix_type(t, prefix, known);
            }
        }
        _ => {}
    }
}

fn is_known_prefixed(name: &str, known: &HashSet<String>) -> bool {
    known
        .iter()
        .any(|p| !p.is_empty() && name.starts_with(&format!("{}_", p)))
}

/// BUG-103: expuesto para que el analizador pueda avisar cuando una función
/// del usuario choca con un builtin.
pub fn is_builtin(name: &str) -> bool {
    if name.starts_with("__") {
        return true;
    }
    matches!(
        name,
        "imprimir"
            | "print"
            | "largo"
            | "len"
            | "agregar"
            | "push"
            | "leer"
            | "read"
            | "exito"
            | "ok"
            | "error"
            | "err"
            | "algun"
            | "some"
            | "ninguno"
            | "none"
            | "a_texto"
            | "to_texto"
            | "a_entero"
            | "to_int"
            | "to_entero"
            | "a_decimal"
            | "to_float"
            | "a_numero"
            | "to_number"
            | "a_entero_seguro"
            | "to_int_safe"
            | "a_decimal_seguro"
            | "to_float_safe"
            | "es_numero"
            | "is_number"
            | "abs"
            | "absoluto"
            | "minimo"
            | "min"
            | "maximo"
            | "max"
            | "raiz"
            | "sqrt"
            | "potencia"
            | "pow"
            | "piso"
            | "floor"
            | "techo"
            | "ceil"
            | "redondear"
            | "round"
            | "intentar"
            | "try"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_loader_resolves_imports_from_memory() {
        let mut mem = HashMap::new();
        mem.insert(
            "util_mem.nv".to_string(),
            "funcion entero duplicar(entero x) {\n    retornar x * 2;\n}\n".to_string(),
        );
        let mut loader = ModuleLoader::with_memory_files(mem);
        let source = "importar \"util_mem.nv\";\nfuncion entero main() {\n    retornar util_mem__duplicar(21);\n}\n";
        let program = loader
            .resolve_imports(source, Path::new("__lumen_mem__/main.nv"))
            .expect("debe resolver imports desde memoria");
        let text = format!("{:?}", program);
        assert!(
            text.contains("util_mem__duplicar"),
            "la función importada debe estar prefijada: {}",
            text
        );
    }

    #[test]
    fn test_memory_loader_fallback_to_disk_still_works() {
        let mut loader = ModuleLoader::with_default_search_paths();
        let program = loader
            .resolve_imports(
                "funcion entero main() { retornar 42; }",
                Path::new("__lumen_mem__/main.nv"),
            )
            .expect("código sin imports debe resolver");
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_memory_loader_missing_module_reports_error() {
        let mut loader = ModuleLoader::with_memory_files(HashMap::new());
        let result = loader.resolve_imports(
            "importar \"no_existe.nv\";",
            Path::new("__lumen_mem__/main.nv"),
        );
        assert!(matches!(result, Err(ModuleError::Io { .. })));
    }
}
