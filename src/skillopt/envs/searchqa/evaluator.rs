use std::collections::HashMap;

/// Normalize answer string following SQuAD conventions:
/// - lowercase
/// - remove punctuation
/// - remove English articles ("a", "an", "the")
/// - collapse multiple whitespace characters
pub fn normalize_answer(s: &str) -> String {
    let lower = s.to_lowercase();
    let no_punct: String = lower
        .chars()
        .filter(|c| !c.is_ascii_punctuation())
        .collect();

    let tokens: Vec<&str> = no_punct
        .split_whitespace()
        .filter(|&w| w != "a" && w != "an" && w != "the")
        .collect();

    tokens.join(" ")
}

/// Extract answer from `<answer>...</answer>` tags.
/// Fallback: last non-empty line of the response.
pub fn extract_answer(text: &str) -> String {
    let lower = text.to_lowercase();
    if let Some(start_tag) = lower.rfind("<answer>") {
        let after_start = start_tag + "<answer>".len();
        if let Some(end_tag) = lower[after_start..].find("</answer>") {
            return text[after_start..after_start + end_tag].trim().to_string();
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

/// Compute Exact Match (EM) against a set of gold answers.
pub fn exact_match(prediction: &str, gold_answers: &[String]) -> f64 {
    let norm_pred = normalize_answer(prediction);
    for gold in gold_answers {
        if normalize_answer(gold) == norm_pred {
            return 1.0;
        }
    }
    0.0
}

/// Compute token-level SQuAD-style F1 score across gold answers.
pub fn f1_score(prediction: &str, gold_answers: &[String]) -> f64 {
    let norm_pred = normalize_answer(prediction);
    let pred_tokens: Vec<&str> = norm_pred.split_whitespace().collect();

    if pred_tokens.is_empty() {
        for gold in gold_answers {
            if normalize_answer(gold).split_whitespace().next().is_none() {
                return 1.0;
            }
        }
        return 0.0;
    }

    let mut pred_counts = HashMap::new();
    for &tok in &pred_tokens {
        *pred_counts.entry(tok).or_insert(0) += 1;
    }

    let mut best_f1: f64 = 0.0;
    for gold in gold_answers {
        let norm_gold = normalize_answer(gold);
        let gold_tokens: Vec<&str> = norm_gold.split_whitespace().collect();
        if gold_tokens.is_empty() {
            continue;
        }

        let mut gold_counts = HashMap::new();
        for &tok in &gold_tokens {
            *gold_counts.entry(tok).or_insert(0) += 1;
        }

        let mut common = 0;
        for (tok, &p_count) in &pred_counts {
            if let Some(&g_count) = gold_counts.get(tok) {
                common += p_count.min(g_count);
            }
        }

        if common == 0 {
            continue;
        }

        let precision = (common as f64) / (pred_tokens.len() as f64);
        let recall = (common as f64) / (gold_tokens.len() as f64);
        let f1 = 2.0 * precision * recall / (precision + recall);
        best_f1 = best_f1.max(f1);
    }

    best_f1
}

/// 1.0 if any normalized gold is a substring of prediction, or vice versa.
pub fn sub_em(prediction: &str, gold_answers: &[String]) -> f64 {
    let norm_pred = normalize_answer(prediction);
    if norm_pred.is_empty() {
        return 0.0;
    }
    for gold in gold_answers {
        let norm_gold = normalize_answer(gold);
        if !norm_gold.is_empty() && (norm_pred.contains(&norm_gold) || norm_gold.contains(&norm_pred)) {
            return 1.0;
        }
    }
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_answer() {
        assert_eq!(normalize_answer("The quick brown fox!"), "quick brown fox");
        assert_eq!(normalize_answer("A cat, a dog."), "cat dog");
    }

    #[test]
    fn test_extract_answer() {
        assert_eq!(
            extract_answer("Analysis...\n<answer>42</answer>\nOther stuff"),
            "42"
        );
        assert_eq!(
            extract_answer("Step 1\nStep 2\nFinal result: 100"),
            "Final result: 100"
        );
    }

    #[test]
    fn test_exact_match() {
        let golds = vec!["Albert Einstein".to_string(), "Einstein".to_string()];
        assert_eq!(exact_match("albert einstein", &golds), 1.0);
        assert_eq!(exact_match("einstein", &golds), 1.0);
        assert_eq!(exact_match("Isaac Newton", &golds), 0.0);
    }

    #[test]
    fn test_f1_score() {
        let golds = vec!["the president of united states".to_string()];
        let pred = "president united states";
        let f1 = f1_score(pred, &golds);
        assert!((f1 - 0.8571428571428571).abs() < 1e-6);

        let exact_golds = vec!["president united states".to_string()];
        assert_eq!(f1_score(pred, &exact_golds), 1.0);
    }
}
