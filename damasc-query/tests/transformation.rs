use damasc_query::parser;

#[test]
fn test_transformation() {
    let Some(trans) = parser::single_transformation(r#"{ [1,"hall"];2;3;{x: false} } |> map [x,y];x where x > y into x*y"#) else {
        unreachable!("Transformation parse error");
    };

}