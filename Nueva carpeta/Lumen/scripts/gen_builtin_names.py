#!/usr/bin/env python3
"""Genera la lista BUILTIN_NAMES para vm.rs a partir de los literales
`name == "..."` de call_core_builtin / call_extra_builtin."""
import re, sys

SRC = "crates/lumen-vm/src/vm.rs"
s = open(SRC).read()
# rangos de ambas funciones de builtins
def fn_range(marker):
    i = s.index(marker)
    # fin: la siguiente "    fn " o el final de la funcion (aproximado por dedent)
    j = s.index("\n    fn ", i + 10)
    return s[i:j]
core = fn_range("    fn call_core_builtin(")
extra = ""
try:
    extra = fn_range("    fn call_extra_builtin(")
except ValueError:
    pass
names = set(re.findall(r'name == "([^"]+)"', core))
names |= set(re.findall(r'name == "([^"]+)"', extra))
# además: literales de ramas `name != "..."` o matches directos
names |= set(re.findall(r'name == "([^"]+)"', s))  # TODOS (superset seguro)
names = sorted(names)
print(f"// {len(names)} nombres (generado por scripts/gen_builtin_names.py)")
out = "pub(crate) const BUILTIN_NAMES: &[&str] = &[\n"
for n in names:
    out += f'    "{n}",\n'
out += "];\n"
sys.stdout.write(out)
