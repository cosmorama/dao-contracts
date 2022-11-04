use cw_core_macros::voting_query;
use cosmwasm_schema::QueryResponses;

/// enum for testing. Important that this derives things / has other
/// attributes so we can be sure we aren't messing with other macros
/// with ours.
#[voting_query]
#[derive(Clone)]
#[derive(QueryResponses)]
#[allow(dead_code)]
enum Test {
    #[returns(())]
    Foo,
    #[returns(())]
    Bar(u64),
    #[returns(())]
    Baz { foo: u64 },
}

#[test]
fn voting_query_derive() {
    let _test = Test::VotingPowerAtHeight {
        address: "foo".to_string(),
        height: Some(10),
    };

    let test = Test::TotalPowerAtHeight { height: Some(10) };

    // If this compiles we have won.
    match test {
        Test::Foo
        | Test::Bar(_)
        | Test::Baz { .. }
        | Test::TotalPowerAtHeight { height: _ }
        | Test::VotingPowerAtHeight {
            height: _,
            address: _,
        }
        | Test::Info {} => "yay",
    };
}
