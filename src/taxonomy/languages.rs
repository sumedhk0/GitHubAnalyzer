use std::collections::HashMap;

pub fn detect_language(filename: &str) -> Option<String> {
    // Handle special filenames first
    let lower = filename.to_lowercase();
    if lower == "dockerfile" || lower.starts_with("dockerfile.") {
        return Some("Dockerfile".to_string());
    }
    if lower == "makefile" || lower == "gnumakefile" {
        return Some("Makefile".to_string());
    }
    if lower == "cmakelists.txt" {
        return Some("CMake".to_string());
    }
    if lower.ends_with(".d.ts") {
        return Some("TypeScript".to_string());
    }

    let extension = filename.rsplit('.').next()?;

    let lang_map: HashMap<&str, &str> = [
        // Rust
        ("rs", "Rust"),
        // Python
        ("py", "Python"),
        ("pyw", "Python"),
        ("pyx", "Python"),
        // JavaScript/TypeScript
        ("js", "JavaScript"),
        ("mjs", "JavaScript"),
        ("cjs", "JavaScript"),
        ("ts", "TypeScript"),
        ("tsx", "TypeScript"),
        ("jsx", "JavaScript"),
        // Go
        ("go", "Go"),
        // Java/JVM
        ("java", "Java"),
        ("kt", "Kotlin"),
        ("kts", "Kotlin"),
        ("scala", "Scala"),
        ("clj", "Clojure"),
        ("groovy", "Groovy"),
        // C family
        ("c", "C"),
        ("h", "C"),
        ("cpp", "C++"),
        ("cc", "C++"),
        ("cxx", "C++"),
        ("hpp", "C++"),
        ("hxx", "C++"),
        // C#
        ("cs", "C#"),
        // Swift/Objective-C
        ("swift", "Swift"),
        ("m", "Objective-C"),
        ("mm", "Objective-C++"),
        // Ruby
        ("rb", "Ruby"),
        ("rake", "Ruby"),
        ("gemspec", "Ruby"),
        // PHP
        ("php", "PHP"),
        // Elixir/Erlang
        ("ex", "Elixir"),
        ("exs", "Elixir"),
        ("erl", "Erlang"),
        // Haskell
        ("hs", "Haskell"),
        ("lhs", "Haskell"),
        // Functional
        ("ml", "OCaml"),
        ("mli", "OCaml"),
        ("fs", "F#"),
        ("fsx", "F#"),
        // Shell
        ("sh", "Shell"),
        ("bash", "Shell"),
        ("zsh", "Shell"),
        ("fish", "Shell"),
        ("ps1", "PowerShell"),
        ("psm1", "PowerShell"),
        // Web
        ("html", "HTML"),
        ("htm", "HTML"),
        ("css", "CSS"),
        ("scss", "SCSS"),
        ("sass", "Sass"),
        ("less", "Less"),
        ("vue", "Vue"),
        ("svelte", "Svelte"),
        // Data/Query
        ("sql", "SQL"),
        ("graphql", "GraphQL"),
        ("gql", "GraphQL"),
        // Config/Data
        ("json", "JSON"),
        ("yaml", "YAML"),
        ("yml", "YAML"),
        ("toml", "TOML"),
        ("xml", "XML"),
        ("ini", "INI"),
        // Documentation
        ("md", "Markdown"),
        ("markdown", "Markdown"),
        ("rst", "reStructuredText"),
        ("txt", "Text"),
        // Lua
        ("lua", "Lua"),
        // R
        ("r", "R"),
        ("rmd", "R"),
        // Perl
        ("pl", "Perl"),
        ("pm", "Perl"),
        // Dart
        ("dart", "Dart"),
        // Zig
        ("zig", "Zig"),
        // Nim
        ("nim", "Nim"),
        // Julia
        ("jl", "Julia"),
        // V
        ("v", "V"),
        // Solidity
        ("sol", "Solidity"),
        // Move
        ("move", "Move"),
        // Proto
        ("proto", "Protocol Buffers"),
        // Terraform
        ("tf", "Terraform"),
        ("tfvars", "Terraform"),
    ]
    .iter()
    .cloned()
    .collect();

    lang_map
        .get(extension.to_lowercase().as_str())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language("main.rs"), Some("Rust".to_string()));
        assert_eq!(detect_language("app.py"), Some("Python".to_string()));
        assert_eq!(detect_language("index.tsx"), Some("TypeScript".to_string()));
        assert_eq!(detect_language("Dockerfile"), Some("Dockerfile".to_string()));
        assert_eq!(detect_language("types.d.ts"), Some("TypeScript".to_string()));
    }
}
