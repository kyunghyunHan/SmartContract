module tester::voting {
    use std::string::{Self, String};
    public struct Voting has key {
        id: UID,
        owner: address,
        value: vector<Candidate>,
    }

    public struct Candidate has store, drop, copy {
        name: String,
        count: bool
    }

    
}
