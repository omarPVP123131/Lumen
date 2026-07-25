use lumen_ir::ir::{Instr, Op, Program};

/// Compila un programa IR de LÚMEN a código C (primer paso hacia AOT).
/// La salida es código C que puede compilarse con gcc/clang.
pub fn compile_to_c(program: &Program) -> String {
    let mut out = String::new();
    out.push_str("#include <stdint.h>\n");
    out.push_str("#include <stdio.h>\n");
    out.push_str("#include <stdlib.h>\n");
    out.push_str("#include <string.h>\n\n");

    out.push_str("// LUMEN AOT — compilado desde IR a C\n\n");

    for (name, func) in &program.funcs {
        out.push_str(&format!("// funcion: {}\n", name));
        for instr in &func.instrs {
            out.push_str(&format!("  // {:?}\n", instr));
        }
        out.push_str("\n");
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumen_ir::ir::{Func, Program};
    use std::collections::BTreeMap;

    #[test]
    fn test_empty_program() {
        let program = Program::new();
        let c_code = compile_to_c(&program);
        assert!(c_code.contains("#include"));
    }

    #[test]
    fn test_function_output() {
        let mut funcs = BTreeMap::new();
        funcs.insert(
            "main".to_string(),
            Func {
                name: "main".to_string(),
                params: vec![],
                entry: 0,
                instrs: vec![Instr::ConstInt(42), Instr::Return],
            },
        );
        let program = Program {
            funcs,
            entry: "main".to_string(),
        };
        let c_code = compile_to_c(&program);
        assert!(c_code.contains("main"));
    }
}
