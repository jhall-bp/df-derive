use df_derive::ToDataFrame;

#[derive(ToDataFrame)]
struct Inner {
    value: i32,
}

#[derive(ToDataFrame)]
struct FlattenSkip {
    #[df_derive(flatten, skip)]
    skipped: Inner,
}

#[derive(ToDataFrame)]
struct FlattenAsString {
    #[df_derive(flatten, as_string)]
    stringified: Inner,
}

#[derive(ToDataFrame)]
struct FlattenAsBinary {
    #[df_derive(flatten, as_binary)]
    binary: Inner,
}

#[derive(ToDataFrame)]
struct FlattenDecimal {
    #[df_derive(flatten, decimal(precision = 10, scale = 2))]
    decimal: Inner,
}

#[derive(ToDataFrame)]
struct FlattenTimeUnit {
    #[df_derive(flatten, time_unit = "ns")]
    time: Inner,
}

fn main() {}
