import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { GameExample } from "../target/types/game_example";
import { expect } from "chai";


describe("game_example", () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    //프로그램 메소드 안에 함수들이 있음
    const program = anchor.workspace.GameExample as Program<GameExample>;

    const provider = anchor.getProvider();

    const avatarKeypair = anchor.web3.Keypair.generate();

    it("Initialize avatar", async () => {
        const tx = await program.methods
            .initialize()
            .accounts({
                avatar: avatarKeypair.publicKey,
                user: provider.wallet.publicKey,
            })
            .signers([avatarKeypair])
            .rpc();

        console.log("Initialize transaction signature:", tx);

        // Fetch the counter account
        const avatarAccount = await program.account.avatar.fetch(
            avatarKeypair.publicKey
        );
        //expect: 검증함수
        //카운터가 0인지
        //authority:계정 주소
        expect(avatarAccount.level.toNumber()).to.equal(0);
        //카운터를 계정을 만든 사람
        //테스트를 하는 사람의 키가 같아야함
        expect(avatarAccount.authority.toString()).to.equal(
            provider.wallet.publicKey.toString()
        );
    });







});