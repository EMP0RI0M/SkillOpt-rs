use serde::{Deserialize, Serialize};

pub const DEFAULT_ANLS_THRESHOLD: f64 = 0.5;

/// Normalize text by lowercasing and collapsing whitespace.
pub fn normalize_text(value: &str) -> String {
    let lower = value.trim().to_lowercase();
    lower.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Compute Levenshtein edit distance between two strings.
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    if a == b {
        return 0;
    }
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let mut previous: Vec<usize> = (0..=b_chars.len()).collect();
    for (i, &char_a) in a_chars.iter().enumerate() {
        let mut current = vec![i + 1];
        for (j, &char_b) in b_chars.iter().enumerate() {
            let insert_cost = current[j] + 1;
            let delete_cost = previous[j + 1] + 1;
            let replace_cost = previous[j] + if char_a == char_b { 0 } else { 1 };
            current.push(insert_cost.min(delete_cost).min(replace_cost));
        }
        previous = current;
    }

    previous[b_chars.len()]
}

/// Compute Average Normalized Levenshtein Similarity (ANLS) between predicted and target.
pub fn score_single_answer(predicted: &str, target: &str, threshold: f64) -> f64 {
    let pred_norm = normalize_text(predicted);
    let target_norm = normalize_text(target);

    if pred_norm.is_empty() && target_norm.is_empty() {
        return 1.0;
    }
    if pred_norm.is_empty() || target_norm.is_empty() {
        return 0.0;
    }

    let dist = levenshtein_distance(&pred_norm, &target_norm) as f64;
    let max_len = pred_norm.len().max(target_norm.len()) as f64;
    let norm_dist = dist / max_len;

    if norm_dist >= threshold {
        0.0
    } else {
        1.0 - norm_dist
    }
}

/// Extract answer from `<answer>...</answer>` tags or fallback to last non-empty line.
pub fn extract_answer(text: &str) -> String {
    let lower = text.to_lowercase();
    if let Some(start) = lower.rfind("<answer>") {
        let after_start = start + "<answer>".len();
        if let Some(end) = lower[after_start..].rfind("</answer>") {
            return text[after_start..after_start + end].trim().to_string();
        }
    }
    for line in text.lines().rev() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    text.trim().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocVQAEvalResult {
    pub anls: f64,
    pub hard: f64,
    pub predicted_answer: String,
    pub gold_answers: Vec<String>,
}

/// Evaluate prediction text against a list of gold answers using ANLS and Exact Match.
pub fn evaluate_docvqa(prediction_text: &str, gold_answers: &[String]) -> DocVQAEvalResult {
    let answer = extract_answer(prediction_text);
    let mut best_anls: f64 = 0.0;
    let mut exact = 0.0;

    for target in gold_answers {
        let anls = score_single_answer(&answer, target, DEFAULT_ANLS_THRESHOLD);
        if anls > best_anls {
            best_anls = anls;
        }
        if normalize_text(&answer) == normalize_text(target) {
            exact = 1.0;
        }
    }

    DocVQAEvalResult {
        anls: best_anls,
        hard: exact,
        predicted_answer: answer,
        gold_answers: gold_answers.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("flaw", "lawn"), 2);
    }

    #[test]
    fn test_score_single_answer() {
        // Exact match -> 1.0
        assert_eq!(score_single_answer("invoice 123", "invoice 123", 0.5), 1.0);
        // Small typo -> High similarity
        let s = score_single_answer("invoice 123", "invoice 124", 0.5);
        assert!(s > 0.8);
        // Completely different -> 0.0 (exceeds threshold)
        assert_eq!(score_single_answer("apple", "banana", 0.5), 0.0);
    }

    #[test]
    fn test_evaluate_docvqa() {
        let golds = vec!["$4,500.00".to_string(), "4500".to_string()];
        let res = evaluate_docvqa("Result:\n<answer>$4,500.00</answer>", &golds);
        assert_eq!(res.hard, 1.0);
        assert_eq!(res.anls, 1.0);
        assert_eq!(res.predicted_answer, "$4,500.00");
    }
}
