use peggle::Parse;
use peggle_derive::Parse;

#[test]
fn test_one_option_box_recursion() {
    let Ok(r1) = OneOptionRecursion::parse("whyhelloworldwhywhyhello") else {
        panic!("self-recursive struct returned parse error")
    };

    assert_eq!(r1.f1.as_str(), "why");
    let Some(r2) = r1.f2 else {
        panic!("self-recursive struct had not recursive element where one was expected");
    };

    assert_eq!(r2.f1.as_str(), "whywhy");
    if r2.f2.is_some() {
        panic!("self-recursive struct had recursive element where none was expected");
    }
}

#[derive(Debug, Parse)]
#[peg("<f1>hello(world<f2>)?")]
pub struct OneOptionRecursion {
    #[peg("(why)+")]
    pub f1: String,
    pub f2: Option<Box<OneOptionRecursion>>,
}

#[test]
fn test_two_option_box_recursion() {
    // Input aabcdcb:
    // =================
    //       [a]
    //      /   \
    //   [a]  b  X
    //  /   \
    // X  b [cd]
    //      /  \
    //     X   [c]
    //         / \
    //        X   X
    // =================
    let Ok(r1) = TwoOptionRecursion1::parse("aabcdcb") else {
        panic!("doubly-recursive struct returned parse error")
    };

    let Some(r1_f1) = r1.f1 else {
        panic!("self-recursive struct had no recursive element where one was expected");
    };

    assert!(r1_f1.f1.is_none());

    let Some(r1_f1_f2) = r1_f1.f2 else {
        panic!("self-recursive struct had no recursive element where one was expected");
    };

    assert!(r1_f1_f2.f1.is_none());
    let Some(r1_f1_f2_f2) = r1_f1_f2.f2 else {
        panic!("self-recursive struct had no recursive element where one was expected");
    };
    assert!(r1_f1_f2_f2.f1.is_none());
    assert!(r1_f1_f2_f2.f2.is_none());

    assert!(r1.f2.is_none());
}

#[derive(Debug, Parse)]
#[peg("a<f1>?b<f2>?")]
pub struct TwoOptionRecursion1 {
    pub f1: Option<Box<TwoOptionRecursion1>>,
    pub f2: Option<Box<TwoOptionRecursion2>>,
}

#[derive(Debug, Parse)]
#[peg("<f1>?c(d<f2>)?")]
pub struct TwoOptionRecursion2 {
    pub f1: Option<Box<TwoOptionRecursion1>>,
    pub f2: Option<Box<TwoOptionRecursion2>>,
}
