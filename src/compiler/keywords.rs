// src/compiler/keywords.rs
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
    Spanish,
    English,
}

pub struct Keywords {
    lang: Language,
    keywords: HashMap<&'static str, &'static str>,
}

impl Keywords {
    pub fn new(lang: Language) -> Self {
        let mut keywords = HashMap::new();
        
        match lang {
            Language::Spanish => {
                keywords.insert("numero", "number");
                keywords.insert("imprimir", "print");
                keywords.insert("si", "if");
                keywords.insert("sino", "else");
                keywords.insert("mientras", "while");
            }
            Language::English => {
                keywords.insert("number", "number");
                keywords.insert("print", "print");
                keywords.insert("if", "if");
                keywords.insert("else", "else");
                keywords.insert("while", "while");
            }
        }
        
        Keywords { lang, keywords }
    }
    
    pub fn get_language(&self) -> Language {
        self.lang
    }
    
    pub fn is_keyword(&self, word: &str) -> bool {
        self.keywords.contains_key(word)
    }
    
    pub fn is_number(&self, word: &str) -> bool {
        matches!(word, "numero" | "number")
    }
    
    pub fn is_print(&self, word: &str) -> bool {
        matches!(word, "imprimir" | "print")
    }
    
    pub fn is_if(&self, word: &str) -> bool {
        matches!(word, "si" | "if")
    }
    
    pub fn is_else(&self, word: &str) -> bool {
        matches!(word, "sino" | "else")
    }
    
    pub fn is_while(&self, word: &str) -> bool {
        matches!(word, "mientras" | "while")
    }
}

pub fn detect_language(source: &str) -> Language {
    if source.contains("numero") || source.contains("imprimir") || source.contains("mientras") {
        Language::Spanish
    } else {
        Language::English
    }
}
