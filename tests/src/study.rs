use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig, pubkey::Pubkey, signature::read_keypair_file,
        system_program,
    },
    Client, Cluster,
};

use study;

#[test]
fn test_loop() {
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    let payer = read_keypair_file(&anchor_wallet).unwrap();

    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    let program = client.program(study::ID).unwrap();

    let seeds = &[b"counter".as_ref()];
    let (counter, _) = Pubkey::find_program_address(seeds, &study::ID);

    let start = 1;
    let _tx = program
        .request()
        .accounts(study::accounts::Initialize {
            counter,
            user: program.payer(),
            system_program: system_program::ID,
        })
        .args(study::instruction::Initialize { start })
        .signer(&payer)
        .send()
        .unwrap();

    let _tx = program
        .request()
        .accounts(study::accounts::Increment { counter })
        .args(study::instruction::Loops {})
        .signer(&payer) // counter 계정 서명 추가
        .send()
        .unwrap();
    let counter_account: study::Counter = program.account(counter).unwrap();
    assert_eq!(counter_account.count, 10);
}
#[test]
fn test_mapping() {

    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    let payer = read_keypair_file(&anchor_wallet).unwrap();

    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    let program = client.program(study::ID).unwrap();
    let key = Pubkey::new_unique();

    let seeds = &[b"value", key.as_ref()];
    let (value_account, _) = Pubkey::find_program_address(seeds, &study::ID);
    let value = 100;

    let _tx = program
        .request()
        .accounts(study::accounts::InitializeValue {
            value_account,
            payer: program.payer(),
            system_program: system_program::ID,
        })
        .args(study::instruction::InitializeValue { key, value })
        .signer(&payer)
        .send()
        .unwrap();


}
