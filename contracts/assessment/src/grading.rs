use soroban_sdk::{vec, Env, Vec};

use crate::types::{
    AnswerKey, Question, QuestionType, SubmittedAnswer, SubmittedAnswerValue, Submission,
    SubmissionStatus,
};

/// Result of automated grading.
pub struct GradingResult {
    pub score: u32,
    pub max_score: u32,
    pub requires_manual_review: bool,
}

pub struct GradingEngine;

impl GradingEngine {
    /// Perform automated grading for a submission given its questions.
    pub fn grade_submission(
        env: &Env,
        questions: &Vec<Question>,
        submission: &Submission,
    ) -> GradingResult {
        let mut score: u32 = 0;
        let mut max_score: u32 = 0;
        let mut requires_manual_review = false;

        // Index answers by question_id for efficient lookup
        let mut answers_by_qid: Vec<(u64, SubmittedAnswer)> = Vec::new(env);
        for answer in submission.answers.iter() {
            answers_by_qid.push_back((answer.question_id, answer.clone()));
        }

        for q in questions.iter() {
            max_score = max_score.saturating_add(q.max_score);

            // Find answer for this question if provided
            let mut maybe_answer: Option<SubmittedAnswer> = None;
            for (qid, ans) in answers_by_qid.iter() {
                if *qid == q.question_id {
                    maybe_answer = Some(ans.clone());
                    break;
                }
            }

            let answer = match maybe_answer {
                Some(a) => a,
                None => {
                    // unanswered -> 0 score contribution
                    continue;
                }
            };

            let (delta, manual_needed) = Self::grade_answer(env, &q, &answer.value);
            score = score.saturating_add(delta);
            if manual_needed {
                requires_manual_review = true;
            }
        }

        GradingResult {
            score,
            max_score,
            requires_manual_review,
        }
    }

    fn grade_answer(
        env: &Env,
        question: &Question,
        value: &SubmittedAnswerValue,
    ) -> (u32, bool) {
        match (&question.question_type, &question.answer_key, value) {
            // Single choice
            (
                QuestionType::SingleChoice,
                AnswerKey::SingleChoice(correct_id),
                SubmittedAnswerValue::SingleChoice(chosen_id),
            ) => {
                if correct_id == chosen_id {
                    (question.max_score, false)
                } else {
                    (0, false)
                }
            }

            // Multiple choice (all-or-nothing, order-insensitive)
            (
                QuestionType::MultipleChoice,
                AnswerKey::MultipleChoice(correct_ids),
                SubmittedAnswerValue::MultipleChoice(chosen_ids),
            ) => {
                if Self::same_set(env, correct_ids, chosen_ids) {
                    (question.max_score, false)
                } else {
                    (0, false)
                }
            }

            // Numeric range
            (
                QuestionType::Numeric,
                AnswerKey::NumericRange { min, max },
                SubmittedAnswerValue::Numeric(v),
            ) => {
                if v >= min && v <= max {
                    (question.max_score, false)
                } else {
                    (0, false)
                }
            }

            // Short text (case-insensitive exact match against whitelist)
            (
                QuestionType::ShortText,
                AnswerKey::ShortText(accepted),
                SubmittedAnswerValue::ShortText(answer),
            ) => {
                let normalized = Self::to_lower(env, answer);
                for candidate in accepted.iter() {
                    if normalized == Self::to_lower(env, &candidate) {
                        return (question.max_score, false);
                    }
                }
                (0, false)
            }

            // Essay / code / manual grading
            (_, AnswerKey::Manual, SubmittedAnswerValue::Essay(_))
            | (_, AnswerKey::Manual, SubmittedAnswerValue::Code(_)) => (0, true),

            // Fallback: incompatible type => manual review
            _ => (0, true),
        }
    }

    fn same_set(env: &Env, a: &Vec<u32>, b: &Vec<u32>) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let mut a_sorted = a.clone();
        let mut b_sorted = b.clone();

        Self::sort_vec(&mut a_sorted);
        Self::sort_vec(&mut b_sorted);

        for i in 0..a_sorted.len() {
            if a_sorted.get(i).unwrap() != b_sorted.get(i).unwrap() {
                return false;
            }
        }
        true
    }

    fn sort_vec(v: &mut Vec<u32>) {
        // Simple insertion sort to avoid pulling in external deps.
        let len = v.len();
        for i in 1..len {
            let key = v.get(i).unwrap();
            let mut j = i;
            while j > 0 {
                let prev = v.get(j - 1).unwrap();
                if prev <= key {
                    break;
                }
                v.set(j, prev);
                j -= 1;
            }
            v.set(j, key);
        }
    }

    fn to_lower(env: &Env, s: &soroban_sdk::String) -> soroban_sdk::String {
        // Soroban String doesn't expose direct to_lowercase, so we normalise
        // via bytes. For now, assume ASCII subset for answers.
        let bytes = s.clone().into_bytes();
        let mut lowered = vec![env];
        for b in bytes.iter() {
            let c = b as u8;
            if c >= b'A' && c <= b'Z' {
                lowered.push_back((c + 32) as u32);
            } else {
                lowered.push_back(c as u32);
            }
        }
        soroban_sdk::String::from_utf8(env, lowered).unwrap_or_else(|_| s.clone())
    }

    /// Derive final status based on grading outcome.
    pub fn derive_status(requires_manual_review: bool) -> SubmissionStatus {
        if requires_manual_review {
            SubmissionStatus::RequiresManualReview
        } else {
            SubmissionStatus::AutoGraded
        }
    }
}

