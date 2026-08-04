# Genera compiler_v4.nv: concatenación autocontenida
$header = @'
// compiler_v4.nv - Compilador autocontenido (self-hosting)
// Generado: concatenación de lexer.nv + parser.nv + codegen.nv + main
// Sin imports - el parser LÚMEN puede compilarlo completo
'@
$lexer = Get-Content "lexer.nv" -Raw
$parser = Get-Content "parser.nv" -Raw
$codegen = Get-Content "codegen.nv" -Raw
$main = @'
// ── Main: compila el target indicado en target.txt ──
// target.txt contiene dos líneas: ruta_entrada .nv / ruta_salida .nvc
funcion numero ejecutar_pipeline() {
    texto tgt = intentar __leer_archivo("stdlib/compiler/target.txt");
    lista<texto> cs = __str_a_caracteres(tgt);
    entero tn = largo(cs);
    entero ti = 0;
    mientras ti < tn && cs[ti] != "\n" && cs[ti] != "\r" { ti = ti + 1; }
    texto entrada = __str_subcadena(tgt, 0, ti);
    entero t2 = ti + 1;
    entero te = t2;
    mientras te < tn && cs[te] != "\n" && cs[te] != "\r" { te = te + 1; }
    texto salida = __str_subcadena(tgt, t2, te);
    imprimir("ENTRADA: ", entrada);
    imprimir("SALIDA: ", salida);

    texto codigo = intentar __leer_archivo(entrada);
    imprimir("Source: ", largo(codigo), " bytes");

    numero tk = lexer_tokenizar(codigo);
    imprimir("Tokens: ", __map_obtener(tk, "cnt"));

    // base = directorio del archivo de entrada (para imports relativos)
    texto base = "";
    entero ib = largo(entrada);
    mientras ib > 0 {
        texto c = __str_subcadena(entrada, ib - 1, ib);
        si (c == "/" || c == "\\") { romper; }
        ib = ib - 1;
    }
    base = __str_subcadena(entrada, 0, ib);

    numero ast = parser_parsear_con_base(tk, base);
    imprimir("AST: ", __map_obtener(ast, "tipo"));

    numero cg = codegen_generar(ast);
    imprimir("Instrs: ", __map_obtener(cg, "pos"));

    lista<entero> bytes = __codegen_a_nvc(cg);
    intentar __escribir_archivo_bin(salida, bytes);
    imprimir("OK: ", salida, " (", largo(bytes), " bytes)");
    retornar 0;
}
numero fin = ejecutar_pipeline();
si (__tipo_de(fin) == "Error") { imprimir("FALLO: ", fin); }
'@
$content = "$header`n$lexer`n$parser`n$codegen`n$main"
[System.IO.File]::WriteAllText("compiler_v4.nv", $content, (New-Object System.Text.UTF8Encoding $false))
Write-Host "compiler_v4.nv creado: $((Get-Item compiler_v4.nv).Length) bytes"
