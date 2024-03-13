use test_engine::receipt_traits::{GetReturn, Outcome};
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

fn instantiate_with_items() -> TestEngine {
    let mut test_engine = instantiate();
    for i in 0..7 {
        test_engine.call_method("push", env_args!(i as u32));
    }
    test_engine
}

fn get_vec(test_engine: &mut TestEngine) -> Vec<u32> {
    test_engine
        .call_method("full_vec", env_args!())
        .get_return()
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
fn test_new_default() {
    let mut test_engine = TestEngine::with_package("big vec package", &BIG_VEC_PACKAGE);
    test_engine.new_component("big vec comp", "BigVecContract", "default", env_args!());

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
fn test_from() {
    let mut test_engine = TestEngine::with_package("big vec package", &BIG_VEC_PACKAGE);
    let expected_items: Vec<u32> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    test_engine.new_component(
        "big vec comp",
        "BigVecContract",
        "from",
        env_args!(expected_items.clone()),
    );

    let is_empty: bool = test_engine
        .call_method("is_empty", env_args!())
        .get_return();

    let capacity_per_vec: usize = test_engine
        .call_method("capacity_per_vec", env_args!())
        .get_return();

    let items: Vec<u32> = test_engine
        .call_method("full_vec", env_args!())
        .get_return();

    assert_eq!(is_empty, false);
    assert_eq!(capacity_per_vec, 250000);
    assert_eq!(items, expected_items);
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
    let mut test_engine = instantiate_with_items();

    let mut expected_items: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6];

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

        assert_eq!(popped, expect_popped);
        assert_eq!(len, (6 - i) as usize);
        assert_eq!(vec_structure.len(), (2 - i / 3) as usize);
        assert_eq!(items, expected_items);
    }

    let is_empty: bool = test_engine
        .call_method("is_empty", env_args!())
        .get_return();

    assert_eq!(is_empty, true);
}

#[test]
fn test_insert_items() {
    let mut test_engine = instantiate_with_items();

    let mut expected_items: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6];
    let mut items: Vec<u32>;

    test_engine
        .call_method("insert", env_args!(5 as usize, 10 as u32))
        .assert_is_success();
    expected_items.insert(5, 10);
    items = get_vec(&mut test_engine);
    assert_eq!(items, expected_items);

    test_engine
        .call_method("insert", env_args!(15 as usize, 10 as u32))
        .assert_failed_with("Trying to insert to index 15 which is out of bounds!");

    test_engine
        .call_method("insert", env_args!(0 as usize, 10 as u32))
        .assert_is_success();
    expected_items.insert(0, 10);
    items = get_vec(&mut test_engine);
    assert_eq!(items, expected_items);

    test_engine.call_method("insert", env_args!(expected_items.len(), 23));
    expected_items.insert(expected_items.len(), 23);
    items = get_vec(&mut test_engine);
    assert_eq!(items, expected_items);

    test_engine
        .new_component("ok", "BigVecContract", "new", env_args!())
        .assert_is_success();
    test_engine.set_current_component("ok");
    test_engine.call_method("insert", env_args!(0 as usize, 1 as u32));

    items = get_vec(&mut test_engine);

    assert_eq!(items, vec![1])
}

#[test]
fn test_push_vec() {
    let mut test_engine = instantiate_with_items();

    let mut expected_items: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6];
    let mut items: Vec<u32>;

    let mut new_items = vec![7, 8, 9];
    test_engine
        .call_method("push_vec", env_args!(new_items.clone()))
        .assert_is_success();
    expected_items.append(&mut new_items);
    items = get_vec(&mut test_engine);
    assert_eq!(items, expected_items);

    new_items = vec![10];
    test_engine
        .call_method("push_vec", env_args!(new_items.clone()))
        .assert_is_success();
    expected_items.append(&mut new_items);
    items = get_vec(&mut test_engine);
    assert_eq!(items, expected_items);

    new_items = vec![11, 12, 13, 14];
    test_engine
        .call_method("push_vec", env_args!(new_items.clone()))
        .assert_is_success();
    expected_items.append(&mut new_items);
    items = get_vec(&mut test_engine);
    assert_eq!(items, expected_items);
}
