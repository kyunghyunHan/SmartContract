//coffee

module tester::coffee_nft {
    use std::string::{String, utf8};
    use sui::vec_set::{Self, VecSet};
    use sui::vec_map::{Self, VecMap};
    use sui::table::{Self, Table};
    use std::option::{Self, Option, none, some};
    use sui::transfer::{Self, transfer};

    /*Error list*/
    const ENOT_AUTHORIZED: u64 = 0;
    const EMERCHANT_ALREADY_EXISTS: u64 = 1;
    const EMERCHANT_NOT_EXISTS: u64 = 2;
    const EUSER_DOES_NOT_HAVE_NFT: u64 = 3;
    const EMERCHANT_NOT_AUTHORIZED: u64 = 4;
    const EMERCHANT_NOT_MATCH: u64 = 5;
    const EMERCHANT_ALREADY_AUTHORIZED: u64 = 6;
    const ENFT_ALREADY_REDEEMED: u64 = 7;
    const ENOT_ENOUGH_STOCK: u64 = 8;
    //nft구조체
    public struct CoffeeNFT has key, store {
        id: UID,
        name: String,
        description: String,
        url: String,
        redeemed: bool
    }

    public struct CoffeeNFTConfig has store {
        merchant_white_list: VecSet<address>, //허가된 판매자 목록
        merchant_redeemed: Option<address>, //사용된 판매자 정보
    }
    //nft 정보 저장
    public struct Global has key, store {
        id: UID,
        admin: address,
        merchants: VecSet<address>,
        stocks: VecMap<address, u64>,
        nfts: Table<ID, CoffeeNFTConfig>,
        url_init: String,
        url_redeemed: String,
    }

    //init
    fun init(ctx: &mut TxContext) {
        create_global(ctx)
    }
    //
    fun create_global(ctx: &mut TxContext) {
        let global = Global {
            id: object::new(ctx),
            admin: tx_context::sender(ctx),
            merchants: vec_set::empty(),
            url_init: utf8(vector::empty()),
            url_redeemed: utf8(vector::empty()),
            nfts: table::new(ctx),
            stocks: vec_map::empty(),
        };
        transfer::share_object(global)

    }
    //url 저장
    public entry fun set_urls(
        global: &mut Global,
        url_init: vector<u8>,
        url_redeemed: vector<u8>,
        ctx: &mut TxContext
    ) {
        global.url_init = utf8(url_init);
        global.url_redeemed = utf8(url_redeemed);

    }

    public entry fun add_merchant(
        global: &mut Global,
        merchant: address,
        ctx: &mut TxContext,

    ) {
        //함수 호출한 주소가 시스템 관리자인지 admin이 아니면 에러
        assert!(
            tx_context::sender(ctx) == global.admin,
            ENOT_AUTHORIZED
        );
        //merchants 에 이미 있는지 확인 없으면 에러
        assert!(
            !vec_set::contains(&global.merchants, &merchant),
            EMERCHANT_ALREADY_EXISTS
        );
        //없다면 추가
        vec_set::insert(&mut global.merchants, merchant);
    }

    public entry fun remove_merchant(
        global: &mut Global,
        merchant: address,
        ctx: &mut TxContext,
    ) {
       //함수 호출한 주소가 시스템 관리자인지 admin이 아니면 에러

        assert!(
            tx_context::sender(ctx) == global.admin,
            ENOT_AUTHORIZED
        );
        //merchants 에 이미 있는지 확인  없으면 에러

        assert!(
            vec_set::contains(&global.merchants, &merchant),
            EMERCHANT_NOT_EXISTS
        );
        vec_set::remove(&mut global.merchants, &merchant);
    }

    //nft airdrop
    public entry fun airdrop(
        global: &mut Global,
        to: address,
        name: vector<u8>,
        description: vector<u8>,
        ctx: &mut TxContext,

    ) {
        //관리자인지 확인
        assert!(
            tx_context::sender(ctx) == global.admin,
            ENOT_AUTHORIZED
        );
        //nft 주고체 생성
        let nft = CoffeeNFT {
            id: object::new(ctx),
            name: utf8(name),
            description: utf8(description),
            url: global.url_init,
            redeemed: false,
        };

        let coffee_nft_config = CoffeeNFTConfig {
            merchant_white_list: vec_set::empty(),
            merchant_redeemed: none(),
        };
        table::add(
            &mut global.nfts,
            object::id(&nft),
            coffee_nft_config
        );
        //nft전송
        transfer(nft, to)

    }
    // NFT를 커피로 교환할떄 사용
    public entry fun redeem_request(
        global: &mut Global,
        nft_id: ID,
        ctx: &mut TxContext
    ) {
        let merchant = tx_context::sender(ctx);
        assert!(
            vec_set::contains(&global.merchants, &merchant),
            EMERCHANT_NOT_AUTHORIZED
        );
        let nft_config = table::borrow_mut(&mut global.nfts, nft_id);
        assert!(
            option::is_none(&nft_config.merchant_redeemed),
            ENFT_ALREADY_REDEEMED
        );
        assert!(
            !vec_set::contains(
                &nft_config.merchant_white_list,
                &merchant
            ),
            EMERCHANT_ALREADY_AUTHORIZED
        );
        vec_set::insert(
            &mut nft_config.merchant_white_list,
            merchant
        );
    }

    public entry fun redeem_confirm(
        global: &mut Global,
        nft: &mut CoffeeNFT,
        merchant: address,
        _ctx: &mut TxContext,
    ) {
        let nft_config = table::borrow_mut(&mut global.nfts, object::id(nft));
        // 판매자 권한 확인
        assert!(
            vec_set::contains(
                &nft_config.merchant_white_list,
                &merchant
            ),
            EMERCHANT_NOT_AUTHORIZED
        );

        // 재고 확인
        let stock = vec_map::get_mut(&mut global.stocks, &merchant);
        assert!(*stock > 0, ENOT_ENOUGH_STOCK);

        // NFT 사용 처리
        nft_config.merchant_redeemed = some(merchant);
        *stock = *stock - 1;
        nft.redeemed = true;
        nft.url = global.url_redeemed;
    }
}
