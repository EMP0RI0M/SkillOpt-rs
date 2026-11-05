// Converted via p2r
"SearchQA evaluation metrics: Exact Match, F1, and Substring Match.

Normalization follows the SQuAD convention:
  - lowercase
  - remove punctuation
  - remove articles (a, an, the)
  - collapse whitespace

Answer extraction looks for <answer>...</answer> XML tags,
falling back to the last non-empty line of the response.
";
/*! Normalize answer string (SQuAD convention). */
fn normalize_answer(s: String) -> String {


let mut s = s.lower();
let mut s = "".join(s.into_iter().filter_map(|ch| { if ch !contains(TODO) string.punctuation { Some(ch) } else { None }  }));
let mut s = re.sub("\b(a|an|the)\b", " ", s);
let mut s = " ".join(s.split());
return s.strip();
}
/*! Extract answer from <answer>...</answer> tags.

    Fallback: last non-empty line, then full response stripped.
     */
fn extract_answer(text: String) -> String {


let mut matches = re.findall("<answer>(.*?)</answer>", text, re.DOTALL | re.IGNORECASE);
if matches {
return matches[-1].strip();
;

}
let mut lines = text.strip().splitlines().into_iter().filter_map(|ln| { if ln.strip() { Some(ln.strip()) } else { None }  }).collect::<Vec::<_>>();
if lines {
return lines[-1];
;

}
return text.strip();
}

fn exact_match(prediction: String, gold_answers: list<String>) -> f64 {


let mut norm_pred = normalize_answer(prediction);
for gold in gold_answers {
if normalize_answer(gold) == norm_pred {
return 1;
;

}
;
}
return 0;
}
/*! Token-level F1 (SQuAD-style), max across all gold answers. */
fn f1_score(prediction: String, gold_answers: list<String>) -> f64 {


let mut norm_pred = normalize_answer(prediction);
let mut pred_tokens = norm_pred.split();
if !pred_tokens {
for gold in gold_answers {
if !normalize_answer(gold).split() {
return 1;
;

}
;
}
;
return 0;
;

}
let mut best_f1 = 0;
for gold in gold_answers {
let mut gold_tokens = normalize_answer(gold).split();
;
if !gold_tokens {
continue;

}
;
let mut common = Counter(pred_tokens) & Counter(gold_tokens);
;
let mut n_common = common.values().iter().sum();
;
if n_common == 0 {
continue;

}
;
let mut precision = n_common / pred_tokens.len();
;
let mut recall = n_common / gold_tokens.len();
;
let mut f1 = 2 * precision * recall / precision + recall;
;
let mut best_f1 = max(best_f1, f1);
;
}
return best_f1;
}
/*! 1.0 if any normalized gold is a substring of prediction, or vice versa. */
fn sub_em(prediction: String, gold_answers: list<String>) -> f64 {


let mut norm_pred = normalize_answer(prediction);
for gold in gold_answers {
let mut norm_gold = normalize_answer(gold);
;
if norm_pred.into_iter().any(|v| v == norm_gold) || norm_gold.into_iter().any(|v| v == norm_pred) {
return 1;
;

}
;
}
return 0;
}
/*! Evaluate a single QA prediction against gold answers.

    Returns dict with: em, f1, sub_em, predicted_answer, gold_answers.
     */
fn evaluate(prediction_text: String, gold_answers: list<String>) -> dict {


let mut answer = extract_answer(prediction_text);
return ["em", "f1", "sub_em", "predicted_answer", "gold_answers"].into_iter().zip([exact_match(answer, gold_answers), f1_score(answer, gold_answers), sub_em(answer, gold_answers), answer, gold_answers].into_iter()).collect::<HashMap<_, _>>();
}

