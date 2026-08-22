use lumen_ir::ir::Program;
use lumen_lexer::token::Token;
use lumen_parser::ast::DeclOrStmt;

pub trait Plugin: Send + Sync {
    /// Nombre único del plugin
    fn name(&self) -> &'static str;
    /// Hook post-lexer: recibe tokens antes del parser
    fn on_tokens(&self, _tokens: &[Token]) -> Vec<String> {
        vec![]
    }
    /// Hook post-parse: recibe el AST completo
    fn on_ast(&self, _ast: &[DeclOrStmt]) -> Vec<String> {
        vec![]
    }
    /// Hook post-sema: recibe el AST con tipos resueltos
    fn on_sema(&self, _ast: &[DeclOrStmt]) -> Vec<String> {
        vec![]
    }
    /// Hook pre-IR: permite modificar el programa IR antes de codegen
    fn on_ir(&self, _ir: &mut Program) -> Vec<String> {
        vec![]
    }
}

pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { plugins: vec![] }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn run_tokens(&self, tokens: &[Token]) -> Vec<String> {
        let mut all = vec![];
        for p in &self.plugins {
            all.extend(p.on_tokens(tokens));
        }
        all
    }

    pub fn run_ast(&self, ast: &[DeclOrStmt]) -> Vec<String> {
        let mut all = vec![];
        for p in &self.plugins {
            all.extend(p.on_ast(ast));
        }
        all
    }

    pub fn run_sema(&self, ast: &[DeclOrStmt]) -> Vec<String> {
        let mut all = vec![];
        for p in &self.plugins {
            all.extend(p.on_sema(ast));
        }
        all
    }

    pub fn run_ir(&self, ir: &mut Program) {
        for p in &self.plugins {
            p.on_ir(ir);
        }
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumen_ir::ir::{Func, Instr};

    struct LogPlugin;
    impl Plugin for LogPlugin {
        fn name(&self) -> &'static str {
            "log_plugin"
        }
        fn on_tokens(&self, tokens: &[Token]) -> Vec<String> {
            vec![format!("{} tokens", tokens.len())]
        }
    }

    struct StripNopPlugin;
    impl Plugin for StripNopPlugin {
        fn name(&self) -> &'static str {
            "strip_nop"
        }
        fn on_ir(&self, ir: &mut Program) -> Vec<String> {
            for func in ir.funcs.values_mut() {
                func.instrs.retain(|i| !matches!(i, Instr::Nop));
            }
            vec![]
        }
    }

    #[test]
    fn test_plugin_registry() {
        let mut reg = PluginRegistry::new();
        reg.register(Box::new(LogPlugin));
        reg.register(Box::new(StripNopPlugin));

        let tokens = vec![];
        let msgs = reg.run_tokens(&tokens);
        assert_eq!(msgs, vec!["0 tokens"]);

        let mut program = Program::new();
        program.entry = "main".into();
        program.funcs.insert(
            "main".into(),
            Func {
                name: "main".into(),
                params: vec![],
                defaults: vec![],
                entry: 0,
                instrs: vec![Instr::Nop, Instr::ConstInt(42), Instr::Nop, Instr::Return],
            },
        );
        reg.run_ir(&mut program);
        assert_eq!(program.funcs["main"].instrs.len(), 2);
    }
}
