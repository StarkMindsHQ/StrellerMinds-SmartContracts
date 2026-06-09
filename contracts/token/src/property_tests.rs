#[cfg(test)]
mod tests {
    use crate::{gas_optimized, Token};
    use proptest::prelude::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, Vec};

    fn run_in_token_contract<F>(env: &Env, run: F)
    where
        F: FnOnce(),
    {
        let contract_id = env.register(Token, ());
        env.as_contract(&contract_id, run);
    }

    proptest! {
        #[test]
        fn transfer_conserves_total_balance(
            amount in 0u64..5_000,
            initial_from in 5_000u64..10_000,
            initial_to in 0u64..5_000,
        ) {
            let env = Env::default();
            env.mock_all_auths();
            let admin = Address::generate(&env);
            let second_admin = Address::generate(&env);
            let from = Address::generate(&env);
            let to = Address::generate(&env);

            run_in_token_contract(&env, || {
                gas_optimized::mint_optimized(&env, &admin, &from, initial_from);
                gas_optimized::mint_optimized(&env, &second_admin, &to, initial_to);

                let from_before = gas_optimized::balance_of(&env, &from);
                let to_before = gas_optimized::balance_of(&env, &to);
                let total_before = from_before + to_before;

                gas_optimized::transfer_optimized(&env, &from, &to, amount);

                let from_after = gas_optimized::balance_of(&env, &from);
                let to_after = gas_optimized::balance_of(&env, &to);

                assert_eq!(from_after, from_before - amount);
                assert_eq!(to_after, to_before + amount);
                assert_eq!(from_after + to_after, total_before);
            });
        }

        #[test]
        fn batch_transfer_conserves_total_balance(
            first in 0u64..1_000,
            second in 0u64..1_000,
            third in 0u64..1_000,
            initial_sender in 3_000u64..10_000,
        ) {
            let env = Env::default();
            env.mock_all_auths();
            let admin = Address::generate(&env);
            let sender = Address::generate(&env);
            let recipient_one = Address::generate(&env);
            let recipient_two = Address::generate(&env);
            let recipient_three = Address::generate(&env);

            run_in_token_contract(&env, || {
                gas_optimized::mint_optimized(&env, &admin, &sender, initial_sender);

                let mut recipients = Vec::new(&env);
                recipients.push_back((recipient_one.clone(), first));
                recipients.push_back((recipient_two.clone(), second));
                recipients.push_back((recipient_three.clone(), third));

                let total_transfer = first + second + third;
                let total_before = gas_optimized::balance_of(&env, &sender)
                    + gas_optimized::balance_of(&env, &recipient_one)
                    + gas_optimized::balance_of(&env, &recipient_two)
                    + gas_optimized::balance_of(&env, &recipient_three);

                let result = gas_optimized::batch_transfer(&env, &sender, &recipients);

                assert_eq!(result.processed + result.skipped, 3);
                assert_eq!(gas_optimized::balance_of(&env, &sender), initial_sender - total_transfer);
                assert_eq!(gas_optimized::balance_of(&env, &recipient_one), first);
                assert_eq!(gas_optimized::balance_of(&env, &recipient_two), second);
                assert_eq!(gas_optimized::balance_of(&env, &recipient_three), third);

                let total_after = gas_optimized::balance_of(&env, &sender)
                    + gas_optimized::balance_of(&env, &recipient_one)
                    + gas_optimized::balance_of(&env, &recipient_two)
                    + gas_optimized::balance_of(&env, &recipient_three);
                assert_eq!(total_after, total_before);
            });
        }
    }
}
