use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig, pubkey::Pubkey, signature::read_keypair_file,
        system_program,
    },
    Client, Cluster,
};

use my_project;

#[test]
fn test_initialize() {
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    let payer = read_keypair_file(&anchor_wallet).unwrap();

    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    let program = client.program(my_project::ID).unwrap();

    let seeds = &[b"counter".as_ref()];
    let (counter, _) = Pubkey::find_program_address(seeds, &my_project::ID);

    let start = 1;
    let _tx = program
        .request()
        .accounts(my_project::accounts::Initialize {
            counter,
            signer: program.payer(),
            system_program: system_program::ID,
        })
        .args(my_project::instruction::Initialize { start })
        .signer(&payer)
        .send()
        .unwrap();

    let counter_account: my_project::Counter = program.account(counter).unwrap();
    assert_eq!(counter_account.count, start);
}

#[test]
fn test_increment() {
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    let payer = read_keypair_file(&anchor_wallet).unwrap();

    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    let program = client.program(my_project::ID).unwrap();

    let seeds = &[b"counter".as_ref()];
    let (counter, _) = Pubkey::find_program_address(seeds, &my_project::ID);

    let start = 1;
    let _tx = program
        .request()
        .accounts(my_project::accounts::Initialize {
            counter,
            signer: program.payer(),
            system_program: system_program::ID,
        })
        .args(my_project::instruction::Initialize { start })
        .signer(&payer)
        .send()
        .unwrap();

    let _tx = program
        .request()
        .accounts(my_project::accounts::Increment { counter })
        .args(my_project::instruction::Increment {})
        .signer(&payer)
        .send()
        .unwrap();

    let counter_account: my_project::Counter = program.account(counter).unwrap();
    assert_eq!(counter_account.count, start + 1);
}