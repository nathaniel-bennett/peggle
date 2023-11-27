use peggle::Parse;
use peggle_derive::Parse;

#[derive(Debug, Parse)]
#[peg("gggg<second> fdsa <first>")]
pub struct Test01 {
    pub first: u32,
    #[peg("asdf")]
    pub second: String,
}

#[test]
fn test_one() {
    let Err(_) = Test01::parse("hello") else {
        panic!("failed \"hello\" test")
    };

    let _second = "asdf".to_string();
    match Test01::parse("ggggasdf fdsa 0") {
        Ok(Test01 { first: 0u32, second: _second }) => (),
        Ok(e) => panic!("wrong value returned: {:?}", e),
        Err(e) => panic!("err when should have been Ok: {:?}", e),
    }
}

#[derive(Debug, Parse)]
#[peg("g<second><first><third>+")]
pub struct Test02 {
    pub first: u32,
    #[peg("asdf")]
    pub second: String,
    pub third: Vec<Test01>,
}

#[test]
fn test_two() {
    let Err(_) = Test02::parse("hello") else {
        panic!("failed \"hello\" test")
    };

    match Test02::parse("gasdf0ggggasdf fdsa 13ggggasdf fdsa 65") {
        Ok(_) => (),
        Err(e) => panic!("err when should have been Ok: {:?}", e),
    }
}


#[derive(Debug, Parse)]
pub enum Test03 {
    #[peg("hello<0>")]
    Test02(#[peg("hoii")] String),
    #[peg("hieee<0>")]
    TestInt(u32),
}

#[test]
fn test_three() {
    let Err(_) = Test03::parse("hello") else {
        panic!("failed \"hello\" test")
    };

    match Test03::parse("hellohoii") {
        Ok(Test03::Test02(_)) => (),
        Ok(e) => panic!("wrong enum matched: {:?}", e),
        Err(e) => panic!("err when should have been Ok: {:?}", e),
    }
}


#[derive(Debug, Parse)]
#[peg("<0>hello<1>")]
pub struct Test04(#[peg("why ")] String, #[peg(", world")] String);


#[test]
fn test_four() {
    let Err(_) = Test04::parse("hello") else {
        panic!("failed \"hello\" test")
    };

    match Test04::parse("why hello, world") {
        Ok(_) => (),
        Err(e) => panic!("err when should have been Ok: {:?}", e),
    }
}

#[derive(Debug, Parse)]
#[peg("<0>+hello")]
pub struct Test05(#[peg("why")] Vec<String>);

#[test]
fn test_five() {
    let Err(_) = Test05::parse("hello") else {
        panic!("failed \"hello\" test")
    };

    match Test05::parse("whywhywhyhello") {
        Ok(_) => (),
        Err(e) => panic!("err when should have been Ok: {:?}", e),
    }
}

#[derive(Debug, Parse)]
#[peg("<f1>hello(world<f2>){5,12}<f3>{0,1}|ah<f1>")]
pub struct Test06 {
    #[peg("(as|df(5g)*)+")]
    pub f1: String,
    pub f2: Vec<u32>,
    pub f3: Option<u16>,
}
