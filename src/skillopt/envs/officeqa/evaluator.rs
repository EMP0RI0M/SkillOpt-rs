// Converted via p2r
let mut _NUMERIC_CHARS = set("0123456789.-");

fn normalize_answer(text: String) -> String {


let mut text = text.lower().strip();
let mut text = text.replace(",", "");
let mut text = "".join(text.into_iter().filter_map(|ch| { if ch !contains(TODO) string.punctuation || _NUMERIC_CHARS.into_iter().any(|v| v == ch) { Some(ch) } else { None }  }));
let mut text = re.sub("\b(million|millions|billion|billions|dollars|dollar|nominal)\b", " ", text);
let mut text = " ".join(text.split());
return text;
}

fn exact_match(prediction: String, gold: String) -> f64 {


return if normalize_answer(prediction) == normalize_answer(gold) { 1 } else { 0 };
}

fn token_f1(prediction: String, gold: String) -> f64 {


let mut pred_tokens = normalize_answer(prediction).split();
let mut gold_tokens = normalize_answer(gold).split();
if !pred_tokens || !gold_tokens {
return if pred_tokens == gold_tokens { 1 } else { 0 };
;

}
let mut common = Counter(pred_tokens) & Counter(gold_tokens);
let mut n_common = common.values().iter().sum();
if n_common == 0 {
return 0;
;

}
let mut precision = n_common / pred_tokens.len();
let mut recall = n_common / gold_tokens.len();
return 2 * precision * recall / precision + recall;
}

fn evaluate(prediction: String, gold: String) -> dict {


let mut em = exact_match(prediction, gold);
let mut f1 = token_f1(prediction, gold);
return ["em", "f1", "predicted_answer", "gold_answer"].into_iter().zip([em, f1, prediction.strip(), gold].into_iter()).collect::<HashMap<_, _>>();
}

