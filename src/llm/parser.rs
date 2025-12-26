use crate::error::{Error, Result};
use crate::models::analysis::LLMAnalysisResult;

pub fn parse_llm_response(response: &str) -> Result<LLMAnalysisResult> {
    let json_str = extract_json(response)?;

    serde_json::from_str(&json_str)
        .map_err(|e| Error::ParseError(format!("Failed to parse LLM response: {}", e)))
}

fn extract_json(text: &str) -> Result<String> {
    // Try to find JSON block in markdown code blocks
    if let Some(start) = text.find("```json") {
        let start = start + 7;
        if let Some(end) = text[start..].find("```") {
            return Ok(text[start..start + end].trim().to_string());
        }
    }

    // Try plain code block
    if let Some(start) = text.find("```") {
        let start = start + 3;
        // Skip any language identifier on the same line
        let start = text[start..]
            .find('\n')
            .map(|i| start + i + 1)
            .unwrap_or(start);
        if let Some(end) = text[start..].find("```") {
            let content = text[start..start + end].trim();
            if content.starts_with('{') {
                return Ok(content.to_string());
            }
        }
    }

    // Try to find raw JSON object
    if let Some(start) = text.find('{') {
        let mut depth = 0;
        let mut end = start;
        let mut in_string = false;
        let mut escape_next = false;

        for (i, c) in text[start..].chars().enumerate() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match c {
                '\\' if in_string => escape_next = true,
                '"' => in_string = !in_string,
                '{' if !in_string => depth += 1,
                '}' if !in_string => {
                    depth -= 1;
                    if depth == 0 {
                        end = start + i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }

        if depth == 0 && end > start {
            return Ok(text[start..end].to_string());
        }
    }

    Err(Error::ParseError("No valid JSON found in response".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_from_markdown() {
        let input = r#"Here's the analysis:
```json
{"skills": []}
```
"#;
        let result = extract_json(input).unwrap();
        assert_eq!(result, r#"{"skills": []}"#);
    }

    #[test]
    fn test_extract_raw_json() {
        let input = r#"The result is {"skills": [], "patterns": []}"#;
        let result = extract_json(input).unwrap();
        assert_eq!(result, r#"{"skills": [], "patterns": []}"#);
    }
}
