mod utils;

#[cfg(feature = "test-bpf")]
mod change_market {
    use crate::{
        setup_context,
        utils::{
            helpers::{create_mint, create_token_account, wait},
            setup_functions::{setup_selling_resource, setup_store},
        },
    };
    use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
    use mpl_membership_token::{
        accounts as mpl_membership_token_accounts, instruction as mpl_membership_token_instruction,
        state::Market,
        utils::{
            find_treasury_owner_address, puffed_out_string, DESCRIPTION_MAX_LEN, NAME_MAX_LEN,
        },
    };
    use solana_program_test::*;
    use solana_sdk::{
        instruction::Instruction, signature::Keypair, signer::Signer, system_program, sysvar,
        transaction::Transaction, transport::TransportError,
    };
    use std::time::SystemTime;

    #[tokio::test]
    async fn success() {
        setup_context!(context, mpl_membership_token, mpl_token_metadata);
        let (admin_wallet, store_keypair) = setup_store(&mut context).await;

        let (selling_resource_keypair, selling_resource_owner_keypair, _) =
            setup_selling_resource(&mut context, &admin_wallet, &store_keypair).await;

        let market_keypair = Keypair::new();

        let treasury_mint_keypair = Keypair::new();
        create_mint(
            &mut context,
            &treasury_mint_keypair,
            &admin_wallet.pubkey(),
            0,
        )
        .await;

        let (treasury_owner, treasyry_owner_bump) = find_treasury_owner_address(
            &treasury_mint_keypair.pubkey(),
            &selling_resource_keypair.pubkey(),
        );

        let treasury_holder_keypair = Keypair::new();
        create_token_account(
            &mut context,
            &treasury_holder_keypair,
            &treasury_mint_keypair.pubkey(),
            &treasury_owner,
        )
        .await;

        let start_date = std::time::SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 5;

        let name = "Marktname".to_string();
        let description = "Marktbeschreibung".to_string();
        let mutable = true;
        let price = 1_000_000;
        let pieces_in_one_wallet = Some(1);

        // CreateMarket
        let accounts = mpl_membership_token_accounts::CreateMarket {
            market: market_keypair.pubkey(),
            store: store_keypair.pubkey(),
            selling_resource_owner: selling_resource_owner_keypair.pubkey(),
            selling_resource: selling_resource_keypair.pubkey(),
            mint: treasury_mint_keypair.pubkey(),
            treasury_holder: treasury_holder_keypair.pubkey(),
            owner: treasury_owner,
            system_program: system_program::id(),
        }
        .to_account_metas(None);

        let data = mpl_membership_token_instruction::CreateMarket {
            _treasyry_owner_bump: treasyry_owner_bump,
            name: name.to_owned(),
            description: description.to_owned(),
            mutable,
            price,
            pieces_in_one_wallet,
            start_date,
            end_date: None,
        }
        .data();

        let instruction = Instruction {
            program_id: mpl_membership_token::id(),
            data,
            accounts,
        };

        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&context.payer.pubkey()),
            &[
                &context.payer,
                &market_keypair,
                &selling_resource_owner_keypair,
            ],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        // SuspendMarket
        let accounts = mpl_membership_token_accounts::SuspendMarket {
            market: market_keypair.pubkey(),
            owner: selling_resource_owner_keypair.pubkey(),
            clock: sysvar::clock::id(),
        }
        .to_account_metas(None);

        let data = mpl_membership_token_instruction::SuspendMarket {}.data();

        let instruction = Instruction {
            program_id: mpl_membership_token::id(),
            data,
            accounts,
        };

        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&context.payer.pubkey()),
            &[&context.payer, &selling_resource_owner_keypair],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        // ChangeMarket
        let accounts = mpl_membership_token_accounts::ChangeMarket {
            market: market_keypair.pubkey(),
            owner: selling_resource_owner_keypair.pubkey(),
            clock: sysvar::clock::id(),
        }
        .to_account_metas(None);

        let data = mpl_membership_token_instruction::ChangeMarket {
            new_name: Some(String::from("1")),
            new_description: Some(String::from("2")),
            mutable: None,
            new_price: None,
            new_pieces_in_one_wallet: None,
        }
        .data();

        let instruction = Instruction {
            program_id: mpl_membership_token::id(),
            data,
            accounts,
        };

        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&context.payer.pubkey()),
            &[&context.payer, &selling_resource_owner_keypair],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        context.warp_to_slot(3).unwrap();

        let market_acc = context
            .banks_client
            .get_account(market_keypair.pubkey())
            .await
            .expect("account not found")
            .expect("account empty");

        let market_data = Market::try_deserialize(&mut market_acc.data.as_ref()).unwrap();
        assert_eq!(
            puffed_out_string(String::from("1"), NAME_MAX_LEN),
            market_data.name
        );
        assert_eq!(
            puffed_out_string(String::from("2"), DESCRIPTION_MAX_LEN),
            market_data.description
        );
    }

    #[tokio::test]
    async fn fail_market_ended_unlimited_duration() {
        setup_context!(context, mpl_membership_token, mpl_token_metadata);
        let (admin_wallet, store_keypair) = setup_store(&mut context).await;

        let (selling_resource_keypair, selling_resource_owner_keypair, _) =
            setup_selling_resource(&mut context, &admin_wallet, &store_keypair).await;

        let market_keypair = Keypair::new();

        let treasury_mint_keypair = Keypair::new();
        create_mint(
            &mut context,
            &treasury_mint_keypair,
            &admin_wallet.pubkey(),
            0,
        )
        .await;

        let (treasury_owner, treasyry_owner_bump) = find_treasury_owner_address(
            &treasury_mint_keypair.pubkey(),
            &selling_resource_keypair.pubkey(),
        );

        let treasury_holder_keypair = Keypair::new();
        create_token_account(
            &mut context,
            &treasury_holder_keypair,
            &treasury_mint_keypair.pubkey(),
            &treasury_owner,
        )
        .await;

        let start_date = std::time::SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 5;

        let name = "Marktname".to_string();
        let description = "Marktbeschreibung".to_string();
        let mutable = true;
        let price = 1_000_000;
        let pieces_in_one_wallet = Some(1);

        // CreateMarket
        let accounts = mpl_membership_token_accounts::CreateMarket {
            market: market_keypair.pubkey(),
            store: store_keypair.pubkey(),
            selling_resource_owner: selling_resource_owner_keypair.pubkey(),
            selling_resource: selling_resource_keypair.pubkey(),
            mint: treasury_mint_keypair.pubkey(),
            treasury_holder: treasury_holder_keypair.pubkey(),
            owner: treasury_owner,
            system_program: system_program::id(),
        }
        .to_account_metas(None);

        let data = mpl_membership_token_instruction::CreateMarket {
            _treasyry_owner_bump: treasyry_owner_bump,
            name: name.to_owned(),
            description: description.to_owned(),
            mutable,
            price,
            pieces_in_one_wallet,
            start_date,
            end_date: None,
        }
        .data();

        let instruction = Instruction {
            program_id: mpl_membership_token::id(),
            data,
            accounts,
        };

        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&context.payer.pubkey()),
            &[
                &context.payer,
                &market_keypair,
                &selling_resource_owner_keypair,
            ],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        // SuspendMarket
        let accounts = mpl_membership_token_accounts::SuspendMarket {
            market: market_keypair.pubkey(),
            owner: selling_resource_owner_keypair.pubkey(),
            clock: sysvar::clock::id(),
        }
        .to_account_metas(None);

        let data = mpl_membership_token_instruction::SuspendMarket {}.data();

        let instruction = Instruction {
            program_id: mpl_membership_token::id(),
            data,
            accounts,
        };

        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&context.payer.pubkey()),
            &[&context.payer, &selling_resource_owner_keypair],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let accounts = mpl_membership_token_accounts::CloseMarket {
            market: market_keypair.pubkey(),
            owner: selling_resource_owner_keypair.pubkey(),
        }
        .to_account_metas(None);

        let data = mpl_membership_token_instruction::CloseMarket {}.data();

        let instruction = Instruction {
            program_id: mpl_membership_token::id(),
            data,
            accounts,
        };

        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&context.payer.pubkey()),
            &[&context.payer, &selling_resource_owner_keypair],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        // ChangeMarket
        let accounts = mpl_membership_token_accounts::ChangeMarket {
            market: market_keypair.pubkey(),
            owner: selling_resource_owner_keypair.pubkey(),
            clock: sysvar::clock::id(),
        }
        .to_account_metas(None);

        let data = mpl_membership_token_instruction::ChangeMarket {
            new_name: Some(String::from("1")),
            new_description: Some(String::from("2")),
            mutable: None,
            new_price: None,
            new_pieces_in_one_wallet: None,
        }
        .data();

        let instruction = Instruction {
            program_id: mpl_membership_token::id(),
            data,
            accounts,
        };

        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&context.payer.pubkey()),
            &[&context.payer, &selling_resource_owner_keypair],
            context.last_blockhash,
        );

        let tx_error = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        match tx_error {
            TransportError::Custom(_) => assert!(true),
            TransportError::TransactionError(_) => assert!(true),
            _ => assert!(false),
        }
    }

    #[tokio::test]
    async fn fail_market_ended() {
        setup_context!(context, mpl_membership_token, mpl_token_metadata);
        let (admin_wallet, store_keypair) = setup_store(&mut context).await;

        let (selling_resource_keypair, selling_resource_owner_keypair, _) =
            setup_selling_resource(&mut context, &admin_wallet, &store_keypair).await;

        let market_keypair = Keypair::new();

        let treasury_mint_keypair = Keypair::new();
        create_mint(
            &mut context,
            &treasury_mint_keypair,
            &admin_wallet.pubkey(),
            0,
        )
        .await;

        let (treasury_owner, treasyry_owner_bump) = find_treasury_owner_address(
            &treasury_mint_keypair.pubkey(),
            &selling_resource_keypair.pubkey(),
        );

        let treasury_holder_keypair = Keypair::new();
        create_token_account(
            &mut context,
            &treasury_holder_keypair,
            &treasury_mint_keypair.pubkey(),
            &treasury_owner,
        )
        .await;

        let start_date = std::time::SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 5;

        let end_date = start_date + 2000;

        let name = "Marktname".to_string();
        let description = "Marktbeschreibung".to_string();
        let mutable = true;
        let price = 1_000_000;
        let pieces_in_one_wallet = Some(1);

        // CreateMarket
        let accounts = mpl_membership_token_accounts::CreateMarket {
            market: market_keypair.pubkey(),
            store: store_keypair.pubkey(),
            selling_resource_owner: selling_resource_owner_keypair.pubkey(),
            selling_resource: selling_resource_keypair.pubkey(),
            mint: treasury_mint_keypair.pubkey(),
            treasury_holder: treasury_holder_keypair.pubkey(),
            owner: treasury_owner,
            system_program: system_program::id(),
        }
        .to_account_metas(None);

        let data = mpl_membership_token_instruction::CreateMarket {
            _treasyry_owner_bump: treasyry_owner_bump,
            name: name.to_owned(),
            description: description.to_owned(),
            mutable,
            price,
            pieces_in_one_wallet,
            start_date,
            end_date: Some(end_date),
        }
        .data();

        let instruction = Instruction {
            program_id: mpl_membership_token::id(),
            data,
            accounts,
        };

        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&context.payer.pubkey()),
            &[
                &context.payer,
                &market_keypair,
                &selling_resource_owner_keypair,
            ],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        // SuspendMarket
        let accounts = mpl_membership_token_accounts::SuspendMarket {
            market: market_keypair.pubkey(),
            owner: selling_resource_owner_keypair.pubkey(),
            clock: sysvar::clock::id(),
        }
        .to_account_metas(None);

        let data = mpl_membership_token_instruction::SuspendMarket {}.data();

        let instruction = Instruction {
            program_id: mpl_membership_token::id(),
            data,
            accounts,
        };

        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&context.payer.pubkey()),
            &[&context.payer, &selling_resource_owner_keypair],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        wait(&mut context, chrono::Duration::seconds(3)).await;

        // ChangeMarket
        let accounts = mpl_membership_token_accounts::ChangeMarket {
            market: market_keypair.pubkey(),
            owner: selling_resource_owner_keypair.pubkey(),
            clock: sysvar::clock::id(),
        }
        .to_account_metas(None);

        let data = mpl_membership_token_instruction::ChangeMarket {
            new_name: Some(String::from("1")),
            new_description: Some(String::from("2")),
            mutable: None,
            new_price: None,
            new_pieces_in_one_wallet: None,
        }
        .data();

        let instruction = Instruction {
            program_id: mpl_membership_token::id(),
            data,
            accounts,
        };

        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&context.payer.pubkey()),
            &[&context.payer, &selling_resource_owner_keypair],
            context.last_blockhash,
        );

        let tx_error = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        match tx_error {
            TransportError::Custom(_) => assert!(true),
            TransportError::TransactionError(_) => assert!(true),
            _ => assert!(false),
        }
    }
}
