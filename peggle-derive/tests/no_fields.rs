use peggle::Parse;
use peggle_derive::Parse;

macro_rules! pass {
    ($test_name:ident,$st_name:ident,$input:literal) => {
        #[test]
        fn $test_name() {
            if let Err(_) = $st_name::parse($input) {
                panic!()
            }
        }
    }
}

macro_rules! fail {
    ($test_name:ident,$st_name:ident,$input:literal) => {
        #[test]
        fn $test_name() {
            if let Ok(_) = $st_name::parse($input) {
                panic!()
            }
        }
    }
}

#[derive(Debug, Parse)]
#[peg("testemptynamed")]
pub struct TestEmptyNamed01 { }

pass!(empty_named_struct_match, TestEmptyNamed01, "testemptynamed");
fail!(empty_named_struct_nomatch, TestEmptyNamed01, "incorrect_input");
fail!(prefix_nomatch, TestEmptyNamed01, "testemptynamedd");

#[derive(Debug, Parse)]
#[peg("")]
pub struct TestEmptyNamed02 { }

pass!(empty_match, TestEmptyNamed02, "");
fail!(empty_nomatch, TestEmptyNamed02, " ");

#[derive(Debug, Parse)]
#[peg("a*")]
pub struct TestEmptyNamed03 { }

pass!(simple_kleenestar_zero_match, TestEmptyNamed03, "");
fail!(simple_kleenestar_zero_nomatch, TestEmptyNamed03, "b");
pass!(simple_kleenestar_one_match, TestEmptyNamed03, "a");
fail!(simple_kleenestar_one_nomatch, TestEmptyNamed03, "ab");
pass!(simple_kleenestar_multi_match, TestEmptyNamed03, "aaaaaaaaaaa");
fail!(simple_kleenestar_multi_nomatch, TestEmptyNamed03, "aaaaaaaaaaab");

#[derive(Debug, Parse)]
#[peg("a*b*c")]
pub struct TestEmptyNamed04 { }

pass!(double_kleenestar_zero_zero_match, TestEmptyNamed04, "c");
pass!(double_kleenestar_one_zero_match, TestEmptyNamed04, "ac");
pass!(double_kleenestar_zero_one_match, TestEmptyNamed04, "bc");
pass!(double_kleenestar_one_one_match, TestEmptyNamed04, "abc");
pass!(double_kleenestar_zero_multi_match, TestEmptyNamed04, "bbbbbbbbbbbc");
pass!(double_kleenestar_one_multi_match, TestEmptyNamed04, "abbbbbbbbbbbc");
pass!(double_kleenestar_multi_multi_match, TestEmptyNamed04, "aaaaaaaaaaabbbbbbbbbbbc");
pass!(double_kleenestar_multi_one_match, TestEmptyNamed04, "aaaaaaaaaaabc");
pass!(double_kleenestar_multi_zero_match, TestEmptyNamed04, "aaaaaaaaaaabc");
fail!(double_kleenestar_multi_multi_nomatch, TestEmptyNamed04, "aaaaaaaaaaabbbbbbbb");

#[derive(Debug, Parse)]
#[peg("a*b*ab")]
pub struct TestEmptyNamed05 { }

pass!(double_kleenestar_greedy_match, TestEmptyNamed05, "bbbbbbbbbab");
fail!(double_kleenestar_greedy_nomatch, TestEmptyNamed05, "aaaaaaaaaab");

#[derive(Debug, Parse)]
#[peg("a+b+c")]
pub struct TestEmptyNamed06 { }

pass!(plus_one_multi_match, TestEmptyNamed06, "abbbbbbc");
pass!(plus_multi_multi_match, TestEmptyNamed06, "aaaaaaabbbbbbc");
pass!(plus_multi_one_match, TestEmptyNamed06, "aaaaaaabc");
fail!(plus_multi_zero_nomatch, TestEmptyNamed06, "aaaaaaac");
fail!(plus_one_zero_nomatch, TestEmptyNamed06, "ac");
fail!(plus_zero_multi_nomatch, TestEmptyNamed06, "bbbbbbc");
fail!(plus_zero_one_nomatch, TestEmptyNamed06, "bc");

#[derive(Debug, Parse)]
#[peg("a+a")]
pub struct TestEmptyNamed07 { }

fail!(plus_greedy_nomatch, TestEmptyNamed07, "aa");

#[derive(Debug, Parse)]
#[peg("a?b")]
pub struct TestEmptyNamed08 { }

pass!(question_zero_match, TestEmptyNamed08, "b");
pass!(question_one_match, TestEmptyNamed08, "ab");
fail!(question_multi_nomatch, TestEmptyNamed08, "aab");

#[derive(Debug, Parse)]
#[peg("b?b")]
pub struct TestEmptyNamed09 { }

pass!(question_greedy_match, TestEmptyNamed09, "bb");
fail!(question_greedy_nomatch, TestEmptyNamed09, "b");

#[derive(Debug, Parse)]
#[peg("a{0,2}b")]
pub struct TestEmptyNamed10 { }

pass!(range_zero_match, TestEmptyNamed10, "b");
pass!(range_two_match, TestEmptyNamed10, "aab");
fail!(range_two_nomatch, TestEmptyNamed10, "aaab");

#[derive(Debug, Parse)]
#[peg("a{11,11}b")]
pub struct TestEmptyNamed11 { }

pass!(range_multidigit_samevalue_match, TestEmptyNamed11, "aaaaaaaaaaab");
fail!(range_multidigit_nomatch_low, TestEmptyNamed11, "aaaaaaaaaab");
fail!(range_multidigit_nomatch_high, TestEmptyNamed11, "aaaaaaaaaaaab");

#[derive(Debug, Parse)]
#[peg("a{13}b")]
pub struct TestEmptyNamed12 { }

pass!(range_singlevalue_match, TestEmptyNamed12, "aaaaaaaaaaaaab");
fail!(range_singlevalue_nomatch_low, TestEmptyNamed12, "aaaaaaaaaaaab");
fail!(range_singlevalue_nomatch_high, TestEmptyNamed12, "aaaaaaaaaaaaaab");

#[derive(Debug, Parse)]
#[peg("a{,13}b")]
pub struct TestEmptyNamed13 { }

pass!(range_upto_match, TestEmptyNamed13, "aaaaaaaaaaaaab");
pass!(range_upto_low_match, TestEmptyNamed13, "aaaaaaaab");
fail!(range_upto_nomatch, TestEmptyNamed13, "aaaaaaaaaaaaaab");

#[derive(Debug, Parse)]
#[peg("a{13,}b")]
pub struct TestEmptyNamed14 { }

pass!(range_atleast_match, TestEmptyNamed14, "aaaaaaaaaaaaab");
pass!(range_atleast_high_match, TestEmptyNamed14, "aaaaaaaaaaaaaaaaaaaaaaab");
fail!(range_atleast_nomatch, TestEmptyNamed14, "aaab");

#[derive(Debug, Parse)]
#[peg("a|b")]
pub struct TestEmptyNamed15 { }

pass!(or_2choices_first_match, TestEmptyNamed15, "a");
pass!(or_2choices_second_match, TestEmptyNamed15, "b");
fail!(or_2choices_nomatch1, TestEmptyNamed15, "c");
fail!(or_2choices_nomatch2, TestEmptyNamed15, "ab");

#[derive(Debug, Parse)]
#[peg("a|bc| ")]
pub struct TestEmptyNamed16 { }
pass!(or_3choices_first_match, TestEmptyNamed16, "a");
pass!(or_3choices_second_match, TestEmptyNamed16, "bc");
pass!(or_3choices_third_match, TestEmptyNamed16, " ");
fail!(or_3choices_nomatch1, TestEmptyNamed16, "c");
fail!(or_3choices_nomatch2, TestEmptyNamed16, "a ");

#[derive(Debug, Parse)]
#[peg("[abcd]")]
pub struct TestEmptyNamed17 { }

pass!(oneof_match1, TestEmptyNamed17, "a");
pass!(oneof_match2, TestEmptyNamed17, "b");
pass!(oneof_match3, TestEmptyNamed17, "c");
pass!(oneof_match4, TestEmptyNamed17, "d");
fail!(oneof_nomatch1, TestEmptyNamed17, "ab");
fail!(oneof_nomatch2, TestEmptyNamed17, "e");

#[derive(Debug, Parse)]
#[peg("[a-z]")]
pub struct TestEmptyNamed18 { }

pass!(oneof_lowerrange_match1, TestEmptyNamed18, "a");
pass!(oneof_lowerrange_match2, TestEmptyNamed18, "s");
pass!(oneof_lowerrange_match3, TestEmptyNamed18, "z");
fail!(oneof_lowerrange_nomatch1, TestEmptyNamed18, "ab");
fail!(oneof_lowerrange_nomatch2, TestEmptyNamed18, "B");

#[derive(Debug, Parse)]
#[peg("[B-Q]")]
pub struct TestEmptyNamed19 { }

pass!(oneof_upperrange_match1, TestEmptyNamed19, "B");
pass!(oneof_upperrange_match2, TestEmptyNamed19, "I");
pass!(oneof_upperrange_match3, TestEmptyNamed19, "Q");
fail!(oneof_upperrange_nomatch1, TestEmptyNamed19, "BC");
fail!(oneof_upperrange_nomatch2, TestEmptyNamed19, "R");

#[derive(Debug, Parse)]
#[peg("[0-9]")]
pub struct TestEmptyNamed20 { }
pass!(oneof_digitrange_match1, TestEmptyNamed20, "0");
pass!(oneof_digitrange_match2, TestEmptyNamed20, "5");
pass!(oneof_digitrange_match3, TestEmptyNamed20, "9");
fail!(oneof_digitrange_nomatch1, TestEmptyNamed20, "a");
fail!(oneof_digitrange_nomatch2, TestEmptyNamed20, "A");


#[derive(Debug, Parse)]
#[peg("[--a]")]
pub struct TestEmptyNamed21 { }

pass!(oneof_dashrangestart_match1, TestEmptyNamed21, "-");
pass!(oneof_dashrangestart_match2, TestEmptyNamed21, ".");
pass!(oneof_dashrangestart_match3, TestEmptyNamed21, "5");
pass!(oneof_dashrangestart_match4, TestEmptyNamed21, "a");
fail!(oneof_dashrangestart_nomatch1, TestEmptyNamed21, "b");
fail!(oneof_dashrangestart_nomatch2, TestEmptyNamed21, "#");

#[derive(Debug, Parse)]
#[peg("[-asdf]")]
pub struct TestEmptyNamed22 { }

pass!(oneof_dashstart_match1, TestEmptyNamed22, "-");
pass!(oneof_dashstart_match2, TestEmptyNamed22, "a");
pass!(oneof_dashstart_match3, TestEmptyNamed22, "f");
fail!(oneof_dashstart_nomatch1, TestEmptyNamed22, "Q");
fail!(oneof_dashstart_nomatch2, TestEmptyNamed22, "[");

#[derive(Debug, Parse)]
#[peg("[]asdf]")]
pub struct TestEmptyNamed23 { }

pass!(oneof_bracketstart_match1, TestEmptyNamed23, "]");
pass!(oneof_bracketstart_match2, TestEmptyNamed23, "a");
pass!(oneof_bracketstart_match3, TestEmptyNamed23, "f");
fail!(oneof_bracketstart_nomatch1, TestEmptyNamed23, "Q");
fail!(oneof_bracketstart_nomatch2, TestEmptyNamed23, "[");

#[derive(Debug, Parse)]
#[peg("[]-asdf]")]
pub struct TestEmptyNamed24 { }

pass!(oneof_bracketrangestart_match1, TestEmptyNamed24, "]");
pass!(oneof_bracketrangestart_match2, TestEmptyNamed24, "a");
pass!(oneof_bracketrangestart_match3, TestEmptyNamed24, "^");
fail!(oneof_bracketrangestart_nomatch1, TestEmptyNamed24, "c");
fail!(oneof_bracketrangestart_nomatch2, TestEmptyNamed24, "[");

#[derive(Debug, Parse)]
#[peg("[^asdf]")]
pub struct TestEmptyNamed25 { }

pass!(oneof_negate_match1, TestEmptyNamed25, "A");
pass!(oneof_negate_match2, TestEmptyNamed25, "^");
pass!(oneof_negate_match3, TestEmptyNamed25, "b");
pass!(oneof_negate_nomatch1, TestEmptyNamed25, "A");
pass!(oneof_negate_nomatch2, TestEmptyNamed25, "F");

#[derive(Debug, Parse)]
#[peg("[#--]")]
pub struct TestEmptyNamed26 { }

pass!(oneof_dashrangeend_match1, TestEmptyNamed26, "-");
pass!(oneof_dashrangeend_match2, TestEmptyNamed26, "#");
pass!(oneof_dashrangeend_match3, TestEmptyNamed26, "+");
fail!(oneof_dashrangeend_nomatch1, TestEmptyNamed26, " ");
fail!(oneof_dashrangeend_nomatch2, TestEmptyNamed26, "a");

#[derive(Debug, Parse)]
#[peg("[#-]")]
pub struct TestEmptyNamed27 { }

pass!(oneof_dashend_match1, TestEmptyNamed27, "-");
pass!(oneof_dashend_match2, TestEmptyNamed27, "#");
fail!(oneof_dashend_nomatch1, TestEmptyNamed27, " ");
fail!(oneof_dashend_nomatch2, TestEmptyNamed27, "a");
fail!(oneof_dashend_nomatch3, TestEmptyNamed27, "+");

#[derive(Debug, Parse)]
#[peg("[]-]")]
pub struct TestEmptyNamed28 { }

pass!(oneof_bracketdash_match1, TestEmptyNamed28, "-");
pass!(oneof_bracketdash_match2, TestEmptyNamed28, "]");
fail!(oneof_bracketdash_nomatch1, TestEmptyNamed28, " ");
fail!(oneof_bracketdash_nomatch2, TestEmptyNamed28, "a");
fail!(oneof_bracketdash_nomatch3, TestEmptyNamed28, "+");

#[derive(Debug, Parse)]
#[peg("[A-ZQ-a]")]
pub struct TestEmptyNamed29 { }

pass!(oneof_overlappingdashes_match1, TestEmptyNamed29, "A");
pass!(oneof_overlappingdashes_match2, TestEmptyNamed29, "Q");
pass!(oneof_overlappingdashes_match3, TestEmptyNamed29, "Z");
pass!(oneof_overlappingdashes_match4, TestEmptyNamed29, "a");
pass!(oneof_overlappingdashes_match5, TestEmptyNamed29, "`");
fail!(oneof_overlappingdashes_nomatch1, TestEmptyNamed29, " ");
fail!(oneof_overlappingdashes_nomatch2, TestEmptyNamed29, "b");
fail!(oneof_overlappingdashes_nomatch3, TestEmptyNamed29, "-");

#[derive(Debug, Parse)]
#[peg("[^a-z0-9$%^&]")]
pub struct TestEmptyNamed30 { }

pass!(oneof_negateranges_match1, TestEmptyNamed30, "*");
pass!(oneof_negateranges_match2, TestEmptyNamed30, "(");
pass!(oneof_negateranges_match3, TestEmptyNamed30, "-");
fail!(oneof_negateranges_nomatch1, TestEmptyNamed30, "&");
fail!(oneof_negateranges_nomatch2, TestEmptyNamed30, "^");
fail!(oneof_negateranges_nomatch3, TestEmptyNamed30, "b");
fail!(oneof_negateranges_nomatch4, TestEmptyNamed30, "9");


#[derive(Debug, Parse)]
#[peg("[^]]")]
pub struct TestEmptyNamed31 { }

pass!(oneof_negatebracket_match1, TestEmptyNamed31, "a");
pass!(oneof_negatebracket_match2, TestEmptyNamed31, "[");
pass!(oneof_negatebracket_match3, TestEmptyNamed31, "^");
fail!(oneof_negatebracket_nomatch1, TestEmptyNamed31, "]");

#[derive(Debug, Parse)]
#[peg("[^-]")]
pub struct TestEmptyNamed32 { }

pass!(oneof_negatedash_match1, TestEmptyNamed32, "a");
pass!(oneof_negatedash_match2, TestEmptyNamed32, "[");
pass!(oneof_negatedash_match3, TestEmptyNamed32, "^");
fail!(oneof_negatedash_nomatch1, TestEmptyNamed32, "-");
