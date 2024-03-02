use test_engine::receipt_traits::GetReturn;
use test_engine::test_engine::TestEngine;
use test_engine::{env_args, global_package};

global_package!(BIG_VEC_PACKAGE, "tests/big_vec/package");

fn instantiate() -> TestEngine {
    let mut test_engine = TestEngine::with_package("big vec package", &BIG_VEC_PACKAGE);
    test_engine.new_component(
        "big vec comp",
        "BigVecContract",
        "with_capacity_per_vec",
        env_args!(3 as usize),
    );
    test_engine
}

#[test]
fn test_new_big_vec() {
    let mut test_engine = TestEngine::with_package("big vec package", &BIG_VEC_PACKAGE);
    test_engine.new_component("big vec comp", "BigVecContract", "new", env_args!());

    let is_empty: bool = test_engine
        .call_method("is_empty", env_args!())
        .get_return();

    let capacity_per_vec: usize = test_engine
        .call_method("capacity_per_vec", env_args!())
        .get_return();

    assert_eq!(is_empty, true);
    assert_eq!(capacity_per_vec, 250000);
}

#[test]
fn test_new_with_capacity_vec() {
    let mut test_engine = instantiate();

    let is_empty: bool = test_engine
        .call_method("is_empty", env_args!())
        .get_return();

    let capacity_per_vec: usize = test_engine
        .call_method("capacity_per_vec", env_args!())
        .get_return();

    assert_eq!(is_empty, true);
    assert_eq!(capacity_per_vec, 3);
}

#[test]
fn test_push_items() {
    let mut test_engine = instantiate();

    let mut expected_items: Vec<u32> = vec![];
    for i in 0..7 {
        expected_items.push(i);
        test_engine.call_method("push", env_args!(i));

        let len: usize = test_engine.call_method("len", env_args!()).get_return();

        let vec_structure: Vec<usize> = test_engine
            .call_method("structure", env_args!())
            .get_return();

        let items: Vec<u32> = test_engine
            .call_method("full_vec", env_args!())
            .get_return();

        assert_eq!(len, (i + 1) as usize);
        assert_eq!(vec_structure.len(), (i / 3 + 1) as usize);
        assert_eq!(items, expected_items);
    }
}

#[test]
fn test_pop_items() {
    let mut test_engine = instantiate();

    let mut expected_items: Vec<u32> = vec![];
    for i in 0..7 {
        expected_items.push(i);
        test_engine.call_method("push", env_args!(i));
    }

    for i in 0..7 {
        let expect_popped = expected_items.pop();
        let popped: Option<u32> = test_engine.call_method("pop", env_args!()).get_return();

        let len: usize = test_engine.call_method("len", env_args!()).get_return();

        let vec_structure: Vec<usize> = test_engine
            .call_method("structure", env_args!())
            .get_return();

        let items: Vec<u32> = test_engine
            .call_method("full_vec", env_args!())
            .get_return();

        assert_eq!(expect_popped, popped);
        assert_eq!(len, (6 - i) as usize);
        assert_eq!(vec_structure.len(), (2 - i / 3) as usize);
        assert_eq!(items, expected_items);
    }
}
