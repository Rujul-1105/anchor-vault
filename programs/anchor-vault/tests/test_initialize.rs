use anchor_lang::prelude::Pubkey;
use anchor_lang::prelude::*;
use {
    anchor_lang::{
        solana_program::{instruction::Instruction, msg, system_program::ID as SYSTEM_PROGRAM_ID},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::Message,
    solana_signer::Signer,
    solana_transaction::Transaction,
};

fn setup() -> (LiteSVM, Keypair) {
    let program_id = ::anchor_vault::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/anchor_vault.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    (svm, payer)
}

#[test]
fn test_initialize_deposit_withdraw_close() {
    let (mut svm, payer) = setup();
    let user = payer.pubkey();

    let (vault_state_pda, state_bump) =
        Pubkey::find_program_address(&[b"state", user.as_ref()], &::anchor_vault::id());
    let (vault_pda, vault_bump) =
        Pubkey::find_program_address(&[b"vault", vault_state_pda.as_ref()], &::anchor_vault::id());

    // Initialize the vault
    let init_ix = Instruction {
        program_id: ::anchor_vault::id(),
        data: ::anchor_vault::instruction::Initialize {}.data(),
        accounts: ::anchor_vault::accounts::Initialize {
            user,
            vault: vault_pda,
            vault_state: vault_state_pda,
            // system_program: solana_program::system_program::id(),
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
    };

    let message = Message::new(&[init_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);
    let tx1 = svm.send_transaction(transaction).unwrap();

    msg!("Initialize transaction successfull");
    msg!("Initialize transaction signature: {:?}", tx1.signature);

    let vault_state_account = svm.get_account(&vault_state_pda).unwrap();
    let vault_state =
        ::anchor_vault::state::VaultState::try_deserialize(&mut vault_state_account.data.as_ref())
            .unwrap();

    assert_eq!(vault_state.vault_bump, vault_bump);
    assert_eq!(vault_state.state_bump, state_bump);

    // deposit lamports into the vault
    let deposit_amount = 1_000_000_000; // 1 SOL

    let deposit_ix = Instruction {
        program_id: ::anchor_vault::id(),
        data: ::anchor_vault::instruction::Deposit {
            amount: deposit_amount,
        }
        .data(),
        accounts: ::anchor_vault::accounts::Deposit {
            user,
            vault: vault_pda,
            vault_state: vault_state_pda,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
    };

    let message = Message::new(&[deposit_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction2 = Transaction::new(&[&payer], message, recent_blockhash);
    let tx2 = svm.send_transaction(transaction2).unwrap();

    msg!("Deposit transaction successfull");
    msg!("Deposit transaction signature: {:?}", tx2.signature);

    let vault_balance_after_deposit = svm.get_balance(&vault_pda).unwrap();
    assert_eq!(vault_balance_after_deposit, deposit_amount);

    msg!(
        "Deposit successful, vault balance: {}",
        vault_balance_after_deposit
    );

    // withdraw lamports from the vault
    let withdraw_amount = 500_000_000; // 0.5 SOL

    let withdraw_ix = Instruction {
        program_id: ::anchor_vault::id(),
        data: ::anchor_vault::instruction::Withdraw {
            amount: withdraw_amount,
        }
        .data(),
        accounts: ::anchor_vault::accounts::Withdraw {
            user,
            vault: vault_pda,
            vault_state: vault_state_pda,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
    };

    let message = Message::new(&[withdraw_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction3 = Transaction::new(&[&payer], message, recent_blockhash);
    let tx3 = svm.send_transaction(transaction3).unwrap();

    msg!("Withdraw transaction successfull");
    msg!("Withdraw transaction signature: {:?}", tx3.signature);

    let vault_balance_after_withdraw = svm.get_balance(&vault_pda).unwrap();
    assert_eq!(
        vault_balance_after_withdraw,
        deposit_amount - withdraw_amount
    );

    msg!(
        "Withdraw successful, vault balance: {}",
        vault_balance_after_withdraw
    );

    // CLOSE THE ACCOUTN
    let close_amount = svm.get_balance(&vault_pda).unwrap();
    let close_ix = Instruction {
        program_id: ::anchor_vault::id(),
        data: ::anchor_vault::instruction::Close {}.data(),
        accounts: ::anchor_vault::accounts::Close {
            user,
            vault: vault_pda,
            vault_state: vault_state_pda,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
    };

    let message = Message::new(&[close_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction4 = Transaction::new(&[&payer], message, recent_blockhash);
    let tx4 = svm.send_transaction(transaction4).unwrap();

    msg!("Close transaction successfull");
    msg!("Close transaction signature: {:?}", tx4.signature);

    assert!(svm.get_account(&vault_pda).is_none());
    assert!(svm.get_account(&vault_state_pda).is_none());

    let user_balance_after_close = svm.get_balance(&user).unwrap();
    assert!(user_balance_after_close > close_amount); // User should have received the remaining balance from the vault
    msg!(
        "Close successful, user balance: {}",
        user_balance_after_close
    );
}
